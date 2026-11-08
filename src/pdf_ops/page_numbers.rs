use crate::pdf_ops::watermark::{
    attach_overlay_stream, encode_win_ansi, ensure_page_font, get_page_dimensions,
};
use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageNumberFormat {
    Simple,      // "1"
    PageOfTotal, // "Seite 1 von 10"
    SlashTotal,  // "1 / 10"
    Bates,       // "PREFIX-0001"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageNumberPosition {
    BottomCenter,
    BottomRight,
    BottomLeft,
    TopCenter,
    TopRight,
    TopLeft,
}

#[derive(Debug, Clone)]
pub struct PageNumberOptions {
    pub format: PageNumberFormat,
    pub position: PageNumberPosition,
    pub start_number: u32,
    pub skip_first_page: bool,
    pub font_size: f32,
    pub margin_pt: f32,
    pub bates_prefix: String,
}

impl Default for PageNumberOptions {
    fn default() -> Self {
        Self {
            format: PageNumberFormat::PageOfTotal,
            position: PageNumberPosition::BottomCenter,
            start_number: 1,
            skip_first_page: false,
            font_size: 10.0,
            margin_pt: 28.0,
            bates_prefix: "DOK-".to_string(),
        }
    }
}

pub fn add_page_numbers(
    input_path: &Path,
    options: &PageNumberOptions,
    output_path: &Path,
) -> Result<usize, String> {
    let mut doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden von '{}': {e}", input_path.display()))?;

    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
        "Encoding" => "WinAnsiEncoding",
    });

    let pages = doc.get_pages();
    let total_pages = pages.len() as u32;
    let mut numbered_count = 0;

    for (page_num, page_id) in pages {
        if options.skip_first_page && page_num == 1 {
            continue;
        }

        let display_num = options.start_number + (page_num - 1);
        let display_total = options.start_number + total_pages - 1;

        let label = match options.format {
            PageNumberFormat::Simple => format!("{display_num}"),
            PageNumberFormat::PageOfTotal => format!("Seite {display_num} von {display_total}"),
            PageNumberFormat::SlashTotal => format!("{display_num} / {display_total}"),
            PageNumberFormat::Bates => format!("{}{display_num:04}", options.bates_prefix),
        };

        let (width, height) = get_page_dimensions(&doc, page_id);
        ensure_page_font(&mut doc, page_id, b"F_PN", font_id);

        let text_width = label.chars().count() as f32 * options.font_size * 0.52;
        let m = options.margin_pt.max(10.0);

        let (tx, ty) = match options.position {
            PageNumberPosition::BottomCenter => (((width - text_width) / 2.0).max(m), m),
            PageNumberPosition::BottomRight => ((width - text_width - m).max(m), m),
            PageNumberPosition::BottomLeft => (m, m),
            PageNumberPosition::TopCenter => (
                ((width - text_width) / 2.0).max(m),
                (height - options.font_size - m).max(m),
            ),
            PageNumberPosition::TopRight => (
                (width - text_width - m).max(m),
                (height - options.font_size - m).max(m),
            ),
            PageNumberPosition::TopLeft => (m, (height - options.font_size - m).max(m)),
        };

        let encoded = encode_win_ansi(&label);
        let content = Content {
            operations: vec![
                Operation::new("q", vec![]),
                Operation::new(
                    "rg",
                    vec![
                        Object::Real(0.2),
                        Object::Real(0.2),
                        Object::Real(0.2),
                    ],
                ),
                Operation::new("BT", vec![]),
                Operation::new(
                    "Tf",
                    vec![
                        Object::Name(b"F_PN".to_vec()),
                        Object::Real(options.font_size),
                    ],
                ),
                Operation::new("Td", vec![Object::Real(tx), Object::Real(ty)]),
                Operation::new(
                    "Tj",
                    vec![Object::String(encoded, lopdf::StringFormat::Literal)],
                ),
                Operation::new("ET", vec![]),
                Operation::new("Q", vec![]),
            ],
        };

        let bytes = content
            .encode()
            .map_err(|e| format!("Fehler beim Erzeugen der Seitenzahl: {e}"))?;
        let stream_id = doc.add_object(Stream::new(dictionary! {}, bytes));
        attach_overlay_stream(&mut doc, page_id, stream_id, false);
        numbered_count += 1;
    }

    doc.save(output_path)
        .map_err(|e| format!("Fehler beim Speichern mit Seitenzahlen: {e}"))?;

    Ok(numbered_count)
}
