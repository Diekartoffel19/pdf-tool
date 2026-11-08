use lopdf::{Document, Object, ObjectId};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn combine_documents_ordered(
    mut documents: Vec<Document>,
    interleave_two: Option<bool>, // Some(reverse_second) when interleaving 2 docs
    output_path: &Path,
) -> Result<usize, String> {
    let mut max_id = 1;
    let mut per_doc_pages: Vec<Vec<(ObjectId, Object)>> = Vec::new();
    let mut documents_objects = BTreeMap::new();
    let mut merged = Document::with_version("1.5");

    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    for mut doc in documents.drain(..) {
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;

        let mut doc_pages = Vec::new();
        for (_, object_id) in doc.get_pages() {
            if let Ok(page_obj) = doc.get_object(object_id) {
                doc_pages.push((object_id, page_obj.to_owned()));
            }
        }
        per_doc_pages.push(doc_pages);
        documents_objects.extend(doc.objects);
    }

    let ordered_pages: Vec<(ObjectId, Object)> = if let Some(reverse_second) = interleave_two {
        let mut iter_list = per_doc_pages.into_iter();
        let first = iter_list.next().unwrap_or_default();
        let mut second = iter_list.next().unwrap_or_default();
        if reverse_second {
            second.reverse();
        }
        let mut interleaved = Vec::with_capacity(first.len() + second.len());
        let mut it1 = first.into_iter();
        let mut it2 = second.into_iter();
        loop {
            match (it1.next(), it2.next()) {
                (Some(p1), Some(p2)) => {
                    interleaved.push(p1);
                    interleaved.push(p2);
                }
                (Some(p1), None) => interleaved.push(p1),
                (None, Some(p2)) => interleaved.push(p2),
                (None, None) => break,
            }
        }
        interleaved
    } else {
        per_doc_pages.into_iter().flatten().collect()
    };

    if ordered_pages.is_empty() {
        return Err("Keine Seiten zum Zusammenführen gefunden".to_string());
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
                    let chosen_id = if let Some((id, _)) = pages_object {
                        id
                    } else {
                        object_id
                    };
                    pages_object = Some((chosen_id, Object::Dictionary(dict)));
                }
            }
            b"Page" | b"Outlines" | b"Outline" => {}
            _ => {
                merged.objects.insert(object_id, object);
            }
        }
    }

    let (pages_id, pages_obj) =
        pages_object.ok_or_else(|| "Keine Seitenstruktur gefunden".to_string())?;
    let (catalog_id, catalog_obj) =
        catalog_object.ok_or_else(|| "Kein Katalog in PDFs gefunden".to_string())?;

    let mut kids = Vec::with_capacity(ordered_pages.len());
    for (object_id, object) in &ordered_pages {
        if let Ok(dict) = object.as_dict() {
            let mut dict = dict.clone();
            dict.set("Parent", pages_id);
            merged.objects.insert(*object_id, Object::Dictionary(dict));
            kids.push(Object::Reference(*object_id));
        }
    }

    let total_pages = kids.len();

    if let Ok(dict) = pages_obj.as_dict() {
        let mut dict = dict.clone();
        dict.set("Count", total_pages as u32);
        dict.set("Kids", kids);
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

    merged
        .save(output_path)
        .map_err(|e| format!("Fehler beim Speichern der zusammengeführten PDF: {e}"))?;

    Ok(total_pages)
}

pub fn merge_pdfs(input_paths: &[PathBuf], output_path: &Path) -> Result<usize, String> {
    if input_paths.len() < 2 {
        return Err("Mindestens 2 PDF-Dateien zum Zusammenführen erforderlich".to_string());
    }

    let mut documents = Vec::with_capacity(input_paths.len());
    for p in input_paths {
        let doc = Document::load(p)
            .map_err(|e| format!("Fehler beim Laden von '{}': {e}", p.display()))?;
        documents.push(doc);
    }

    combine_documents_ordered(documents, None, output_path)
}

/// Interleaves pages from two PDFs (e.g. front pages 1,3,5 and back pages 2,4,6 for duplex scanning).
pub fn interleave_pdfs(
    front_pdf: &Path,
    back_pdf: &Path,
    reverse_back: bool,
    output_path: &Path,
) -> Result<usize, String> {
    let doc1 = Document::load(front_pdf)
        .map_err(|e| format!("Fehler beim Laden von Vorderseiten '{}': {e}", front_pdf.display()))?;
    let doc2 = Document::load(back_pdf)
        .map_err(|e| format!("Fehler beim Laden von Rückseiten '{}': {e}", back_pdf.display()))?;

    combine_documents_ordered(vec![doc1, doc2], Some(reverse_back), output_path)
}
