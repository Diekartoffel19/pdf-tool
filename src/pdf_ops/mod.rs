pub mod compress;
pub mod crop_scale;
pub mod delete;
pub mod extract_images;
pub mod images_to_pdf;
pub mod merge;
pub mod metadata;
pub mod organize;
pub mod page_numbers;
pub mod page_range;
pub mod rotate;
pub mod sanitize;
pub mod security;
pub mod split;
pub mod text_info;
pub mod watermark;

pub use compress::{compress_pdf, CompressResult};
pub use crop_scale::{crop_margins_mm, scale_pages_to_format, TargetPageFormat};
pub use delete::delete_pages_from_pdf;
pub use extract_images::extract_images_from_pdf;
pub use images_to_pdf::{convert_images_to_pdf, ImagePageLayout};
pub use merge::{interleave_pdfs, merge_pdfs};
pub use metadata::{read_metadata, write_metadata, PdfMetadata};
pub use organize::{insert_blank_pages, reorder_pages, reverse_pages};
pub use page_numbers::{add_page_numbers, PageNumberFormat, PageNumberOptions, PageNumberPosition};
pub use page_range::{parse_page_ranges, parse_page_sequence};
pub use rotate::rotate_pdf;
pub use sanitize::{sanitize_pdf, SanitizeOptions, SanitizeReport};
pub use security::{decrypt_pdf, encrypt_pdf, SecurityPermissions};
pub use split::{extract_pages, split_all_pages, split_by_chunks};
pub use text_info::{extract_text, get_pdf_details, PdfDetails};
pub use watermark::{add_watermark, WatermarkOptions, WatermarkPosition};

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::content::{Content, Operation};
    use lopdf::{dictionary, Document, Object, Stream};
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
    fn test_page_ranges_and_sequences() {
        assert_eq!(parse_page_ranges("1-3, 5, 7-8", 10).unwrap(), vec![1, 2, 3, 5, 7, 8]);
        assert_eq!(parse_page_ranges("alle", 4).unwrap(), vec![1, 2, 3, 4]);
        assert_eq!(parse_page_ranges("gerade", 5).unwrap(), vec![2, 4]);
        assert_eq!(parse_page_ranges("ungerade", 5).unwrap(), vec![1, 3, 5]);
        assert_eq!(parse_page_ranges("3-", 5).unwrap(), vec![3, 4, 5]);
        assert_eq!(parse_page_ranges("-3", 5).unwrap(), vec![1, 2, 3]);
        assert_eq!(parse_page_sequence("3, 1, 4-2, 1", 4).unwrap(), vec![3, 1, 4, 3, 2, 1]);
    }

    #[test]
    fn test_full_pdf_toolkit_pipeline() {
        let test_dir = PathBuf::from("target/test_output_full");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let mut doc1 = create_sample_pdf("Doc1", 3);
        let path1 = test_dir.join("doc1.pdf");
        doc1.save(&path1).unwrap();

        let mut doc2 = create_sample_pdf("Doc2", 2);
        let path2 = test_dir.join("doc2.pdf");
        doc2.save(&path2).unwrap();

        // Merge & Interleave
        let merged_path = test_dir.join("merged.pdf");
        assert_eq!(merge_pdfs(&[path1.clone(), path2.clone()], &merged_path).unwrap(), 5);

        let interleaved_path = test_dir.join("interleaved.pdf");
        assert_eq!(interleave_pdfs(&path1, &path2, true, &interleaved_path).unwrap(), 5);

        // Details
        let details = get_pdf_details(&merged_path).unwrap();
        assert_eq!(details.page_count, 5);
        assert!(details.page_format_label.contains("DIN A4"));

        // Split & Chunks
        let extract_path = test_dir.join("extracted.pdf");
        assert_eq!(extract_pages(&merged_path, &[1, 3, 5], &extract_path).unwrap(), 3);

        let chunks_dir = test_dir.join("chunks");
        fs::create_dir_all(&chunks_dir).unwrap();
        assert_eq!(split_by_chunks(&merged_path, 2, &chunks_dir).unwrap(), 3);

        // Organize: Reorder, Reverse, Insert Blank
        let reordered_path = test_dir.join("reordered.pdf");
        assert_eq!(reorder_pages(&extract_path, &[3, 1, 2, 1], &reordered_path).unwrap(), 4);

        let reversed_path = test_dir.join("reversed.pdf");
        assert_eq!(reverse_pages(&extract_path, &reversed_path).unwrap(), 3);

        let blank_path = test_dir.join("with_blank.pdf");
        assert_eq!(insert_blank_pages(&extract_path, 1, 2, 595.0, 842.0, &blank_path).unwrap(), 5);

        // Watermark & Page Numbers
        let wm_path = test_dir.join("watermarked.pdf");
        let wm_opts = WatermarkOptions::default();
        assert_eq!(add_watermark(&extract_path, &wm_opts, None, &wm_path).unwrap(), 3);

        let pn_path = test_dir.join("numbered.pdf");
        let pn_opts = PageNumberOptions::default();
        assert_eq!(add_page_numbers(&wm_path, &pn_opts, &pn_path).unwrap(), 3);

        // Crop & Scale
        let cropped_path = test_dir.join("cropped.pdf");
        assert_eq!(crop_margins_mm(&extract_path, 10.0, 10.0, 10.0, 10.0, None, &cropped_path).unwrap(), 3);

        let scaled_path = test_dir.join("scaled.pdf");
        assert_eq!(
            scale_pages_to_format(&cropped_path, TargetPageFormat::A4Landscape, None, &scaled_path).unwrap(),
            3
        );

        // Metadata & Sanitize
        let mut meta = PdfMetadata::default();
        meta.title = "Geheim".to_string();
        meta.author = "Tester".to_string();
        let meta_path = test_dir.join("meta.pdf");
        write_metadata(&merged_path, &meta, &meta_path).unwrap();
        assert_eq!(read_metadata(&meta_path).unwrap().title, "Geheim");

        let sanitized_path = test_dir.join("sanitized.pdf");
        let san_rep = sanitize_pdf(&meta_path, &SanitizeOptions::default(), &sanitized_path).unwrap();
        assert!(san_rep.metadata_cleared);
        assert_eq!(read_metadata(&sanitized_path).unwrap().title, "");

        // Rotate, Delete, Compress
        let rotated_path = test_dir.join("rotated.pdf");
        assert_eq!(rotate_pdf(&extract_path, Some(&[1]), 90, &rotated_path).unwrap(), 1);

        let deleted_path = test_dir.join("deleted.pdf");
        assert_eq!(delete_pages_from_pdf(&extract_path, &[2], &deleted_path).unwrap(), 2);

        let compressed_path = test_dir.join("compressed.pdf");
        assert!(compress_pdf(&merged_path, &compressed_path, true).unwrap().compressed_size > 0);

        let _ = fs::remove_dir_all(&test_dir);
    }
}
