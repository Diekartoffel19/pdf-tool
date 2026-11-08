use lopdf::{Document, Object, Stream, Dictionary};
use std::collections::HashSet;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;

pub fn merge(files: Vec<Vec<u8>>) -> Result<Vec<u8>, String> {
    if files.is_empty() {
        return Err("Keine Dateien zum Zusammenfügen übergeben".into());
    }

    let mut docs = Vec::new();
    for f in &files {
        let doc = Document::load_mem(f).map_err(|e| format!("Fehler beim Laden von PDF: {}", e))?;
        docs.push(doc);
    }

    let mut merged = docs.remove(0);
    merged.renumber_objects_with(1);

    for mut next_doc in docs {
        let max_id = merged.max_id;
        next_doc.renumber_objects_with(max_id + 1);

        let next_pages = next_doc.get_pages();
        let mut page_ids = Vec::new();
        for (_pno, id) in next_pages {
            page_ids.push(id);
        }

        for (id, obj) in next_doc.objects {
            merged.objects.insert(id, obj);
        }

        let cat_pages_id = merged.catalog()
            .map_err(|e| e.to_string())?
            .get(b"Pages")
            .map_err(|e| e.to_string())?
            .as_reference()
            .map_err(|e| e.to_string())?;

        for pid in &page_ids {
            if let Ok(pdict) = merged.get_object_mut(*pid).and_then(|o| o.as_dict_mut()) {
                pdict.set("Parent", Object::Reference(cat_pages_id));
            }
        }

        let pages_dict = merged.get_object_mut(cat_pages_id)
            .map_err(|e| e.to_string())?
            .as_dict_mut()
            .map_err(|e| e.to_string())?;

        let kids = pages_dict.get_mut(b"Kids")
            .map_err(|e| e.to_string())?
            .as_array_mut()
            .map_err(|e| e.to_string())?;

        for pid in page_ids {
            kids.push(Object::Reference(pid));
        }

        let count = kids.len() as i64;
        pages_dict.set("Count", Object::Integer(count));
    }

    let mut out = Vec::new();
    merged.save_to(&mut out).map_err(|e| format!("Fehler beim Speichern: {}", e))?;
    Ok(out)
}

pub fn parse_range(range_str: &str, total_pages: u32) -> Vec<u32> {
    let mut selected = Vec::new();
    for part in range_str.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((start_s, end_s)) = part.split_once('-') {
            if let (Ok(s), Ok(e)) = (start_s.trim().parse::<u32>(), end_s.trim().parse::<u32>()) {
                let from = s.max(1);
                let to = e.min(total_pages);
                for p in from..=to {
                    if !selected.contains(&p) {
                        selected.push(p);
                    }
                }
            }
        } else if let Ok(p) = part.parse::<u32>() {
            if p >= 1 && p <= total_pages && !selected.contains(&p) {
                selected.push(p);
            }
        }
    }
    selected
}

pub fn split(data: &[u8], mode: &str, range_str: &str) -> Result<(Vec<u8>, String), String> {
    let doc = Document::load_mem(data).map_err(|e| format!("Fehler beim Laden: {}", e))?;
    let pages = doc.get_pages();
    let total = pages.len() as u32;

    if total == 0 {
        return Err("PDF enthält keine Seiten".into());
    }

    if mode == "single" {
        let mut zip_buf = Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut zip_buf);
            let opts = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);

            for p in 1..=total {
                let mut single_doc = Document::load_mem(data).map_err(|e| e.to_string())?;
                let to_delete: Vec<u32> = (1..=total).filter(|&x| x != p).collect();
                single_doc.delete_pages(&to_delete);
                let mut out_bytes = Vec::new();
                single_doc.save_to(&mut out_bytes).map_err(|e| e.to_string())?;

                zip.start_file(format!("seite_{:03}.pdf", p), opts)
                    .map_err(|e| e.to_string())?;
                zip.write_all(&out_bytes).map_err(|e| e.to_string())?;
            }
            zip.finish().map_err(|e| e.to_string())?;
        }
        return Ok((zip_buf.into_inner(), "seiten_geteilt.zip".into()));
    }

    let selected = parse_range(range_str, total);
    if selected.is_empty() {
        return Err("Keine gültigen Seiten angegeben (z. B. 1-3, 5)".into());
    }

    let mut out_doc = Document::load_mem(data).map_err(|e| e.to_string())?;
    let to_delete: Vec<u32> = (1..=total).filter(|p| !selected.contains(p)).collect();
    out_doc.delete_pages(&to_delete);

    let mut out_bytes = Vec::new();
    out_doc.save_to(&mut out_bytes).map_err(|e| e.to_string())?;
    Ok((out_bytes, "extrahierte_seiten.pdf".into()))
}

