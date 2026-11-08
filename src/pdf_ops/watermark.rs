use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, ObjectId, Stream};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatermarkPosition {
    DiagonalCenter,
    Center,
    TopCenter,
    BottomCenter,
}

#[derive(Debug, Clone)]
pub struct WatermarkOptions {
    pub text: String,
    pub font_size: f32,
    pub color_rgb: (f32, f32, f32),
    pub position: WatermarkPosition,
    pub underneath: bool,
}

impl Default for WatermarkOptions {
    fn default() -> Self {
        Self {
            text: "VERTRAULICH".to_string(),
            font_size: 54.0,
            color_rgb: (0.72, 0.72, 0.72),
            position: WatermarkPosition::DiagonalCenter,
            underneath: false,
        }
    }
}

pub(crate) fn encode_win_ansi(text: &str) -> Vec<u8> {
    text.chars()
        .map(|c| match c {
            '\u{0020}'..='\u{00FF}' => c as u8,
            '€' => 0x80,
            '–' => 0x96,
            '—' => 0x97,
            '„' => 0x84,
            '“' | '”' => 0x93,
            '•' => 0x95,
            _ => b'?',
        })
        .collect()
}

pub(crate) fn get_page_dimensions(doc: &Document, page_id: ObjectId) -> (f32, f32) {
    let obj_to_f32 = |o: &Object| -> Option<f32> {
        match o {
            Object::Integer(i) => Some(*i as f32),
            Object::Real(f) => Some(*f),
            _ => None,
        }
    };

    let parse_box = |arr: &[Object]| -> Option<(f32, f32)> {
        if arr.len() >= 4 {
            let x0 = obj_to_f32(&arr[0])?;
            let y0 = obj_to_f32(&arr[1])?;
            let x1 = obj_to_f32(&arr[2])?;
            let y1 = obj_to_f32(&arr[3])?;
            let w = (x1 - x0).abs();
            let h = (y1 - y0).abs();
            if w > 1.0 && h > 1.0 {
                return Some((w, h));
            }
        }
        None
    };

    let mut current_id = Some(page_id);
    let mut depth = 0;
    while let Some(id) = current_id {
        if depth > 6 {
            break;
        }
        depth += 1;
        if let Ok(dict) = doc.get_object(id).and_then(|o| o.as_dict()) {
            if let Ok(mb) = dict.get(b"MediaBox") {
                match mb {
                    Object::Array(arr) => {
                        if let Some(dim) = parse_box(arr) {
                            return dim;
                        }
                    }
                    Object::Reference(ref_id) => {
                        if let Ok(Object::Array(arr)) = doc.get_object(*ref_id) {
                            if let Some(dim) = parse_box(arr) {
                                return dim;
                            }
                        }
                    }
                    _ => {}
                }
            }
            current_id = dict.get(b"Parent").and_then(|p| p.as_reference()).ok();
        } else {
            break;
        }
    }

    (595.0, 842.0)
}

