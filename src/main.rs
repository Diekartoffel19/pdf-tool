#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
pub mod pdf_ops;

use app::PdfToolkitApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("PDF Werkzeugkasten Pro")
            .with_inner_size([1240.0, 800.0])
            .with_min_inner_size([960.0, 620.0])
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "PDF Werkzeugkasten Pro",
        options,
        Box::new(|cc| Ok(Box::new(PdfToolkitApp::new(cc)))),
    )
}
