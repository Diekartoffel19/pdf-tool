use lopdf::{Document, Object, ObjectId};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub fn merge_pdfs(input_paths: &[PathBuf], output_path: &Path) -> Result<usize, String> {
    if input_paths.len() < 2 {
        return Err("Mindestens 2 PDF-Dateien zum Zusammenführen erforderlich".to_string());
    }

    let mut documents = Vec::new();
    for p in input_paths {
        let doc = Document::load(p)
            .map_err(|e| format!("Fehler beim Laden von '{}': {e}", p.display()))?;
        documents.push(doc);
    }

    let mut max_id = 1;
    let mut documents_pages = BTreeMap::new();
    let mut documents_objects = BTreeMap::new();
    let mut merged = Document::with_version("1.5");

    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    for mut doc in documents {
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;

        let pages = doc.get_pages();
        for (_, object_id) in pages {
            if let Ok(page_obj) = doc.get_object(object_id) {
                documents_pages.insert(object_id, page_obj.to_owned());
            }
        }

        documents_objects.extend(doc.objects);
    }

    for (object_id, object) in documents_objects {
        match object.type_name().unwrap_or(b"") {
            b"Catalog" => {
                if catalog_object.is_none() {
                    catalog_object = Some((object_id, object));
                }
            }
            b"Pages" => {
                if let Ok(dict) = object.as_dict() {
                    let mut dict = dict.clone();
                    if let Some((_, ref old_obj)) = pages_object {
                        if let Ok(old_dict) = old_obj.as_dict() {
                            dict.extend(old_dict);
                        }
                    }
                    let chosen_id = if let Some((id, _)) = pages_object { id } else { object_id };
                    pages_object = Some((chosen_id, Object::Dictionary(dict)));
                }
            }
            b"Page" | b"Outlines" | b"Outline" => {}
            _ => {
                merged.objects.insert(object_id, object);
            }
        }
    }

    let (pages_id, pages_obj) = pages_object.ok_or_else(|| "Keine Seitenstruktur gefunden".to_string())?;
    let (catalog_id, catalog_obj) = catalog_object.ok_or_else(|| "Kein Katalog in PDFs gefunden".to_string())?;

    for (object_id, object) in &documents_pages {
        if let Ok(dict) = object.as_dict() {
            let mut dict = dict.clone();
            dict.set("Parent", pages_id);
            merged.objects.insert(*object_id, Object::Dictionary(dict));
        }
    }

    if let Ok(dict) = pages_obj.as_dict() {
        let mut dict = dict.clone();
        dict.set("Count", documents_pages.len() as u32);
        dict.set(
            "Kids",
            documents_pages.keys().map(|id| Object::Reference(*id)).collect::<Vec<_>>(),
        );
        merged.objects.insert(pages_id, Object::Dictionary(dict));
    }

    if let Ok(dict) = catalog_obj.as_dict() {
        let mut dict = dict.clone();
        dict.set("Pages", pages_id);
        dict.remove(b"Outlines");
        merged.objects.insert(catalog_id, Object::Dictionary(dict));
    }

    merged.trailer.set("Root", catalog_id);
    merged.max_id = merged.objects.len() as u32;
    merged.renumber_objects();
    merged.adjust_zero_pages();

    if let Some(outline_id) = merged.build_outline() {
        if let Ok(Object::Dictionary(dict)) = merged.get_object_mut(catalog_id) {
            dict.set("Outlines", Object::Reference(outline_id));
        }
    }

    let total_pages = documents_pages.len();
    merged.save(output_path)
        .map_err(|e| format!("Fehler beim Speichern der zusammengeführten PDF: {e}"))?;

    Ok(total_pages)
}
