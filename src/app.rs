use crate::pdf_ops::*;
use eframe::egui::{self, Color32, RichText, ScrollArea, Vec2};
use std::path::PathBuf;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Tab {
    Merge,
    Split,
    Rotate,
    Delete,
    Compress,
    ImagesToPdf,
    ExtractImages,
    Security,
    Metadata,
    TextInfo,
}

pub struct PdfToolkitApp {
    current_tab: Tab,
    status_message: Option<(String, bool)>, // (message, is_error)

    // Merge State
    merge_files: Vec<PathBuf>,

    // Split State
    split_file: Option<PathBuf>,
    split_page_count: usize,
    split_range_input: String,
    split_mode_all: bool,

    // Rotate State
    rotate_file: Option<PathBuf>,
    rotate_page_count: usize,
    rotate_pages_input: String,
    rotate_angle: i64,

    // Delete State
    delete_file: Option<PathBuf>,
    delete_page_count: usize,
    delete_pages_input: String,

    // Compress State
    compress_file: Option<PathBuf>,
    compress_max: bool,
    compress_result: Option<CompressResult>,

    // Images to PDF State
    img_files: Vec<PathBuf>,

    // Extract Images State
    extract_img_file: Option<PathBuf>,
    extract_img_count: Option<usize>,

    // Security State
    sec_file: Option<PathBuf>,
    sec_user_pwd: String,
    sec_owner_pwd: String,
    sec_use_aes256: bool,
    dec_file: Option<PathBuf>,
    dec_pwd: String,

    // Metadata State
    meta_file: Option<PathBuf>,
    metadata: PdfMetadata,

    // Text & Info State
    info_file: Option<PathBuf>,
    info_details: Option<PdfDetails>,
    extracted_text: String,
    text_page_range: String,
}

impl Default for PdfToolkitApp {
    fn default() -> Self {
        Self {
            current_tab: Tab::Merge,
            status_message: None,

            merge_files: Vec::new(),

            split_file: None,
            split_page_count: 0,
            split_range_input: "1-".to_string(),
            split_mode_all: false,

            rotate_file: None,
            rotate_page_count: 0,
            rotate_pages_input: "alle".to_string(),
            rotate_angle: 90,

            delete_file: None,
            delete_page_count: 0,
            delete_pages_input: "".to_string(),

            compress_file: None,
            compress_max: true,
            compress_result: None,

            img_files: Vec::new(),

            extract_img_file: None,
            extract_img_count: None,

            sec_file: None,
            sec_user_pwd: String::new(),
            sec_owner_pwd: String::new(),
            sec_use_aes256: true,
            dec_file: None,
            dec_pwd: String::new(),

            meta_file: None,
            metadata: PdfMetadata::default(),

            info_file: None,
            info_details: None,
            extracted_text: String::new(),
            text_page_range: "alle".to_string(),
        }
    }
}

