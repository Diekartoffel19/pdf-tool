use crate::pdf_ops::watermark::get_page_dimensions;
use lopdf::Document;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct PdfDetails {
    pub page_count: usize,
    pub version: String,
    pub object_count: usize,
    pub image_count: usize,
    pub font_count: usize,
    pub is_encrypted: bool,
    pub has_forms: bool,
    pub has_outlines: bool,
    pub file_size_bytes: u64,
    pub page_format_label: String,
}

fn classify_dimensions_mm(w_mm: u32, h_mm: u32) -> String {
    let (short, long) = if w_mm <= h_mm {
        (w_mm, h_mm)
    } else {
        (h_mm, w_mm)
    };
    let orientation = if w_mm <= h_mm { "Hochformat" } else { "Querformat" };

    let name = if short.abs_diff(210) <= 3 && long.abs_diff(297) <= 3 {
        Some("DIN A4")
    } else if short.abs_diff(297) <= 3 && long.abs_diff(420) <= 4 {
        Some("DIN A3")
    } else if short.abs_diff(148) <= 3 && long.abs_diff(210) <= 3 {
        Some("DIN A5")
    } else if short.abs_diff(216) <= 3 && long.abs_diff(279) <= 3 {
        Some("US Letter")
    } else {
        None
    };

    match name {
        Some(n) => format!("{w_mm} × {h_mm} mm ({n}, {orientation})"),
        None => format!("{w_mm} × {h_mm} mm ({orientation})"),
    }
}

pub fn get_pdf_details(input_path: &Path) -> Result<PdfDetails, String> {
    let file_size_bytes = std::fs::metadata(input_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden von '{}': {e}", input_path.display()))?;

    let pages = doc.get_pages();
    let page_count = pages.len();

    let page_format_label = if let Some((_, &first_id)) = pages.iter().next() {
        let (w_pt, h_pt) = get_page_dimensions(&doc, first_id);
        let w_mm = (w_pt / 2.834_645_7).round() as u32;
        let h_mm = (h_pt / 2.834_645_7).round() as u32;
        classify_dimensions_mm(w_mm, h_mm)
    } else {
        "Unbekannt".to_string()
    };

    let mut image_count = 0;
    let mut font_count = 0;

    for obj in doc.objects.values() {
        if let Ok(stream) = obj.as_stream() {
            if stream
                .dict
                .get(b"Subtype")
                .and_then(|s| s.as_name())
                .map(|n| n == b"Image")
                .unwrap_or(false)
            {
                image_count += 1;
            }
        } else if let Ok(dict) = obj.as_dict() {
            if dict
                .get(b"Type")
                .and_then(|t| t.as_name())
                .map(|n| n == b"Font")
                .unwrap_or(false)
            {
                font_count += 1;
            }
        }
    }

    let mut has_forms = false;
    let mut has_outlines = false;
    if let Ok(cat_id) = doc.trailer.get(b"Root").and_then(|o| o.as_reference()) {
        if let Ok(cat) = doc.get_object(cat_id).and_then(|o| o.as_dict()) {
            has_forms = cat.get(b"AcroForm").is_ok();
            has_outlines = cat.get(b"Outlines").is_ok();
        }
    }

    Ok(PdfDetails {
        page_count,
        version: doc.version.clone(),
        object_count: doc.objects.len(),
        image_count,
        font_count,
        is_encrypted: doc.is_encrypted(),
        has_forms,
        has_outlines,
        file_size_bytes,
        page_format_label,
    })
}

pub fn extract_text(input_path: &Path, pages: &[u32]) -> Result<String, String> {
    let doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden: {e}"))?;

    doc.extract_text(pages)
        .map_err(|e| format!("Fehler beim Extrahieren von Text: {e}"))
}