pub fn organize(
    data: &[u8],
    ordered_indices: &[u32],
    rotations: &[(u32, i64)]
) -> Result<Vec<u8>, String> {
    let mut doc = Document::load_mem(data).map_err(|e| format!("Fehler beim Laden: {}", e))?;
    let orig_pages = doc.get_pages();
    let total = orig_pages.len() as u32;

    if ordered_indices.is_empty() {
        return Err("Mindestens eine Seite muss beibehalten werden".into());
    }

    // Apply rotations
    for &(p, rot) in rotations {
        if let Some(&page_id) = orig_pages.get(&p) {
            if let Ok(pdict) = doc.get_object_mut(page_id).and_then(|o| o.as_dict_mut()) {
                pdict.set("Rotate", Object::Integer(rot % 360));
            }
        }
    }

    // Delete omitted pages
    let keep_set: HashSet<u32> = ordered_indices.iter().cloned().collect();
    let to_delete: Vec<u32> = (1..=total).filter(|p| !keep_set.contains(p)).collect();
    if !to_delete.is_empty() {
        doc.delete_pages(&to_delete);
    }

    // Reorder kids in Pages catalog
    let cat_pages_id = doc.catalog()
        .map_err(|e| e.to_string())?
        .get(b"Pages")
        .map_err(|e| e.to_string())?
        .as_reference()
        .map_err(|e| e.to_string())?;

    let mut new_kids = Vec::new();
    for &p in ordered_indices {
        // Find matching page id in updated_pages
        if let Some(&id) = orig_pages.get(&p) {
            new_kids.push(Object::Reference(id));
        }
    }

    if !new_kids.is_empty() {
        let pages_dict = doc.get_object_mut(cat_pages_id)
            .map_err(|e| e.to_string())?
            .as_dict_mut()
            .map_err(|e| e.to_string())?;
        pages_dict.set("Kids", Object::Array(new_kids.clone()));
        pages_dict.set("Count", Object::Integer(new_kids.len() as i64));
    }

    let mut out = Vec::new();
    doc.save_to(&mut out).map_err(|e| e.to_string())?;
    Ok(out)
}

pub fn compress(data: &[u8]) -> Result<(Vec<u8>, usize, usize), String> {
    let orig_len = data.len();
    let mut doc = Document::load_mem(data).map_err(|e| format!("Fehler beim Laden: {}", e))?;
    
    doc.compress();
    doc.prune_objects();
    doc.change_producer("PDF Toolkit Rust");

    let mut out = Vec::new();
    doc.save_to(&mut out).map_err(|e| e.to_string())?;
    let new_len = out.len();

    Ok((out, orig_len, new_len))
}

pub fn extract_text(data: &[u8]) -> Result<String, String> {
    let doc = Document::load_mem(data).map_err(|e| format!("Fehler beim Laden: {}", e))?;
    let pages = doc.get_pages();
    let mut out_text = String::new();

    for (pno, _id) in pages {
        if let Ok(text) = doc.extract_text(&[pno]) {
            out_text.push_str(&format!("=== SEITE {} ===\n\n{}\n\n", pno, text.trim()));
        }
    }

    if out_text.trim().is_empty() {
        out_text = "Kein lesbarer Text in dieser PDF-Datei gefunden.".into();
    }

    Ok(out_text)
}

pub fn watermark(data: &[u8], text: &str) -> Result<Vec<u8>, String> {
    let mut doc = Document::load_mem(data).map_err(|e| format!("Fehler beim Laden: {}", e))?;
    let pages = doc.get_pages();
    let safe_text = text.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)");

    // Clean semi-transparent watermark stream using Type 1 font Helvetica
    let wm_stream_content = format!(
        "q\n0.6 0.6 0.6 rg\n0.6 0.6 0.6 RG\nBT\n/Helvetica 42 Tf\n1 0 0 1 150 400 cm\n0.7071 0.7071 -0.7071 0.7071 0 0 cm\n({}) Tj\nET\nQ\n",
        safe_text
    );

    for (_pno, page_id) in pages {
        let stream = Stream::new(Dictionary::new(), wm_stream_content.as_bytes().to_vec());
        let stream_id = doc.add_object(stream);

        if let Ok(pdict) = doc.get_object_mut(page_id).and_then(|o| o.as_dict_mut()) {
            match pdict.get_mut(b"Contents") {
                Ok(Object::Array(ref mut arr)) => {
                    arr.push(Object::Reference(stream_id));
                }
                Ok(Object::Reference(orig_ref)) => {
                    let old_id = *orig_ref;
                    pdict.set("Contents", Object::Array(vec![Object::Reference(old_id), Object::Reference(stream_id)]));
                }
                _ => {
                    pdict.set("Contents", Object::Array(vec![Object::Reference(stream_id)]));
                }
            }
        }
    }

    let mut out = Vec::new();
    doc.save_to(&mut out).map_err(|e| e.to_string())?;
    Ok(out)
}