impl PdfToolkitApp {
    fn set_status(&mut self, msg: impl Into<String>, is_error: bool) {
        self.status_message = Some((msg.into(), is_error));
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        ui.heading(RichText::new("🛠️ PDF Toolkit").strong().size(19.0));
        ui.label(RichText::new("100% Native Rust Desktop").small().weak());
        ui.add_space(15.0);

        let nav_button = |ui: &mut egui::Ui, current: &mut Tab, target: Tab, title: &str| {
            let is_selected = *current == target;
            let text = RichText::new(title).size(14.0);
            let btn = ui.add_sized(
                [ui.available_width(), 32.0],
                egui::SelectableLabel::new(is_selected, text),
            );
            if btn.clicked() {
                *current = target;
            }
        };

        nav_button(ui, &mut self.current_tab, Tab::Merge, "📄 Zusammenführen");
        nav_button(ui, &mut self.current_tab, Tab::Split, "✂️ Teilen / Extrahieren");
        nav_button(ui, &mut self.current_tab, Tab::Rotate, "🔄 Seiten Drehen");
        nav_button(ui, &mut self.current_tab, Tab::Delete, "🗑️ Seiten Löschen");
        nav_button(ui, &mut self.current_tab, Tab::Compress, "🗜️ Komprimieren");
        nav_button(ui, &mut self.current_tab, Tab::ImagesToPdf, "🖼️ Bilder zu PDF");
        nav_button(ui, &mut self.current_tab, Tab::ExtractImages, "🔍 Bilder Extrahieren");
        nav_button(ui, &mut self.current_tab, Tab::Security, "🔒 Sicherheit & Passwort");
        nav_button(ui, &mut self.current_tab, Tab::Metadata, "📝 Metadaten");
        nav_button(ui, &mut self.current_tab, Tab::TextInfo, "📊 Info & Text");

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);
        ui.label(RichText::new("🔒 100% Offline").small().weak());
        ui.label(RichText::new("⚡ Ohne Cloud / Web").small().weak());
    }

    fn render_merge_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("PDF-Dateien zusammenführen (Merge)");
        ui.label("Füge mehrere PDFs hinzu, passe die Reihenfolge an und erstelle ein einzelnes Gesamtdokument.");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("➕ PDFs hinzufügen...").clicked() {
                if let Some(files) = rfd::FileDialog::new()
                    .add_filter("PDF Dokumente", &["pdf"])
                    .pick_files()
                {
                    self.merge_files.extend(files);
                }
            }
            if !self.merge_files.is_empty() && ui.button("🗑️ Liste leeren").clicked() {
                self.merge_files.clear();
            }
        });

        ui.add_space(10.0);

        ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                if self.merge_files.is_empty() {
                    ui.label(RichText::new("Keine PDFs ausgewählt. Klicke auf 'PDFs hinzufügen...'").italics().weak());
                } else {
                    let mut to_remove = None;
                    let mut move_up = None;
                    let mut move_down = None;

                    for (idx, file) in self.merge_files.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}.", idx + 1));
                            let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("PDF");
                            ui.label(RichText::new(name).strong());

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("❌").clicked() {
                                    to_remove = Some(idx);
                                }
                                if idx + 1 < self.merge_files.len() && ui.small_button("⬇").clicked() {
                                    move_down = Some(idx);
                                }
                                if idx > 0 && ui.small_button("⬆").clicked() {
                                    move_up = Some(idx);
                                }
                            });
                        });
                        ui.separator();
                    }

                    if let Some(idx) = to_remove {
                        self.merge_files.remove(idx);
                    } else if let Some(idx) = move_up {
                        self.merge_files.swap(idx, idx - 1);
                    } else if let Some(idx) = move_down {
                        self.merge_files.swap(idx, idx + 1);
                    }
                }
            });

        ui.add_space(15.0);
        if ui.add_enabled(self.merge_files.len() >= 2, egui::Button::new("💾 Zusammenführen und Speichern...").min_size(Vec2::new(240.0, 36.0))).clicked() {
            if let Some(dest) = rfd::FileDialog::new()
                .add_filter("PDF Dokument", &["pdf"])
                .set_file_name("zusammengefuehrt.pdf")
                .save_file()
            {
                match merge_pdfs(&self.merge_files, &dest) {
                    Ok(pages) => self.set_status(format!("Erfolgreich zusammengeführt ({pages} Seiten): {}", dest.display()), false),
                    Err(e) => self.set_status(e, true),
                }
            }
        }
    }

    fn render_split_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("PDF Teilen & Seiten Extrahieren (Split)");
        ui.label("Wähle Seitenbereiche aus oder zerlege eine PDF in Einzelseiten.");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("📂 PDF auswählen...").clicked() {
                if let Some(file) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).pick_file() {
                    match get_pdf_details(&file) {
                        Ok(details) => {
                            self.split_page_count = details.page_count;
                            self.split_range_input = format!("1-{}", details.page_count);
                            self.split_file = Some(file);
                        }
                        Err(e) => self.set_status(e, true),
                    }
                }
            }
            if let Some(f) = &self.split_file {
                ui.label(RichText::new(f.file_name().unwrap().to_string_lossy()).strong());
                ui.label(format!("({} Seiten)", self.split_page_count));
            }
        });

        if self.split_file.is_some() {
            ui.add_space(15.0);
            ui.checkbox(&mut self.split_mode_all, "Jede einzelne Seite als separate PDF speichern");

            if !self.split_mode_all {
                ui.add_space(5.0);
                ui.label("Seitenbereich (z.B. '1-3, 5, 8-10'):");
                ui.text_edit_singleline(&mut self.split_range_input);
            }

            ui.add_space(15.0);
            if ui.button(RichText::new("✂️ Ausführen...").size(15.0)).clicked() {
                let input = self.split_file.as_ref().unwrap();
                if self.split_mode_all {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        match split_all_pages(input, &folder) {
                            Ok(count) => self.set_status(format!("{count} Einzelseiten in '{}' gespeichert", folder.display()), false),
                            Err(e) => self.set_status(e, true),
                        }
                    }
                } else {
                    match parse_page_ranges(&self.split_range_input, self.split_page_count as u32) {
                        Ok(pages) => {
                            if let Some(dest) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).set_file_name("extrahiert.pdf").save_file() {
                                match extract_pages(input, &pages, &dest) {
                                    Ok(n) => self.set_status(format!("{n} Seiten erfolgreich extrahiert nach: {}", dest.display()), false),
                                    Err(e) => self.set_status(e, true),
                                }
                            }
                        }
                        Err(e) => self.set_status(e, true),
                    }
                }
            }
        }
    }

    fn render_rotate_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("PDF Seiten Drehen (Rotate)");
        ui.label("Drehe einzelne oder alle Seiten im Dokument um 90°, 180° oder 270°.");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("📂 PDF auswählen...").clicked() {
                if let Some(file) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).pick_file() {
                    if let Ok(d) = get_pdf_details(&file) {
                        self.rotate_page_count = d.page_count;
                        self.rotate_file = Some(file);
                    }
                }
            }
            if let Some(f) = &self.rotate_file {
                ui.label(RichText::new(f.file_name().unwrap().to_string_lossy()).strong());
                ui.label(format!("({} Seiten)", self.rotate_page_count));
            }
        });

        if self.rotate_file.is_some() {
            ui.add_space(15.0);
            ui.horizontal(|ui| {
                ui.label("Drehwinkel:");
                ui.radio_value(&mut self.rotate_angle, 90, "90° (Uhrzeigersinn)");
                ui.radio_value(&mut self.rotate_angle, 180, "180°");
                ui.radio_value(&mut self.rotate_angle, 270, "270° (Gegen Uhrzeigersinn)");
            });

            ui.add_space(10.0);
            ui.label("Betroffene Seiten ('alle' oder z.B. '1, 3, 5-7'):");
            ui.text_edit_singleline(&mut self.rotate_pages_input);

            ui.add_space(15.0);
            if ui.button(RichText::new("🔄 Seiten drehen und speichern...").size(15.0)).clicked() {
                let input = self.rotate_file.as_ref().unwrap();
                let pages = if self.rotate_pages_input.trim().eq_ignore_ascii_case("alle") {
                    None
                } else {
                    match parse_page_ranges(&self.rotate_pages_input, self.rotate_page_count as u32) {
                        Ok(p) => Some(p),
                        Err(e) => {
                            self.set_status(e, true);
                            return;
                        }
                    }
                };

                if let Some(dest) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).set_file_name("gedreht.pdf").save_file() {
                    match rotate_pdf(input, pages.as_deref(), self.rotate_angle, &dest) {
                        Ok(n) => self.set_status(format!("{n} Seiten erfolgreich gedreht: {}", dest.display()), false),
                        Err(e) => self.set_status(e, true),
                    }
                }
            }
        }
    }

    fn render_delete_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Seiten Löschen (Delete Pages)");
        ui.label("Entferne gezielt überflüssige Seiten aus deinem PDF-Dokument.");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("📂 PDF auswählen...").clicked() {
                if let Some(file) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).pick_file() {
                    if let Ok(d) = get_pdf_details(&file) {
                        self.delete_page_count = d.page_count;
                        self.delete_file = Some(file);
                    }
                }
            }
            if let Some(f) = &self.delete_file {
                ui.label(RichText::new(f.file_name().unwrap().to_string_lossy()).strong());
                ui.label(format!("({} Seiten)", self.delete_page_count));
            }
        });

        if self.delete_file.is_some() {
            ui.add_space(15.0);
            ui.label("Zu löschende Seiten (z.B. '2, 4-6'):");
            ui.text_edit_singleline(&mut self.delete_pages_input);

            ui.add_space(15.0);
            if ui.button(RichText::new("🗑️ Seiten löschen und neue PDF speichern...").size(15.0)).clicked() {
                let input = self.delete_file.as_ref().unwrap();
                match parse_page_ranges(&self.delete_pages_input, self.delete_page_count as u32) {
                    Ok(to_del) => {
                        if let Some(dest) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).set_file_name("bereinigt.pdf").save_file() {
                            match delete_pages_from_pdf(input, &to_del, &dest) {
                                Ok(rem) => self.set_status(format!("{} Seiten gelöscht, {rem} verbleibend: {}", to_del.len(), dest.display()), false),
                                Err(e) => self.set_status(e, true),
                            }
                        }
                    }
                    Err(e) => self.set_status(e, true),
                }
            }
        }
    }

    fn render_compress_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("PDF Komprimieren & Optimieren");
        ui.label("Reduziere die Dateigröße durch moderne PDF-Stream- und Struktur-Kompression.");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("📂 PDF auswählen...").clicked() {
                if let Some(file) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).pick_file() {
                    self.compress_file = Some(file);
                    self.compress_result = None;
                }
            }
            if let Some(f) = &self.compress_file {
                let size = std::fs::metadata(f).map(|m| m.len()).unwrap_or(0);
                ui.label(RichText::new(f.file_name().unwrap().to_string_lossy()).strong());
                ui.label(format!("({:.2} MB)", size as f64 / 1_048_576.0));
            }
        });

        if self.compress_file.is_some() {
            ui.add_space(15.0);
            ui.checkbox(&mut self.compress_max, "Maximale Kompression (Object Streams + Level 9)");

            ui.add_space(15.0);
            if ui.button(RichText::new("🗜️ Jetzt komprimieren...").size(15.0)).clicked() {
                let input = self.compress_file.as_ref().unwrap();
                if let Some(dest) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).set_file_name("komprimiert.pdf").save_file() {
                    match compress_pdf(input, &dest, self.compress_max) {
                        Ok(res) => {
                            self.set_status(
                                format!(
                                    "Kompression fertig! Vorher: {:.2} MB -> Nachher: {:.2} MB ({:.1}% Ersparnis)",
                                    res.original_size as f64 / 1_048_576.0,
                                    res.compressed_size as f64 / 1_048_576.0,
                                    res.ratio_percent
                                ),
                                false,
                            );
                            self.compress_result = Some(res);
                        }
                        Err(e) => self.set_status(e, true),
                    }
                }
            }

            if let Some(res) = &self.compress_result {
                ui.add_space(15.0);
                ui.group(|ui| {
                    ui.label(RichText::new("📊 Kompressions-Ergebnis").strong());
                    ui.label(format!("Originalgröße: {:.2} KB", res.original_size as f64 / 1024.0));
                    ui.label(format!("Neue Größe: {:.2} KB", res.compressed_size as f64 / 1024.0));
                    ui.label(RichText::new(format!("Größenreduktion: {:.1}%", res.ratio_percent)).color(Color32::from_rgb(46, 204, 113)).strong());
                });
            }
        }
    }

    fn render_images_to_pdf_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Bilder zu PDF (Images to PDF)");
        ui.label("Wandle mehrere Fotos oder Grafiken (JPG, PNG, BMP) in ein mehrseitiges PDF um.");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("➕ Bilder hinzufügen...").clicked() {
                if let Some(files) = rfd::FileDialog::new()
                    .add_filter("Bilder", &["jpg", "jpeg", "png", "bmp"])
                    .pick_files()
                {
                    self.img_files.extend(files);
                }
            }
            if !self.img_files.is_empty() && ui.button("🗑️ Alle entfernen").clicked() {
                self.img_files.clear();
            }
        });

        ui.add_space(10.0);

        ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
            if self.img_files.is_empty() {
                ui.label(RichText::new("Keine Bilder in der Liste.").italics().weak());
            } else {
                let mut to_remove = None;
                for (idx, img) in self.img_files.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}.", idx + 1));
                        ui.label(img.file_name().unwrap().to_string_lossy());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("❌").clicked() {
                                to_remove = Some(idx);
                            }
                        });
                    });
                    ui.separator();
                }
                if let Some(idx) = to_remove {
                    self.img_files.remove(idx);
                }
            }
        });

        ui.add_space(15.0);
        if ui.add_enabled(!self.img_files.is_empty(), egui::Button::new("📄 Als PDF speichern...").min_size(Vec2::new(200.0, 36.0))).clicked() {
            if let Some(dest) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).set_file_name("bilder.pdf").save_file() {
                match convert_images_to_pdf(&self.img_files, &dest) {
                    Ok(n) => self.set_status(format!("{n} Bilder erfolgreich als PDF gespeichert: {}", dest.display()), false),
                    Err(e) => self.set_status(e, true),
                }
            }
        }
    }

    fn render_extract_images_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Bilder aus PDF extrahieren");
        ui.label("Exportiere alle in einem PDF eingebetteten Bilder automatisch in einen Ordner.");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("📂 PDF auswählen...").clicked() {
                if let Some(file) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).pick_file() {
                    self.extract_img_file = Some(file);
                    self.extract_img_count = None;
                }
            }
            if let Some(f) = &self.extract_img_file {
                ui.label(RichText::new(f.file_name().unwrap().to_string_lossy()).strong());
            }
        });

        if self.extract_img_file.is_some() {
            ui.add_space(15.0);
            if ui.button(RichText::new("🔍 Zielordner wählen & Bilder extrahieren...").size(15.0)).clicked() {
                let input = self.extract_img_file.as_ref().unwrap();
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    match extract_images_from_pdf(input, &folder) {
                        Ok(count) => {
                            self.extract_img_count = Some(count);
                            self.set_status(format!("{count} Bilder extrahiert nach: {}", folder.display()), false);
                        }
                        Err(e) => self.set_status(e, true),
                    }
                }
            }

            if let Some(count) = self.extract_img_count {
                ui.add_space(10.0);
                ui.label(RichText::new(format!("Insgesamt {count} Bilder gefunden und gespeichert.")).strong());
            }
        }
    }

    fn render_security_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Sicherheit, Verschlüsselung & Passwort");
        ui.label("Schütze PDFs mit AES-Verschlüsselung oder entferne bestehende Passwörter.");
        ui.add_space(15.0);

        ui.group(|ui| {
            ui.heading(RichText::new("🔒 PDF Verschlüsseln").size(16.0));
            ui.horizontal(|ui| {
                if ui.button("📂 Zu schützende PDF wählen...").clicked() {
                    if let Some(file) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).pick_file() {
                        self.sec_file = Some(file);
                    }
                }
                if let Some(f) = &self.sec_file {
                    ui.label(RichText::new(f.file_name().unwrap().to_string_lossy()).strong());
                }
            });

            if self.sec_file.is_some() {
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.label("Passwort zum Öffnen:");
                    ui.add(egui::TextEdit::singleline(&mut self.sec_user_pwd).password(true));
                });
                ui.horizontal(|ui| {
                    ui.label("Besitzer-Passwort (optional):");
                    ui.add(egui::TextEdit::singleline(&mut self.sec_owner_pwd).password(true));
                });
                ui.checkbox(&mut self.sec_use_aes256, "AES-256 Verschlüsselung nutzen (höchste Sicherheit)");

                if ui.button("🔒 Verschlüsseln und Speichern...").clicked() {
                    let input = self.sec_file.as_ref().unwrap();
                    if let Some(dest) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).set_file_name("geschuetzt.pdf").save_file() {
                        match encrypt_pdf(input, &self.sec_user_pwd, &self.sec_owner_pwd, self.sec_use_aes256, &dest) {
                            Ok(()) => self.set_status(format!("PDF erfolgreich verschlüsselt: {}", dest.display()), false),
                            Err(e) => self.set_status(e, true),
                        }
                    }
                }
            }
        });

        ui.add_space(20.0);

        ui.group(|ui| {
            ui.heading(RichText::new("🔓 Passwort Entfernen / Entschlüsseln").size(16.0));
            ui.horizontal(|ui| {
                if ui.button("📂 Geschützte PDF wählen...").clicked() {
                    if let Some(file) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).pick_file() {
                        self.dec_file = Some(file);
                    }
                }
                if let Some(f) = &self.dec_file {
                    ui.label(RichText::new(f.file_name().unwrap().to_string_lossy()).strong());
                }
            });

            if self.dec_file.is_some() {
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.label("Passwort:");
                    ui.add(egui::TextEdit::singleline(&mut self.dec_pwd).password(true));
                });

                if ui.button("🔓 Entschlüsseln und Speichern...").clicked() {
                    let input = self.dec_file.as_ref().unwrap();
                    if let Some(dest) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).set_file_name("entschluesselt.pdf").save_file() {
                        match decrypt_pdf(input, &self.dec_pwd, &dest) {
                            Ok(()) => self.set_status(format!("PDF erfolgreich entschlüsselt: {}", dest.display()), false),
                            Err(e) => self.set_status(e, true),
                        }
                    }
                }
            }
        });
    }

    fn render_metadata_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("PDF Metadaten Editor");
        ui.label("Lies und bearbeite Dokument-Eigenschaften wie Titel, Autor, Thema und Stichwörter.");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("📂 PDF auswählen...").clicked() {
                if let Some(file) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).pick_file() {
                    match read_metadata(&file) {
                        Ok(meta) => {
                            self.metadata = meta;
                            self.meta_file = Some(file);
                        }
                        Err(e) => self.set_status(e, true),
                    }
                }
            }
            if let Some(f) = &self.meta_file {
                ui.label(RichText::new(f.file_name().unwrap().to_string_lossy()).strong());
            }
        });

        if self.meta_file.is_some() {
            ui.add_space(15.0);
            egui::Grid::new("meta_grid").num_columns(2).spacing([15.0, 10.0]).show(ui, |ui| {
                ui.label("Titel:");
                ui.text_edit_singleline(&mut self.metadata.title);
                ui.end_row();

                ui.label("Autor:");
                ui.text_edit_singleline(&mut self.metadata.author);
                ui.end_row();

                ui.label("Thema / Betreff:");
                ui.text_edit_singleline(&mut self.metadata.subject);
                ui.end_row();

                ui.label("Stichwörter:");
                ui.text_edit_singleline(&mut self.metadata.keywords);
                ui.end_row();

                ui.label("Ersteller (Creator):");
                ui.text_edit_singleline(&mut self.metadata.creator);
                ui.end_row();

                ui.label("Producer:");
                ui.text_edit_singleline(&mut self.metadata.producer);
                ui.end_row();
            });

            ui.add_space(15.0);
            if ui.button(RichText::new("💾 Metadaten Speichern...").size(15.0)).clicked() {
                let input = self.meta_file.as_ref().unwrap();
                if let Some(dest) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).set_file_name("aktualisiert.pdf").save_file() {
                    match write_metadata(input, &self.metadata, &dest) {
                        Ok(()) => self.set_status(format!("Metadaten erfolgreich aktualisiert: {}", dest.display()), false),
                        Err(e) => self.set_status(e, true),
                    }
                }
            }
        }
    }

    fn render_text_info_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("PDF Info & Text-Extraktion");
        ui.label("Detaillierte Datei-Analyse und Auslesen von Dokumententext.");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("📂 PDF analysieren...").clicked() {
                if let Some(file) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).pick_file() {
                    match get_pdf_details(&file) {
                        Ok(d) => {
                            self.info_details = Some(d);
                            self.info_file = Some(file);
                            self.extracted_text.clear();
                        }
                        Err(e) => self.set_status(e, true),
                    }
                }
            }
            if let Some(f) = &self.info_file {
                ui.label(RichText::new(f.file_name().unwrap().to_string_lossy()).strong());
            }
        });

        if let Some(details) = self.info_details.clone() {
            ui.add_space(15.0);
            ui.group(|ui| {
                ui.label(RichText::new("📄 Dokumentinformationen").strong());
                ui.label(format!("Seitenanzahl: {}", details.page_count));
                ui.label(format!("PDF Version: {}", details.version));
                ui.label(format!("PDF Objekte: {}", details.object_count));
                ui.label(format!("Dateigröße: {:.2} KB", details.file_size_bytes as f64 / 1024.0));
                ui.label(format!("Verschlüsselt: {}", if details.is_encrypted { "Ja (Geschützt)" } else { "Nein" }));
            });

            ui.add_space(15.0);
            let mut extract_clicked = false;
            ui.horizontal(|ui| {
                ui.label("Text extrahieren für Seiten ('alle' oder z.B. '1, 2'):");
                ui.text_edit_singleline(&mut self.text_page_range);

                if ui.button("🔍 Text auslesen").clicked() {
                    extract_clicked = true;
                }
            });

            if extract_clicked {
                let input = self.info_file.as_ref().unwrap();
                match parse_page_ranges(&self.text_page_range, details.page_count as u32) {
                    Ok(pages) => {
                        match extract_text(input, &pages) {
                            Ok(txt) => {
                                self.extracted_text = txt;
                                self.set_status("Text erfolgreich extrahiert", false);
                            }
                            Err(e) => self.set_status(e, true),
                        }
                    }
                    Err(e) => self.set_status(e, true),
                }
            }

            if !self.extracted_text.is_empty() {
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Extrahierter Text:").strong());
                    if ui.small_button("📋 Als .txt speichern").clicked() {
                        if let Some(dest) = rfd::FileDialog::new().add_filter("Text", &["txt"]).set_file_name("extrahiert.txt").save_file() {
                            if std::fs::write(&dest, &self.extracted_text).is_ok() {
                                self.set_status(format!("Text gespeichert: {}", dest.display()), false);
                            }
                        }
                    }
                });

                ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.extracted_text)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );
                });
            }
        }
    }
}

