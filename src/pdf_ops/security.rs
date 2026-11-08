use lopdf::encryption::crypt_filters::{Aes128CryptFilter, Aes256CryptFilter, CryptFilter};
use lopdf::{Document, EncryptionState, EncryptionVersion, Permissions};
use rand::Rng;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

pub fn encrypt_pdf(
    input_path: &Path,
    user_password: &str,
    owner_password: &str,
    use_aes256: bool,
    output_path: &Path,
) -> Result<(), String> {
    if user_password.is_empty() && owner_password.is_empty() {
        return Err("Es muss mindestens ein Passwort angegeben werden".to_string());
    }

    let effective_owner = if owner_password.is_empty() {
        user_password
    } else {
        owner_password
    };
    let effective_user = user_password;

    let mut doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden von '{}': {e}", input_path.display()))?;

    if doc.is_encrypted() {
        return Err("Dokument ist bereits verschlüsselt".to_string());
    }

    let permissions = Permissions::PRINTABLE
        | Permissions::COPYABLE
        | Permissions::COPYABLE_FOR_ACCESSIBILITY
        | Permissions::PRINTABLE_IN_HIGH_QUALITY;

    let mut file_encryption_key = [0u8; 32];

    let version = if use_aes256 {
        let crypt_filter: Arc<dyn CryptFilter> = Arc::new(Aes256CryptFilter);
        let mut rng = rand::rng();
        rng.fill(&mut file_encryption_key);

        EncryptionVersion::V5 {
            encrypt_metadata: true,
            crypt_filters: BTreeMap::from([(b"StdCF".to_vec(), crypt_filter)]),
            file_encryption_key: &file_encryption_key,
            stream_filter: b"StdCF".to_vec(),
            string_filter: b"StdCF".to_vec(),
            owner_password: effective_owner,
            user_password: effective_user,
            permissions,
        }
    } else {
        let crypt_filter: Arc<dyn CryptFilter> = Arc::new(Aes128CryptFilter);

        EncryptionVersion::V4 {
            document: &doc,
            encrypt_metadata: true,
            crypt_filters: BTreeMap::from([(b"StdCF".to_vec(), crypt_filter)]),
            stream_filter: b"StdCF".to_vec(),
            string_filter: b"StdCF".to_vec(),
            owner_password: effective_owner,
            user_password: effective_user,
            permissions,
        }
    };

    let state = EncryptionState::try_from(version)
        .map_err(|e| format!("Fehler bei Verschlüsselungskonfiguration: {e:?}"))?;

    doc.encrypt(&state)
        .map_err(|e| format!("Fehler beim Verschlüsseln: {e:?}"))?;

    doc.save(output_path)
        .map_err(|e| format!("Fehler beim Speichern der verschlüsselten PDF: {e}"))?;

    Ok(())
}

pub fn decrypt_pdf(input_path: &Path, password: &str, output_path: &Path) -> Result<(), String> {
    let mut doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden von '{}': {e}", input_path.display()))?;

    if !doc.is_encrypted() {
        return Err("Dokument ist nicht verschlüsselt".to_string());
    }

    doc.decrypt(password)
        .map_err(|e| format!("Falsches Passwort oder Fehler beim Entschlüsseln: {e:?}"))?;

    doc.save(output_path)
        .map_err(|e| format!("Fehler beim Speichern der entschlüsselten PDF: {e}"))?;

    Ok(())
}