pub fn page_numbers(data: &[u8], fmt: &str, pos: &str) -> Result<Vec<u8>, String> {
    let mut doc = Document::load_mem(data).map_err(|e| format!("Fehler beim Laden: {}", e))?;
    let pages = doc.get_pages();
    let total = pages.len();

    let (x, y) = match pos {
        "bottom-right" => (480, 25),
        "bottom-left" => (40, 25),
        "top-center" => (270, 800),
        "top-right" => (480, 800),
        _ => (270, 25), // bottom-center default
    };

    for (pno, page_id) in &pages {
        let label = match fmt {
            "x_von_y" => format!("{} / {}", pno, total),
            "x" => format!("{}", pno),
            _ => format!("Seite {} von {}", pno, total),
        };

        let num_stream = format!(
            "q\n0.3 0.3 0.3 rg\nBT\n/Helvetica 10 Tf\n1 0 0 1 {} {} cm\n({}) Tj\nET\nQ\n",
            x, y, label
        );

        let stream = Stream::new(Dictionary::new(), num_stream.as_bytes().to_vec());
        let stream_id = doc.add_object(stream);

        if let Ok(pdict) = doc.get_object_mut(*page_id).and_then(|o| o.as_dict_mut()) {
            match pdict.get_mut(b"Contents") {
                Ok(Object::Array(ref mut arr)) => {
                    arr.push(Object::Reference(stream_id));
                }
                Ok(Object::Reference(orig_ref)) => {
                    let old_id = *orig_ref;
                    pdict.set("Contents", Object::Array(vec![Object::Reference(old_id), Object::Reference(stream_id)]));
                }
                _ => {
                    pdict.set("Contents", Object::Array(vec![Object::Reference(stream_id)]));
                }
            }
        }
    }

    let mut out = Vec::new();
    doc.save_to(&mut out).map_err(|e| e.to_string())?;
    Ok(out)
}

pub fn images_to_pdf(images: Vec<Vec<u8>>) -> Result<Vec<u8>, String> {
    if images.is_empty() {
        return Err("Keine Bilder übergeben".into());
    }

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let mut page_ids = Vec::new();

    // Standard A4: 595 x 842 points
    for img_bytes in images {
        // Create an Image XObject (JPEG support via /DCTDecode)
        let mut img_dict = Dictionary::new();
        img_dict.set("Type", Object::Name(b"XObject".to_vec()));
        img_dict.set("Subtype", Object::Name(b"Image".to_vec()));
        img_dict.set("Width", Object::Integer(595));
        img_dict.set("Height", Object::Integer(842));
        img_dict.set("ColorSpace", Object::Name(b"DeviceRGB".to_vec()));
        img_dict.set("BitsPerComponent", Object::Integer(8));
        img_dict.set("Filter", Object::Name(b"DCTDecode".to_vec()));

        let img_stream = Stream::new(img_dict, img_bytes);
        let img_id = doc.add_object(img_stream);

        // Content stream: draws image scaled to A4 page
        let content_str = "q\n595 0 0 842 0 0 cm\n/Im1 Do\nQ\n";
        let content_stream = Stream::new(Dictionary::new(), content_str.as_bytes().to_vec());
        let content_id = doc.add_object(content_stream);

        // Resources dictionary with XObject mapping
        let mut xobj_dict = Dictionary::new();
        xobj_dict.set("Im1", Object::Reference(img_id));
        let mut res_dict = Dictionary::new();
        res_dict.set("XObject", Object::Dictionary(xobj_dict));

        // Page dictionary
        let mut page_dict = Dictionary::new();
        page_dict.set("Type", Object::Name(b"Page".to_vec()));
        page_dict.set("Parent", Object::Reference(pages_id));
        page_dict.set("MediaBox", Object::Array(vec![
            Object::Integer(0),
            Object::Integer(0),
            Object::Integer(595),
            Object::Integer(842),
        ]));
        page_dict.set("Contents", Object::Reference(content_id));
        page_dict.set("Resources", Object::Dictionary(res_dict));

        let pid = doc.add_object(page_dict);
        page_ids.push(Object::Reference(pid));
    }

    // Pages dictionary
    let mut pages_dict = Dictionary::new();
    pages_dict.set("Type", Object::Name(b"Pages".to_vec()));
    pages_dict.set("Count", Object::Integer(page_ids.len() as i64));
    pages_dict.set("Kids", Object::Array(page_ids));
    doc.objects.insert(pages_id, Object::Dictionary(pages_dict));

    // Catalog dictionary
    let mut catalog_dict = Dictionary::new();
    catalog_dict.set("Type", Object::Name(b"Catalog".to_vec()));
    catalog_dict.set("Pages", Object::Reference(pages_id));
    let cat_id = doc.add_object(catalog_dict);
    doc.trailer.set("Root", Object::Reference(cat_id));

    let mut out = Vec::new();
    doc.save_to(&mut out).map_err(|e| e.to_string())?;
    Ok(out)
}
