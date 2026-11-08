use lopdf::{dictionary, Document, Object};
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct PdfMetadata {
    pub title: String,
    pub author: String,
    pub subject: String,
    pub keywords: String,
    pub creator: String,
    pub producer: String,
}

fn get_string_value(obj: &Object) -> String {
    match obj {
        Object::String(bytes, _) => String::from_utf8_lossy(bytes).to_string(),
        Object::Name(bytes) => String::from_utf8_lossy(bytes).to_string(),
        _ => String::new(),
    }
}

pub fn read_metadata(input_path: &Path) -> Result<PdfMetadata, String> {
    let doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden von '{}': {e}", input_path.display()))?;

    let mut meta = PdfMetadata::default();

    if let Ok(info_ref) = doc.trailer.get(b"Info") {
        let info_dict = match info_ref {
            Object::Reference(id) => doc.get_object(*id).ok().and_then(|o| o.as_dict().ok()),
            Object::Dictionary(d) => Some(d),
            _ => None,
        };

        if let Some(d) = info_dict {
            if let Ok(v) = d.get(b"Title") {
                meta.title = get_string_value(v);
            }
            if let Ok(v) = d.get(b"Author") {
                meta.author = get_string_value(v);
            }
            if let Ok(v) = d.get(b"Subject") {
                meta.subject = get_string_value(v);
            }
            if let Ok(v) = d.get(b"Keywords") {
                meta.keywords = get_string_value(v);
            }
            if let Ok(v) = d.get(b"Creator") {
                meta.creator = get_string_value(v);
            }
            if let Ok(v) = d.get(b"Producer") {
                meta.producer = get_string_value(v);
            }
        }
    }

    Ok(meta)
}

pub fn write_metadata(
    input_path: &Path,
    metadata: &PdfMetadata,
    output_path: &Path,
) -> Result<(), String> {
    let mut doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden von '{}': {e}", input_path.display()))?;

    let info_id = if let Ok(Object::Reference(id)) = doc.trailer.get(b"Info") {
        *id
    } else {
        let new_id = doc.new_object_id();
        doc.trailer.set("Info", Object::Reference(new_id));
        new_id
    };

    let mut dict = if let Ok(info_obj) = doc.get_object(info_id) {
        info_obj.as_dict().cloned().unwrap_or_else(|_| dictionary! {})
    } else {
        dictionary! {}
    };

    let set_or_remove = |dict: &mut lopdf::Dictionary, key: &'static str, val: &str| {
        if val.trim().is_empty() {
            dict.remove(key.as_bytes());
        } else {
            dict.set(key, Object::string_literal(val));
        }
    };

    set_or_remove(&mut dict, "Title", &metadata.title);
    set_or_remove(&mut dict, "Author", &metadata.author);
    set_or_remove(&mut dict, "Subject", &metadata.subject);
    set_or_remove(&mut dict, "Keywords", &metadata.keywords);
    set_or_remove(&mut dict, "Creator", &metadata.creator);
    set_or_remove(&mut dict, "Producer", &metadata.producer);

    doc.objects.insert(info_id, Object::Dictionary(dict));
    doc.save(output_path)
        .map_err(|e| format!("Fehler beim Speichern mit neuen Metadaten: {e}"))?;

    Ok(())
}
