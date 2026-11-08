use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream};
use std::path::{Path, PathBuf};

pub fn convert_images_to_pdf(image_paths: &[PathBuf], output_path: &Path) -> Result<usize, String> {
    if image_paths.is_empty() {
        return Err("Mindestens ein Bild erforderlich".to_string());
    }

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let mut page_ids = Vec::new();

    for img_path in image_paths {
        let img = image::open(img_path)
            .map_err(|e| format!("Fehler beim Öffnen von '{}': {e}", img_path.display()))?;

        let width = img.width() as f32;
        let height = img.height() as f32;

        let image_stream = lopdf::xobject::image(img_path)
            .map_err(|e| format!("Fehler beim Einbetten des Bildes '{}': {e}", img_path.display()))?;

        let image_id = doc.add_object(image_stream);

        let content = Content {
            operations: vec![
                Operation::new("q", vec![]),
                Operation::new(
                    "cm",
                    vec![
                        (width as i64).into(),
                        0.into(),
                        0.into(),
                        (height as i64).into(),
                        0.into(),
                        0.into(),
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
            "MediaBox" => vec![0.into(), 0.into(), (width as i64).into(), (height as i64).into()],
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