impl eframe::App for PdfToolkitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drag and drop handling
        ctx.input(|i| {
            if let Some(dropped) = i.raw.dropped_files.first() {
                if let Some(path) = &dropped.path {
                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                    if ext == "pdf" {
                        match self.current_tab {
                            Tab::Merge => self.merge_files.push(path.clone()),
                            Tab::Split => {
                                if let Ok(d) = get_pdf_details(path) {
                                    self.split_page_count = d.page_count;
                                    self.split_range_input = format!("1-{}", d.page_count);
                                    self.split_file = Some(path.clone());
                                }
                            }
                            Tab::Rotate => {
                                if let Ok(d) = get_pdf_details(path) {
                                    self.rotate_page_count = d.page_count;
                                    self.rotate_file = Some(path.clone());
                                }
                            }
                            Tab::Delete => {
                                if let Ok(d) = get_pdf_details(path) {
                                    self.delete_page_count = d.page_count;
                                    self.delete_file = Some(path.clone());
                                }
                            }
                            Tab::Compress => self.compress_file = Some(path.clone()),
                            Tab::ExtractImages => self.extract_img_file = Some(path.clone()),
                            Tab::Security => self.sec_file = Some(path.clone()),
                            Tab::Metadata => {
                                if let Ok(m) = read_metadata(path) {
                                    self.metadata = m;
                                    self.meta_file = Some(path.clone());
                                }
                            }
                            Tab::TextInfo => {
                                if let Ok(d) = get_pdf_details(path) {
                                    self.info_details = Some(d);
                                    self.info_file = Some(path.clone());
                                    self.extracted_text.clear();
                                }
                            }
                            _ => {}
                        }
                    } else if matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "bmp") && self.current_tab == Tab::ImagesToPdf {
                        self.img_files.push(path.clone());
                    }
                }
            }
        });

        // Top / Bottom Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some((msg, is_error)) = &self.status_message {
                    let color = if *is_error {
                        Color32::from_rgb(231, 76, 60)
                    } else {
                        Color32::from_rgb(46, 204, 113)
                    };
                    ui.label(RichText::new(msg).color(color).strong());
                } else {
                    ui.label(RichText::new("Bereit. Dateien können per Klick oder Drag & Drop geladen werden.").weak());
                }
            });
        });

        // Sidebar
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .default_width(230.0)
            .show(ctx, |ui| {
                self.render_sidebar(ui);
            });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            match self.current_tab {
                Tab::Merge => self.render_merge_tab(ui),
                Tab::Split => self.render_split_tab(ui),
                Tab::Rotate => self.render_rotate_tab(ui),
                Tab::Delete => self.render_delete_tab(ui),
                Tab::Compress => self.render_compress_tab(ui),
                Tab::ImagesToPdf => self.render_images_to_pdf_tab(ui),
                Tab::ExtractImages => self.render_extract_images_tab(ui),
                Tab::Security => self.render_security_tab(ui),
                Tab::Metadata => self.render_metadata_tab(ui),
                Tab::TextInfo => self.render_text_info_tab(ui),
            }
        });
    }
}
