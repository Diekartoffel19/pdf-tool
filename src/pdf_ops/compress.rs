use lopdf::{Document, SaveOptions};
use std::fs::File;
use std::path::Path;

pub struct CompressResult {
    pub original_size: u64,
    pub compressed_size: u64,
    pub ratio_percent: f64,
}

pub fn compress_pdf(
    input_path: &Path,
    output_path: &Path,
    max_level: bool,
) -> Result<CompressResult, String> {
    let original_size = std::fs::metadata(input_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let mut doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden von '{}': {e}", input_path.display()))?;

    if max_level {
        let mut out_file = File::create(output_path)
            .map_err(|e| format!("Fehler beim Erstellen der Ausgabedatei: {e}"))?;
        let options = SaveOptions::builder()
            .use_object_streams(true)
            .use_xref_streams(true)
            .max_objects_per_stream(200)
            .compression_level(9)
            .build();
        doc.save_with_options(&mut out_file, options)
            .map_err(|e| format!("Fehler beim komprimierten Speichern: {e}"))?;
    } else {
        doc.compress();
        doc.save(output_path)
            .map_err(|e| format!("Fehler beim Speichern: {e}"))?;
    }

    let compressed_size = std::fs::metadata(output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let ratio_percent = if original_size > 0 {
        ((original_size as f64 - compressed_size as f64) / original_size as f64) * 100.0
    } else {
        0.0
    };

    Ok(CompressResult {
        original_size,
        compressed_size,
        ratio_percent,
    })
}
