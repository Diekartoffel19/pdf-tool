use lopdf::Document;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SanitizeOptions {
    pub remove_metadata: bool,
    pub remove_annotations: bool,
    pub remove_forms: bool,
    pub remove_outlines: bool,
    pub remove_scripts_actions: bool,
}

impl Default for SanitizeOptions {
    fn default() -> Self {
        Self {
            remove_metadata: true,
            remove_annotations: true,
            remove_forms: true,
            remove_outlines: false,
            remove_scripts_actions: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SanitizeReport {
    pub removed_annotations_pages: usize,
    pub metadata_cleared: bool,
    pub forms_removed: bool,
    pub scripts_removed: bool,
    pub pruned_objects: usize,
}

pub fn sanitize_pdf(
    input_path: &Path,
    options: &SanitizeOptions,
    output_path: &Path,
) -> Result<SanitizeReport, String> {
    let mut doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden von '{}': {e}", input_path.display()))?;

    let initial_objects = doc.objects.len();
    let mut report = SanitizeReport::default();

    if options.remove_metadata {
        if doc.trailer.remove(b"Info").is_some() {
            report.metadata_cleared = true;
        }
    }

    if let Ok(catalog_id) = doc.trailer.get(b"Root").and_then(|o| o.as_reference()) {
        if let Ok(cat) = doc.get_object_mut(catalog_id).and_then(|o| o.as_dict_mut()) {
            if options.remove_metadata && cat.remove(b"Metadata").is_some() {
                report.metadata_cleared = true;
            }
            if options.remove_forms && cat.remove(b"AcroForm").is_some() {
                report.forms_removed = true;
            }
            if options.remove_outlines {
                cat.remove(b"Outlines");
            }
            if options.remove_scripts_actions {
                if cat.remove(b"OpenAction").is_some()
                    | cat.remove(b"AA").is_some()
                    | cat.remove(b"Names").is_some()
                {
                    report.scripts_removed = true;
                }
            }
        }
    }

    let pages = doc.get_pages();
    for (_, page_id) in pages {
        if let Ok(pdict) = doc.get_object_mut(page_id).and_then(|o| o.as_dict_mut()) {
            if options.remove_annotations && pdict.remove(b"Annots").is_some() {
                report.removed_annotations_pages += 1;
            }
            if options.remove_scripts_actions && pdict.remove(b"AA").is_some() {
                report.scripts_removed = true;
            }
            if options.remove_metadata {
                pdict.remove(b"PieceInfo");
                pdict.remove(b"Metadata");
            }
        }
    }

    doc.prune_objects();
    let after_objects = doc.objects.len();
    report.pruned_objects = initial_objects.saturating_sub(after_objects);

    doc.compress();
    doc.save(output_path)
        .map_err(|e| format!("Fehler beim Speichern der bereinigten PDF: {e}"))?;

    Ok(report)
}
