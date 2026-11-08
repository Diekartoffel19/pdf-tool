use lopdf::Document;
use std::path::Path;

pub fn delete_pages_from_pdf(
    input_path: &Path,
    pages_to_delete: &[u32],
    output_path: &Path,
) -> Result<usize, String> {
    if pages_to_delete.is_empty() {
        return Err("Keine Seiten zum Löschen angegeben".to_string());
    }

    let mut doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden von '{}': {e}", input_path.display()))?;

    let total = doc.get_pages().len() as u32;
    if pages_to_delete.len() as u32 >= total {
        return Err("Es können nicht alle Seiten eines Dokuments gelöscht werden".to_string());
    }

    doc.delete_pages(pages_to_delete);
    doc.prune_objects();
    doc.save(output_path)
        .map_err(|e| format!("Fehler beim Speichern nach dem Löschen: {e}"))?;

    let remaining = doc.get_pages().len();
    Ok(remaining)
}
