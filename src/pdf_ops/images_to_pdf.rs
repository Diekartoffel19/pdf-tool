use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImagePageLayout {
    OriginalSize,
    A4Portrait,
    A4Landscape,
}

pub fn convert_images_to_pdf(
    image_paths: &[PathBuf],
    layout: ImagePageLayout,
    margin_pt: f32,
    output_path: &Path,
) -> Result<usize, String> {
    if image_paths.is_empty() {
        return Err("Mindestens ein Bild erforderlich".to_string());
    }

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let mut page_ids = Vec::new();

    for img_path in image_paths {
        let img = image::open(img_path)
            .map_err(|e| format!("Fehler beim Öffnen von '{}': {e}", img_path.display()))?;

        let img_w = img.width().max(1) as f32;
        let img_h = img.height().max(1) as f32;

        let (page_w, page_h, draw_w, draw_h, offset_x, offset_y) = match layout {
            ImagePageLayout::OriginalSize => (img_w, img_h, img_w, img_h, 0.0, 0.0),
            ImagePageLayout::A4Portrait | ImagePageLayout::A4Landscape => {
                let (pw, ph) = if layout == ImagePageLayout::A4Portrait {
                    (595.28_f32, 841.89_f32)
                } else {
                    (841.89_f32, 595.28_f32)
                };
                let m = margin_pt.clamp(0.0, 120.0);
                let avail_w = (pw - 2.0 * m).max(50.0);
                let avail_h = (ph - 2.0 * m).max(50.0);
                let scale = (avail_w / img_w).min(avail_h / img_h);
                let dw = img_w * scale;
                let dh = img_h * scale;
                let ox = (pw - dw) / 2.0;
                let oy = (ph - dh) / 2.0;
                (pw, ph, dw, dh, ox, oy)
            }
        };

        let image_stream = lopdf::xobject::image(img_path)
            .map_err(|e| format!("Fehler beim Einbetten des Bildes '{}': {e}", img_path.display()))?;

        let image_id = doc.add_object(image_stream);

        let content = Content {
            operations: vec![
                Operation::new("q", vec![]),
                Operation::new(
                    "cm",
                    vec![
                        Object::Real(draw_w),
                        0.into(),
                        0.into(),
                        Object::Real(draw_h),
                        Object::Real(offset_x),
                        Object::Real(offset_y),
                    ],
                ),
                Operation::new("Do", vec![Object::Name(b"Im0".to_vec())]),
                Operation::new("Q", vec![]),
            ],
        };

        let content_encoded = content
            .encode()
            .map_err(|e| format!("Fehler beim Codieren des Seiteninhalts: {e}"))?;

        let content_id = doc.add_object(Stream::new(dictionary! {}, content_encoded));

        let resources = dictionary! {
            "XObject" => dictionary! {
                "Im0" => image_id,
            },
        };
        let resources_id = doc.add_object(resources);

        let page = dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![
                0.into(),
                0.into(),
                Object::Real(page_w),
                Object::Real(page_h),
            ],
            "Resources" => resources_id,
            "Contents" => content_id,
        };
        let page_id = doc.add_object(page);
        page_ids.push(page_id);
    }

    let pages = dictionary! {
        "Type" => "Pages",
        "Kids" => page_ids.into_iter().map(Object::Reference).collect::<Vec<_>>(),
        "Count" => image_paths.len() as i64,
    };
    doc.objects.insert(pages_id, Object::Dictionary(pages));

    let catalog = dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    };
    let catalog_id = doc.add_object(catalog);
    doc.trailer.set("Root", catalog_id);

    doc.max_id = doc.objects.len() as u32;
    doc.renumber_objects();
    doc.compress();

    doc.save(output_path)
        .map_err(|e| format!("Fehler beim Speichern der PDF: {e}"))?;

    Ok(image_paths.len())
}
