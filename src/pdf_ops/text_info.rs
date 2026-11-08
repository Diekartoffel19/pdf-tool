use lopdf::Document;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct PdfDetails {
    pub page_count: usize,
    pub version: String,
    pub object_count: usize,
    pub is_encrypted: bool,
    pub file_size_bytes: u64,
}

pub fn get_pdf_details(input_path: &Path) -> Result<PdfDetails, String> {
    let file_size_bytes = std::fs::metadata(input_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden von '{}': {e}", input_path.display()))?;

    Ok(PdfDetails {
        page_count: doc.get_pages().len(),
        version: doc.version.clone(),
        object_count: doc.objects.len(),
        is_encrypted: doc.is_encrypted(),
        file_size_bytes,
    })
}

pub fn extract_text(input_path: &Path, pages: &[u32]) -> Result<String, String> {
    let doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden: {e}"))?;

    doc.extract_text(pages)
        .map_err(|e| format!("Fehler beim Extrahieren von Text: {e}"))
}