pub(crate) fn ensure_page_font(
    doc: &mut Document,
    page_id: ObjectId,
    font_key: &[u8],
    font_id: ObjectId,
) {
    let res_ref_or_inline = doc
        .get_object(page_id)
        .and_then(|o| o.as_dict())
        .ok()
        .and_then(|d| d.get(b"Resources").ok().cloned());

    match res_ref_or_inline {
        Some(Object::Reference(res_id)) => {
            let font_entry = doc
                .get_object(res_id)
                .and_then(|o| o.as_dict())
                .ok()
                .and_then(|d| d.get(b"Font").ok().cloned());

            match font_entry {
                Some(Object::Reference(font_dict_id)) => {
                    if let Ok(fdict) = doc.get_object_mut(font_dict_id).and_then(|o| o.as_dict_mut())
                    {
                        fdict.set(font_key.to_vec(), Object::Reference(font_id));
                    }
                }
                Some(Object::Dictionary(mut fdict)) => {
                    fdict.set(font_key.to_vec(), Object::Reference(font_id));
                    if let Ok(rdict) = doc.get_object_mut(res_id).and_then(|o| o.as_dict_mut()) {
                        rdict.set("Font", Object::Dictionary(fdict));
                    }
                }
                _ => {
                    let mut fdict = lopdf::Dictionary::new();
                    fdict.set(font_key.to_vec(), Object::Reference(font_id));
                    if let Ok(rdict) = doc.get_object_mut(res_id).and_then(|o| o.as_dict_mut()) {
                        rdict.set("Font", Object::Dictionary(fdict));
                    }
                }
            }
        }
        Some(Object::Dictionary(mut rdict)) => {
            match rdict.get(b"Font").ok().cloned() {
                Some(Object::Reference(font_dict_id)) => {
                    if let Ok(fdict) = doc.get_object_mut(font_dict_id).and_then(|o| o.as_dict_mut())
                    {
                        fdict.set(font_key.to_vec(), Object::Reference(font_id));
                    }
                }
                Some(Object::Dictionary(mut fdict)) => {
                    fdict.set(font_key.to_vec(), Object::Reference(font_id));
                    rdict.set("Font", Object::Dictionary(fdict));
                }
                _ => {
                    let mut fdict = lopdf::Dictionary::new();
                    fdict.set(font_key.to_vec(), Object::Reference(font_id));
                    rdict.set("Font", Object::Dictionary(fdict));
                }
            }
            if let Ok(pdict) = doc.get_object_mut(page_id).and_then(|o| o.as_dict_mut()) {
                pdict.set("Resources", Object::Dictionary(rdict));
            }
        }
        _ => {
            let mut fdict = lopdf::Dictionary::new();
            fdict.set(font_key.to_vec(), Object::Reference(font_id));
            let rdict = dictionary! {
                "Font" => Object::Dictionary(fdict),
            };
            if let Ok(pdict) = doc.get_object_mut(page_id).and_then(|o| o.as_dict_mut()) {
                pdict.set("Resources", Object::Dictionary(rdict));
            }
        }
    }
}

pub(crate) fn attach_overlay_stream(
    doc: &mut Document,
    page_id: ObjectId,
    overlay_stream_id: ObjectId,
    underneath: bool,
) {
    let q_save_id = doc.add_object(Stream::new(dictionary! {}, b"q\n".to_vec()));
    let q_restore_id = doc.add_object(Stream::new(dictionary! {}, b"Q\n".to_vec()));

    let existing_contents = doc
        .get_object(page_id)
        .and_then(|o| o.as_dict())
        .ok()
        .and_then(|d| d.get(b"Contents").ok().cloned());

    let mut existing_refs = Vec::new();
    match existing_contents {
        Some(Object::Reference(r)) => existing_refs.push(Object::Reference(r)),
        Some(Object::Array(arr)) => existing_refs.extend(arr),
        Some(Object::Stream(s)) => {
            let sid = doc.add_object(Object::Stream(s));
            existing_refs.push(Object::Reference(sid));
        }
        _ => {}
    }

    let new_contents = if existing_refs.is_empty() {
        vec![Object::Reference(overlay_stream_id)]
    } else if underneath {
        let mut v = vec![
            Object::Reference(overlay_stream_id),
            Object::Reference(q_save_id),
        ];
        v.extend(existing_refs);
        v.push(Object::Reference(q_restore_id));
        v
    } else {
        let mut v = vec![Object::Reference(q_save_id)];
        v.extend(existing_refs);
        v.push(Object::Reference(q_restore_id));
        v.push(Object::Reference(overlay_stream_id));
        v
    };

    if let Ok(pdict) = doc.get_object_mut(page_id).and_then(|o| o.as_dict_mut()) {
        pdict.set("Contents", Object::Array(new_contents));
    }
}

