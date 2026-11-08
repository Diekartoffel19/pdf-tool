use lopdf::Document;
use std::path::Path;

pub fn rotate_pdf(
    input_path: &Path,
    pages_to_rotate: Option<&[u32]>,
    angle: i64,
    output_path: &Path,
) -> Result<usize, String> {
    if angle % 90 != 0 {
        return Err("Winkel muss ein Vielfaches von 90° sein (z.B. 90, 180, 270)".to_string());
    }

    let mut doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden von '{}': {e}", input_path.display()))?;

    let pages = doc.get_pages();
    let mut modified_count = 0;

    for (page_num, page_id) in pages {
        if let Some(target_pages) = pages_to_rotate {
            if !target_pages.contains(&page_num) {
                continue;
            }
        }

        if let Ok(page_dict) = doc.get_object_mut(page_id).and_then(|o| o.as_dict_mut()) {
            let current_rotation = page_dict.get(b"Rotate").and_then(|o| o.as_i64()).unwrap_or(0);
            let mut new_rotation = (current_rotation + angle) % 360;
            if new_rotation < 0 {
                new_rotation += 360;
            }
            page_dict.set("Rotate", new_rotation);
            modified_count += 1;
        }
    }

    doc.save(output_path)
        .map_err(|e| format!("Fehler beim Speichern der gedrehten PDF: {e}"))?;

    Ok(modified_count)
}
