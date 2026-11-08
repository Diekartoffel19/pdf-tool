use crate::pdf_ops::watermark::get_page_dimensions;
use lopdf::{dictionary, Document, Object, Stream};
use std::path::Path;

const MM_TO_PT: f32 = 2.834_645_7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPageFormat {
    A4Portrait,
    A4Landscape,
    A3Portrait,
    A5Portrait,
    LetterPortrait,
}

impl TargetPageFormat {
    pub fn dimensions_pt(self) -> (f32, f32) {
        match self {
            Self::A4Portrait => (595.28, 841.89),
            Self::A4Landscape => (841.89, 595.28),
            Self::A3Portrait => (841.89, 1190.55),
            Self::A5Portrait => (419.53, 595.28),
            Self::LetterPortrait => (612.0, 792.0),
        }
    }
}

pub fn crop_margins_mm(
    input_path: &Path,
    top_mm: f32,
    bottom_mm: f32,
    left_mm: f32,
    right_mm: f32,
    target_pages: Option<&[u32]>,
    output_path: &Path,
) -> Result<usize, String> {
    if top_mm < 0.0 || bottom_mm < 0.0 || left_mm < 0.0 || right_mm < 0.0 {
        return Err("Ränder dürfen nicht negativ sein".to_string());
    }

    let mut doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden von '{}': {e}", input_path.display()))?;

    let top_pt = top_mm * MM_TO_PT;
    let bottom_pt = bottom_mm * MM_TO_PT;
    let left_pt = left_mm * MM_TO_PT;
    let right_pt = right_mm * MM_TO_PT;

    let pages = doc.get_pages();
    let mut modified = 0;

    for (page_num, page_id) in pages {
        if let Some(filter) = target_pages {
            if !filter.contains(&page_num) {
                continue;
            }
        }

        let (width, height) = get_page_dimensions(&doc, page_id);
        let new_x0 = left_pt;
        let new_y0 = bottom_pt;
        let new_x1 = width - right_pt;
        let new_y1 = height - top_pt;

        if new_x1 - new_x0 < 20.0 || new_y1 - new_y0 < 20.0 {
            return Err(format!(
                "Zuschnitt auf Seite {page_num} zu groß — verbleibender Bereich wäre zu klein"
            ));
        }

        let box_arr = vec![
            Object::Real(new_x0),
            Object::Real(new_y0),
            Object::Real(new_x1),
            Object::Real(new_y1),
        ];

        if let Ok(pdict) = doc.get_object_mut(page_id).and_then(|o| o.as_dict_mut()) {
            pdict.set("MediaBox", box_arr.clone());
            pdict.set("CropBox", box_arr);
            modified += 1;
        }
    }

    doc.save(output_path)
        .map_err(|e| format!("Fehler beim Speichern der zugeschnittenen PDF: {e}"))?;

    Ok(modified)
}

pub fn scale_pages_to_format(
    input_path: &Path,
    format: TargetPageFormat,
    target_pages: Option<&[u32]>,
    output_path: &Path,
) -> Result<usize, String> {
    let mut doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden von '{}': {e}", input_path.display()))?;

    let (target_w, target_h) = format.dimensions_pt();
    let pages = doc.get_pages();
    let mut modified = 0;

    for (page_num, page_id) in pages {
        if let Some(filter) = target_pages {
            if !filter.contains(&page_num) {
                continue;
            }
        }

        let (old_w, old_h) = get_page_dimensions(&doc, page_id);
        let sx = target_w / old_w.max(1.0);
        let sy = target_h / old_h.max(1.0);
        let scale = sx.min(sy);

        let dx = (target_w - old_w * scale) / 2.0;
        let dy = (target_h - old_h * scale) / 2.0;

        let prefix_cmd = format!("q {scale:.5} 0 0 {scale:.5} {dx:.2} {dy:.2} cm\n");
        let prefix_id = doc.add_object(Stream::new(dictionary! {}, prefix_cmd.into_bytes()));
        let suffix_id = doc.add_object(Stream::new(dictionary! {}, b"\nQ\n".to_vec()));

        let existing_contents = doc
            .get_object(page_id)
            .and_then(|o| o.as_dict())
            .ok()
            .and_then(|d| d.get(b"Contents").ok().cloned());

        let mut new_contents = vec![Object::Reference(prefix_id)];
        match existing_contents {
            Some(Object::Reference(r)) => new_contents.push(Object::Reference(r)),
            Some(Object::Array(arr)) => new_contents.extend(arr),
            Some(Object::Stream(s)) => {
                let sid = doc.add_object(Object::Stream(s));
                new_contents.push(Object::Reference(sid));
            }
            _ => {}
        }
        new_contents.push(Object::Reference(suffix_id));

        let target_box = vec![
            0.into(),
            0.into(),
            Object::Real(target_w),
            Object::Real(target_h),
        ];

        if let Ok(pdict) = doc.get_object_mut(page_id).and_then(|o| o.as_dict_mut()) {
            pdict.set("MediaBox", target_box.clone());
            pdict.set("CropBox", target_box);
            pdict.set("Contents", Object::Array(new_contents));
            modified += 1;
        }
    }

    doc.save(output_path)
        .map_err(|e| format!("Fehler beim Speichern der skalierten PDF: {e}"))?;

    Ok(modified)
}
