#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use pdf_toolkit_rust::app::PdfToolkitApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("PDF Werkzeugkasten")
            .with_inner_size([1080.0, 720.0])
            .with_min_inner_size([860.0, 560.0])
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "PDF Werkzeugkasten",
        options,
        Box::new(|_cc| Ok(Box::new(PdfToolkitApp::default()))),
    )
}
