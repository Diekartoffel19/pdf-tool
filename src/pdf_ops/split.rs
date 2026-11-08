use lopdf::Document;
use std::path::Path;

pub fn extract_pages(
    input_path: &Path,
    pages_to_keep: &[u32],
    output_path: &Path,
) -> Result<usize, String> {
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
    split_by_chunks(input_path, 1, output_dir).map(|c| c as u32)
}

/// Splits a PDF into multiple PDFs, each containing up to `pages_per_chunk` pages.
pub fn split_by_chunks(
    input_path: &Path,
    pages_per_chunk: usize,
    output_dir: &Path,
) -> Result<usize, String> {
    if pages_per_chunk == 0 {
        return Err("Seiten pro Teil-PDF muss mindestens 1 sein".to_string());
    }

    let base_doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden von '{}': {e}", input_path.display()))?;

    let total = base_doc.get_pages().len() as u32;
    if total == 0 {
        return Err("Dokument enthält keine Seiten".to_string());
    }

    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("dokument");

    let all_pages: Vec<u32> = (1..=total).collect();
    let mut file_count = 0;

    for (idx, chunk) in all_pages.chunks(pages_per_chunk).enumerate() {
        let mut chunk_doc = base_doc.clone();
        let to_delete: Vec<u32> = (1..=total).filter(|p| !chunk.contains(p)).collect();

        if !to_delete.is_empty() {
            chunk_doc.delete_pages(&to_delete);
            chunk_doc.prune_objects();
        }

        let out_file = if pages_per_chunk == 1 {
            output_dir.join(format!("{stem}_seite_{:03}.pdf", chunk[0]))
        } else {
            output_dir.join(format!(
                "{stem}_teil_{:02}_s{}-{}.pdf",
                idx + 1,
                chunk.first().unwrap(),
                chunk.last().unwrap()
            ))
        };

        chunk_doc
            .save(&out_file)
            .map_err(|e| format!("Fehler beim Speichern von '{}': {e}", out_file.display()))?;
        file_count += 1;
    }

    Ok(file_count)
}
