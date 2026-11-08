use lopdf::Document;
use std::path::Path;

pub fn extract_pages(input_path: &Path, pages_to_keep: &[u32], output_path: &Path) -> Result<usize, String> {
    if pages_to_keep.is_empty() {
        return Err("Keine Seiten zum Extrahieren ausgewählt".to_string());
    }

    let mut doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden von '{}': {e}", input_path.display()))?;

    let total_pages = doc.get_pages().len() as u32;
    let mut to_delete = Vec::new();

    for p in 1..=total_pages {
        if !pages_to_keep.contains(&p) {
            to_delete.push(p);
        }
    }

    if to_delete.len() as u32 == total_pages {
        return Err("Alle Seiten würden gelöscht werden".to_string());
    }

    doc.delete_pages(&to_delete);
    doc.prune_objects();
    doc.save(output_path)
        .map_err(|e| format!("Fehler beim Speichern der extrahierten PDF: {e}"))?;

    Ok(pages_to_keep.len())
}

pub fn split_all_pages(input_path: &Path, output_dir: &Path) -> Result<u32, String> {
    let base_doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden: {e}"))?;

    let total = base_doc.get_pages().len() as u32;
    if total == 0 {
        return Err("Dokument enthält keine Seiten".to_string());
    }

    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("seite");

    for p in 1..=total {
        let mut single_page_doc = Document::load(input_path)
            .map_err(|e| format!("Fehler beim Lesen für Seite {p}: {e}"))?;

        let mut to_delete = Vec::new();
        for other in 1..=total {
            if other != p {
                to_delete.push(other);
            }
        }
        single_page_doc.delete_pages(&to_delete);
        single_page_doc.prune_objects();

        let out_file = output_dir.join(format!("{stem}_seite_{p:03}.pdf"));
        single_page_doc
            .save(&out_file)
            .map_err(|e| format!("Fehler beim Speichern von Seite {p}: {e}"))?;
    }

    Ok(total)
}