pub fn add_watermark(
    input_path: &Path,
    options: &WatermarkOptions,
    target_pages: Option<&[u32]>,
    output_path: &Path,
) -> Result<usize, String> {
    if options.text.trim().is_empty() {
        return Err("Wasserzeichen-Text darf nicht leer sein".to_string());
    }

    let mut doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden von '{}': {e}", input_path.display()))?;

    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica-Bold",
        "Encoding" => "WinAnsiEncoding",
    });

    let pages = doc.get_pages();
    let encoded_text = encode_win_ansi(options.text.trim());
    let char_count = options.text.trim().chars().count() as f32;
    let approx_text_width = char_count * options.font_size * 0.54;
    let (r, g, b) = options.color_rgb;

    let mut count = 0;

    for (page_num, page_id) in pages {
        if let Some(filter) = target_pages {
            if !filter.contains(&page_num) {
                continue;
            }
        }

        let (width, height) = get_page_dimensions(&doc, page_id);
        ensure_page_font(&mut doc, page_id, b"F_WM", font_id);

        let tm_args: Vec<Object> = match options.position {
            WatermarkPosition::DiagonalCenter => {
                let cos45 = 0.7071_f32;
                let sin45 = 0.7071_f32;
                let tx = (width / 2.0) - (approx_text_width * cos45 / 2.0) + (options.font_size * sin45 / 3.0);
                let ty = (height / 2.0) - (approx_text_width * sin45 / 2.0) - (options.font_size * cos45 / 3.0);
                vec![
                    Object::Real(cos45),
                    Object::Real(sin45),
                    Object::Real(-sin45),
                    Object::Real(cos45),
                    Object::Real(tx),
                    Object::Real(ty),
                ]
            }
            WatermarkPosition::Center => {
                let tx = ((width - approx_text_width) / 2.0).max(20.0);
                let ty = height / 2.0;
                vec![
                    1.into(),
                    0.into(),
                    0.into(),
                    1.into(),
                    Object::Real(tx),
                    Object::Real(ty),
                ]
            }
            WatermarkPosition::TopCenter => {
                let tx = ((width - approx_text_width) / 2.0).max(20.0);
                let ty = (height - options.font_size - 36.0).max(20.0);
                vec![
                    1.into(),
                    0.into(),
                    0.into(),
                    1.into(),
                    Object::Real(tx),
                    Object::Real(ty),
                ]
            }
            WatermarkPosition::BottomCenter => {
                let tx = ((width - approx_text_width) / 2.0).max(20.0);
                let ty = 36.0;
                vec![
                    1.into(),
                    0.into(),
                    0.into(),
                    1.into(),
                    Object::Real(tx),
                    Object::Real(ty),
                ]
            }
        };

        let content = Content {
            operations: vec![
                Operation::new("q", vec![]),
                Operation::new(
                    "rg",
                    vec![Object::Real(r), Object::Real(g), Object::Real(b)],
                ),
                Operation::new("BT", vec![]),
                Operation::new(
                    "Tf",
                    vec![
                        Object::Name(b"F_WM".to_vec()),
                        Object::Real(options.font_size),
                    ],
                ),
                Operation::new("Tm", tm_args),
                Operation::new(
                    "Tj",
                    vec![Object::String(encoded_text.clone(), lopdf::StringFormat::Literal)],
                ),
                Operation::new("ET", vec![]),
                Operation::new("Q", vec![]),
            ],
        };

        let bytes = content
            .encode()
            .map_err(|e| format!("Fehler beim Erzeugen des Wasserzeichen-Streams: {e}"))?;
        let stream_id = doc.add_object(Stream::new(dictionary! {}, bytes));
        attach_overlay_stream(&mut doc, page_id, stream_id, options.underneath);
        count += 1;
    }

    doc.save(output_path)
        .map_err(|e| format!("Fehler beim Speichern mit Wasserzeichen: {e}"))?;

    Ok(count)
}
