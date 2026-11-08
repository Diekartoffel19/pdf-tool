use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream};
use pdf_toolkit_rust::pdf_ops::*;
use std::fs;
use std::path::PathBuf;

fn create_sample_pdf(text: &str, pages: usize) -> Document {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let mut page_ids = Vec::new();

    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });

    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! {
            "F1" => font_id,
        },
    });

    for i in 1..=pages {
        let content = Content {
            operations: vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F1".into(), 24.into()]),
                Operation::new("Td", vec![100.into(), 700.into()]),
                Operation::new("Tj", vec![Object::string_literal(format!("{text} - Seite {i}"))]),
                Operation::new("ET", vec![]),
            ],
        };

        let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
        let page = dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        };
        let page_id = doc.add_object(page);
        page_ids.push(page_id);
    }

    let pages_dict = dictionary! {
        "Type" => "Pages",
        "Kids" => page_ids.into_iter().map(Object::Reference).collect::<Vec<_>>(),
        "Count" => pages as i64,
    };
    doc.objects.insert(pages_id, Object::Dictionary(pages_dict));

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);

    doc.max_id = doc.objects.len() as u32;
    doc.renumber_objects();
    doc
}

#[test]
fn test_pdf_toolkit_pipeline() {
    let test_dir = PathBuf::from("target/test_output");
    let _ = fs::create_dir_all(&test_dir);

    // 1. Create two test PDFs
    let mut doc1 = create_sample_pdf("Doc1", 3);
    let path1 = test_dir.join("doc1.pdf");
    doc1.save(&path1).unwrap();

    let mut doc2 = create_sample_pdf("Doc2", 2);
    let path2 = test_dir.join("doc2.pdf");
    doc2.save(&path2).unwrap();

    // 2. Test Merge
    let merged_path = test_dir.join("merged.pdf");
    let merged_count = merge_pdfs(&[path1.clone(), path2.clone()], &merged_path).unwrap();
    assert_eq!(merged_count, 5);

    // 3. Test Details
    let details = get_pdf_details(&merged_path).unwrap();
    assert_eq!(details.page_count, 5);

    // 4. Test Split / Extract
    let extract_path = test_dir.join("extracted.pdf");
    let extracted_count = extract_pages(&merged_path, &[1, 3, 5], &extract_path).unwrap();
    assert_eq!(extracted_count, 3);
    assert_eq!(get_pdf_details(&extract_path).unwrap().page_count, 3);

    // 5. Test Rotate
    let rotated_path = test_dir.join("rotated.pdf");
    let rotated_count = rotate_pdf(&extract_path, Some(&[1]), 90, &rotated_path).unwrap();
    assert_eq!(rotated_count, 1);

    // 6. Test Delete
    let deleted_path = test_dir.join("deleted.pdf");
    let remaining = delete_pages_from_pdf(&extract_path, &[2], &deleted_path).unwrap();
    assert_eq!(remaining, 2);

    // 7. Test Compress
    let compressed_path = test_dir.join("compressed.pdf");
    let comp_res = compress_pdf(&merged_path, &compressed_path, true).unwrap();
    assert!(comp_res.compressed_size > 0);

    // 8. Test Metadata
    let mut meta = PdfMetadata::default();
    meta.title = "Rust PDF Meister".to_string();
    meta.author = "Antigravity".to_string();
    let meta_path = test_dir.join("meta.pdf");
    write_metadata(&merged_path, &meta, &meta_path).unwrap();
    let read_back = read_metadata(&meta_path).unwrap();
    assert_eq!(read_back.title, "Rust PDF Meister");
    assert_eq!(read_back.author, "Antigravity");

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}
