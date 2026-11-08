use lopdf::{dictionary, Document, Object, ObjectId, Stream};
use std::collections::HashSet;
use std::path::Path;

fn get_root_pages_id(doc: &Document) -> Result<ObjectId, String> {
    let catalog_id = doc
        .trailer
        .get(b"Root")
        .and_then(|o| o.as_reference())
        .map_err(|_| "Kein Root-Katalog im PDF gefunden".to_string())?;
    let catalog = doc
        .get_object(catalog_id)
        .and_then(|o| o.as_dict())
        .map_err(|_| "Ungültiger Katalog im PDF".to_string())?;
    catalog
        .get(b"Pages")
        .and_then(|o| o.as_reference())
        .map_err(|_| "Keine Pages-Referenz im Katalog gefunden".to_string())
}

/// Reorders (and optionally duplicates or filters) pages according to `new_order` (1-based page numbers).
pub fn reorder_pages(
    input_path: &Path,
    new_order: &[u32],
    output_path: &Path,
) -> Result<usize, String> {
    if new_order.is_empty() {
        return Err("Neue Seitenreihenfolge darf nicht leer sein".to_string());
    }

    let mut doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden von '{}': {e}", input_path.display()))?;

    let pages_map = doc.get_pages();
    let total = pages_map.len() as u32;
    let root_pages_id = get_root_pages_id(&doc)?;

    for &p in new_order {
        if p == 0 || p > total {
            return Err(format!(
                "Ungültige Seitennummer {p} (Dokument hat {total} Seiten)"
            ));
        }
    }

    let mut used_ids = HashSet::new();
    let mut new_kids = Vec::with_capacity(new_order.len());

    for &p in new_order {
        let orig_id = *pages_map
            .get(&p)
            .ok_or_else(|| format!("Seite {p} nicht gefunden"))?;

        let target_id = if used_ids.insert(orig_id) {
            if let Ok(dict) = doc.get_object_mut(orig_id).and_then(|o| o.as_dict_mut()) {
                dict.set("Parent", root_pages_id);
            }
            orig_id
        } else {
            // Page is duplicated in sequence -> clone page dictionary
            let mut cloned_dict = doc
                .get_object(orig_id)
                .and_then(|o| o.as_dict())
                .map_err(|_| format!("Seitenobjekt für Seite {p} ungültig"))?
                .clone();
            cloned_dict.set("Parent", root_pages_id);
            doc.add_object(Object::Dictionary(cloned_dict))
        };

        new_kids.push(Object::Reference(target_id));
    }

    let count = new_kids.len();
    if let Ok(pages_dict) = doc
        .get_object_mut(root_pages_id)
        .and_then(|o| o.as_dict_mut())
    {
        pages_dict.set("Kids", new_kids);
        pages_dict.set("Count", count as i64);
    }

    doc.prune_objects();
    doc.save(output_path)
        .map_err(|e| format!("Fehler beim Speichern der neu sortierten PDF: {e}"))?;

    Ok(count)
}

pub fn reverse_pages(input_path: &Path, output_path: &Path) -> Result<usize, String> {
    let doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden von '{}': {e}", input_path.display()))?;
    let total = doc.get_pages().len() as u32;
    if total == 0 {
        return Err("Dokument hat keine Seiten".to_string());
    }
    let rev: Vec<u32> = (1..=total).rev().collect();
    reorder_pages(input_path, &rev, output_path)
}

/// Inserts `count` blank pages after `after_page` (0 = before page 1).
pub fn insert_blank_pages(
    input_path: &Path,
    after_page: u32,
    count: usize,
    width_pt: f32,
    height_pt: f32,
    output_path: &Path,
) -> Result<usize, String> {
    if count == 0 {
        return Err("Anzahl der einzufügenden Leerseiten muss mindestens 1 sein".to_string());
    }

    let mut doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden von '{}': {e}", input_path.display()))?;

    let pages_map = doc.get_pages();
    let total = pages_map.len() as u32;
    if after_page > total {
        return Err(format!(
            "Position {after_page} liegt hinter der letzten Seite ({total})"
        ));
    }

    let root_pages_id = get_root_pages_id(&doc)?;

    let mut ordered_ids: Vec<ObjectId> = Vec::with_capacity(total as usize + count);
    for (_, id) in &pages_map {
        if let Ok(dict) = doc.get_object_mut(*id).and_then(|o| o.as_dict_mut()) {
            dict.set("Parent", root_pages_id);
        }
        ordered_ids.push(*id);
    }

    let insert_idx = after_page as usize;
    let mut blank_ids = Vec::with_capacity(count);
    for _ in 0..count {
        let empty_content_id = doc.add_object(Stream::new(dictionary! {}, Vec::new()));
        let resources_id = doc.add_object(dictionary! {});
        let blank_page = dictionary! {
            "Type" => "Page",
            "Parent" => root_pages_id,
            "MediaBox" => vec![
                0.into(),
                0.into(),
                Object::Real(width_pt),
                Object::Real(height_pt),
            ],
            "Resources" => resources_id,
            "Contents" => empty_content_id,
        };
        blank_ids.push(doc.add_object(blank_page));
    }

    ordered_ids.splice(insert_idx..insert_idx, blank_ids);
    let new_total = ordered_ids.len();

    if let Ok(pages_dict) = doc
        .get_object_mut(root_pages_id)
        .and_then(|o| o.as_dict_mut())
    {
        pages_dict.set(
            "Kids",
            ordered_ids
                .into_iter()
                .map(Object::Reference)
                .collect::<Vec<_>>(),
        );
        pages_dict.set("Count", new_total as i64);
    }

    doc.save(output_path)
        .map_err(|e| format!("Fehler beim Speichern mit Leerseiten: {e}"))?;

    Ok(new_total)
}
