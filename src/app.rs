use crate::pdf_ops::*;
use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Pos2, Rect, RichText, ScrollArea, Sense, Stroke,
    StrokeKind, Vec2,
};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Tab {
    Merge,
    Split,
    Organize,
    Rotate,
    Delete,
    Watermark,
    PageNumbers,
    CropScale,
    Compress,
    ImagesToPdf,
    ExtractImages,
    Security,
    Sanitize,
    Metadata,
    TextInfo,
}

impl Tab {
    fn info(self) -> (&'static str, &'static str, &'static str, &'static str) {
        match self {
            Tab::Merge => (
                "📄",
                "Zusammenführen & Duplex",
                "SEITEN & STRUKTUR",
                "Verbinde beliebig viele PDFs oder mische Vorder- und Rückseiten eines Dokumentenscanners.",
            ),
            Tab::Split => (
                "✂️",
                "Teilen & Extrahieren",
                "SEITEN & STRUKTUR",
                "Extrahiere gezielt Seitenbereiche, teile in feste Blöcke oder zerlege in Einzelseiten.",
            ),
            Tab::Organize => (
                "🗂️",
                "Seiten-Manager",
                "SEITEN & STRUKTUR",
                "Sortiere Seiten visuell per Klick um, dupliziere Seiten oder füge Leerseiten ein.",
            ),
            Tab::Rotate => (
                "🔄",
                "Seiten Drehen",
                "SEITEN & STRUKTUR",
                "Drehe ausgewählte, gerade, ungerade oder alle Seiten verlustfrei um 90°, 180° oder 270°.",
            ),
            Tab::Delete => (
                "🗑️",
                "Seiten Löschen",
                "SEITEN & STRUKTUR",
                "Entferne überflüssige Seiten per Klick auf die Seiten-Kacheln oder per Bereichseingabe.",
            ),
            Tab::Watermark => (
                "💧",
                "Wasserzeichen & Stempel",
                "STEMPEL & LAYOUT",
                "Platziere Text-Wasserzeichen und Stempel mit Live-Seiten-Vorschau im Vorder- oder Hintergrund.",
            ),
            Tab::PageNumbers => (
                "🔢",
                "Seitenzahlen & Bates",
                "STEMPEL & LAYOUT",
                "Füge professionelle Seitenzahlen oder fortlaufende Bates-Nummern mit Live-Vorschau ein.",
            ),
            Tab::CropScale => (
                "📐",
                "Zuschneiden & Format",
                "STEMPEL & LAYOUT",
                "Schneide Seitenränder millimetergenau mit Schnitt-Vorschau ab oder skaliere auf DIN-Formate.",
            ),
            Tab::Compress => (
                "🗜️",
                "Komprimieren",
                "BILDER & GRÖSSE",
                "Reduziere die Dateigröße durch PDF-Objekt-Bereinigung und maximale Stream-Kompression.",
            ),
            Tab::ImagesToPdf => (
                "🖼️",
                "Bilder zu PDF",
                "BILDER & GRÖSSE",
                "Wandle Fotos und Scans (JPG, PNG, BMP) in ein mehrseitiges PDF-Dokument um.",
            ),
            Tab::ExtractImages => (
                "🔍",
                "Bilder Extrahieren",
                "BILDER & GRÖSSE",
                "Exportiere alle in einem PDF eingebetteten Originalbilder verlustfrei in einen Ordner.",
            ),
            Tab::Security => (
                "🔒",
                "Passwort & Rechte",
                "SCHUTZ & ANALYSE",
                "Schütze PDFs mit Passwort & Dokumentrechten oder hebe bestehenden Passwortschutz auf.",
            ),
            Tab::Sanitize => (
                "🛡️",
                "Datenschutz-Reiniger",
                "SCHUTZ & ANALYSE",
                "Entferne unsichtbare Metadaten, Kommentare, Formulare, Skripte und verwaiste Objekte.",
            ),
            Tab::Metadata => (
                "📝",
                "Metadaten-Editor",
                "SCHUTZ & ANALYSE",
                "Lies und bearbeite Dokument-Eigenschaften wie Titel, Autor, Thema und Stichwörter.",
            ),
            Tab::TextInfo => (
                "📊",
                "Analyse & Textsuche",
                "SCHUTZ & ANALYSE",
                "Tiefenanalyse der Dokumentstruktur sowie Volltext-Extraktion mit Live-Suchfilter.",
            ),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum SplitMode {
    Range,
    Chunks,
    SinglePages,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum CropScaleMode {
    MarginCrop,
    FormatScale,
}

fn accent_color(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(99, 102, 241) // Indigo 500
    } else {
        Color32::from_rgb(79, 70, 229) // Indigo 600
    }
}

fn card_bg(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(24, 28, 38)
    } else {
        Color32::from_rgb(255, 255, 255)
    }
}

fn card_border(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(45, 53, 72)
    } else {
        Color32::from_rgb(226, 232, 240)
    }
}

fn subtle_bg(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(17, 20, 28)
    } else {
        Color32::from_rgb(241, 245, 249)
    }
}

fn ui_card<R>(ui: &mut egui::Ui, dark: bool, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(card_bg(dark))
        .stroke(Stroke::new(1.0_f32, card_border(dark)))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(egui::Margin::same(14))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add_contents(ui)
        })
        .inner
}

fn primary_button(ui: &mut egui::Ui, dark: bool, enabled: bool, label: &str) -> bool {
    let fill = if enabled {
        accent_color(dark)
    } else if dark {
        Color32::from_rgb(38, 44, 58)
    } else {
        Color32::from_rgb(226, 232, 240)
    };
    let text_color = if enabled {
        Color32::WHITE
    } else if dark {
        Color32::from_rgb(110, 120, 140)
    } else {
        Color32::from_rgb(148, 163, 184)
    };

    let btn = egui::Button::new(RichText::new(label).size(15.0).strong().color(text_color))
        .fill(fill)
        .corner_radius(CornerRadius::same(8))
        .min_size(Vec2::new(260.0, 38.0));

    ui.add_enabled(enabled, btn).clicked()
}

fn render_file_dropzone(
    ui: &mut egui::Ui,
    dark: bool,
    current_file: &Option<PathBuf>,
    page_count: Option<usize>,
    button_label: &str,
) -> (Option<PathBuf>, bool) {
    let mut picked = None;
    let mut cleared = false;
    let accent = accent_color(dark);

    let bg = if current_file.is_some() {
        card_bg(dark)
    } else {
        subtle_bg(dark)
    };
    let border_col = if current_file.is_some() {
        accent.gamma_multiply(0.65)
    } else {
        card_border(dark)
    };

    egui::Frame::new()
        .fill(bg)
        .stroke(Stroke::new(1.5_f32, border_col))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            if let Some(f) = current_file {
                ui.horizontal(|ui| {
                    egui::Frame::new()
                        .fill(accent.gamma_multiply(0.18))
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(egui::Margin::symmetric(10, 6))
                        .show(ui, |ui| {
                            ui.label(RichText::new("📄 PDF").strong().color(accent));
                        });

                    let name = f.file_name().and_then(|n| n.to_str()).unwrap_or("Dokument.pdf");
                    let size_kb = std::fs::metadata(f)
                        .map(|m| m.len() as f64 / 1024.0)
                        .unwrap_or(0.0);

                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.label(RichText::new(name).size(14.5).strong());
                        ui.horizontal(|ui| {
                            if let Some(pages) = page_count {
                                ui.label(
                                    RichText::new(format!("📑 {pages} Seiten"))
                                        .small()
                                        .color(accent),
                                );
                                ui.label(RichText::new("•").small().weak());
                            }
                            let size_str = if size_kb >= 1024.0 {
                                format!("{:.2} MB", size_kb / 1024.0)
                            } else {
                                format!("{size_kb:.1} KB")
                            };
                            ui.label(RichText::new(format!("💾 {size_str}")).small().weak());
                        });
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button("✖")
                            .on_hover_text("Datei aus Werkzeug schließen")
                            .clicked()
                        {
                            cleared = true;
                        }
                        if ui
                            .small_button("📂 Explorer")
                            .on_hover_text("Im Datei-Explorer zeigen")
                            .clicked()
                        {
                            reveal_in_explorer(f);
                        }
                        if ui.button("🔄 Andere PDF...").clicked() {
                            if let Some(file) = rfd::FileDialog::new()
                                .add_filter("PDF Dokument", &["pdf"])
                                .pick_file()
                            {
                                picked = Some(file);
                            }
                        }
                    });
                });
            } else {
                ui.horizontal(|ui| {
                    let btn = egui::Button::new(
                        RichText::new(button_label)
                            .size(14.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(accent)
                    .corner_radius(CornerRadius::same(8))
                    .min_size(Vec2::new(190.0, 34.0));

                    if ui.add(btn).clicked() {
                        if let Some(file) = rfd::FileDialog::new()
                            .add_filter("PDF Dokument", &["pdf"])
                            .pick_file()
                        {
                            picked = Some(file);
                        }
                    }
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.label(RichText::new("Noch keine PDF-Datei geladen").strong());
                        ui.label(
                            RichText::new(
                                "Klicke auf den Button oder ziehe eine PDF per Drag & Drop in das Fenster",
                            )
                            .small()
                            .weak(),
                        );
                    });
                });
            }
        });

    (picked, cleared)
}

fn render_interactive_page_pills(
    ui: &mut egui::Ui,
    dark: bool,
    page_count: usize,
    range_input: &mut String,
    highlight_color: Color32,
    action_verb: &str,
) {
    if page_count == 0 || page_count > 120 {
        return;
    }
    let selected_pages = parse_page_ranges(range_input, page_count as u32).unwrap_or_default();

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!(
                "🖱️ Visuelle Seitenauswahl ({}/{} Seiten {action_verb} – Klick zum Umschalten):",
                selected_pages.len(),
                page_count
            ))
            .small()
            .weak(),
        );
    });

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(5.0, 5.0);
        for p in 1..=(page_count as u32) {
            let is_sel = selected_pages.contains(&p);
            let fill = if is_sel {
                highlight_color
            } else if dark {
                Color32::from_rgb(35, 41, 56)
            } else {
                Color32::from_rgb(241, 245, 249)
            };
            let fg = if is_sel {
                Color32::WHITE
            } else if dark {
                Color32::from_rgb(200, 210, 225)
            } else {
                Color32::from_rgb(51, 65, 85)
            };

            let btn = egui::Button::new(RichText::new(format!("{p}")).size(12.5).strong().color(fg))
                .fill(fill)
                .corner_radius(CornerRadius::same(6))
                .min_size(Vec2::new(34.0, 26.0));

            if ui
                .add(btn)
                .on_hover_text(format!("Seite {p} ein-/abwählen"))
                .clicked()
            {
                let mut next = selected_pages.clone();
                if is_sel {
                    next.retain(|&x| x != p);
                } else {
                    next.push(p);
                    next.sort_unstable();
                }
                *range_input = format_pages_compact(&next);
            }
        }
    });
}

pub struct PdfToolkitApp {
    current_tab: Tab,
    dark_mode: bool,
    sidebar_search: String,
    sync_single_file: bool,
    global_file: Option<PathBuf>,
    status_message: Option<(String, bool)>,
    last_saved_path: Option<PathBuf>,

    // Merge & Duplex State
    merge_files: Vec<PathBuf>,
    duplex_mode: bool,
    duplex_front: Option<PathBuf>,
    duplex_back: Option<PathBuf>,
    duplex_reverse_back: bool,

    // Split State
    split_file: Option<PathBuf>,
    split_page_count: usize,
    split_mode: SplitMode,
    split_range_input: String,
    split_chunk_size: usize,

    // Organize State
    org_file: Option<PathBuf>,
    org_page_count: usize,
    org_sequence_input: String,
    org_blank_after_page: u32,
    org_blank_count: usize,
    org_blank_landscape: bool,

    // Rotate State
    rotate_file: Option<PathBuf>,
    rotate_page_count: usize,
    rotate_pages_input: String,
    rotate_angle: i64,

    // Delete State
    delete_file: Option<PathBuf>,
    delete_page_count: usize,
    delete_pages_input: String,

    // Watermark State
    wm_file: Option<PathBuf>,
    wm_page_count: usize,
    wm_options: WatermarkOptions,
    wm_color_preset: usize,
    wm_pages_input: String,

    // Page Numbers State
    pn_file: Option<PathBuf>,
    pn_page_count: usize,
    pn_options: PageNumberOptions,

    // Crop & Scale State
    cs_file: Option<PathBuf>,
    cs_page_count: usize,
    cs_mode: CropScaleMode,
    cs_top_mm: f32,
    cs_bottom_mm: f32,
    cs_left_mm: f32,
    cs_right_mm: f32,
    cs_uniform_mm: bool,
    cs_target_format: TargetPageFormat,
    cs_pages_input: String,

    // Compress State
    compress_file: Option<PathBuf>,
    compress_max: bool,
    compress_result: Option<CompressResult>,

    // Images to PDF State
    img_files: Vec<PathBuf>,
    img_layout: ImagePageLayout,
    img_margin_pt: f32,

    // Extract Images State
    extract_img_file: Option<PathBuf>,
    extract_img_count: Option<usize>,

    // Security State
    sec_file: Option<PathBuf>,
    sec_user_pwd: String,
    sec_owner_pwd: String,
    sec_use_aes256: bool,
    sec_perms: SecurityPermissions,
    dec_file: Option<PathBuf>,
    dec_pwd: String,

    // Sanitize State
    san_file: Option<PathBuf>,
    san_options: SanitizeOptions,
    san_report: Option<SanitizeReport>,

    // Metadata State
    meta_file: Option<PathBuf>,
    metadata: PdfMetadata,

    // Text & Info State
    info_file: Option<PathBuf>,
    info_details: Option<PdfDetails>,
    extracted_text: String,
    text_page_range: String,
    text_search_query: String,
}

impl Default for PdfToolkitApp {
    fn default() -> Self {
        Self {
            current_tab: Tab::Merge,
            dark_mode: true,
            sidebar_search: String::new(),
            sync_single_file: true,
            global_file: None,
            status_message: None,
            last_saved_path: None,

            merge_files: Vec::new(),
            duplex_mode: false,
            duplex_front: None,
            duplex_back: None,
            duplex_reverse_back: true,

            split_file: None,
            split_page_count: 0,
            split_mode: SplitMode::Range,
            split_range_input: "1-".to_string(),
            split_chunk_size: 2,

            org_file: None,
            org_page_count: 0,
            org_sequence_input: String::new(),
            org_blank_after_page: 1,
            org_blank_count: 1,
            org_blank_landscape: false,

            rotate_file: None,
            rotate_page_count: 0,
            rotate_pages_input: "alle".to_string(),
            rotate_angle: 90,

            delete_file: None,
            delete_page_count: 0,
            delete_pages_input: String::new(),

            wm_file: None,
            wm_page_count: 0,
            wm_options: WatermarkOptions::default(),
            wm_color_preset: 0,
            wm_pages_input: "alle".to_string(),

            pn_file: None,
            pn_page_count: 0,
            pn_options: PageNumberOptions::default(),

            cs_file: None,
            cs_page_count: 0,
            cs_mode: CropScaleMode::MarginCrop,
            cs_top_mm: 10.0,
            cs_bottom_mm: 10.0,
            cs_left_mm: 10.0,
            cs_right_mm: 10.0,
            cs_uniform_mm: true,
            cs_target_format: TargetPageFormat::A4Portrait,
            cs_pages_input: "alle".to_string(),

            compress_file: None,
            compress_max: true,
            compress_result: None,

            img_files: Vec::new(),
            img_layout: ImagePageLayout::A4Portrait,
            img_margin_pt: 24.0,

            extract_img_file: None,
            extract_img_count: None,

            sec_file: None,
            sec_user_pwd: String::new(),
            sec_owner_pwd: String::new(),
            sec_use_aes256: true,
            sec_perms: SecurityPermissions::default(),
            dec_file: None,
            dec_pwd: String::new(),

            san_file: None,
            san_options: SanitizeOptions::default(),
            san_report: None,

            meta_file: None,
            metadata: PdfMetadata::default(),

            info_file: None,
            info_details: None,
            extracted_text: String::new(),
            text_page_range: "alle".to_string(),
            text_search_query: String::new(),
        }
    }
}

impl PdfToolkitApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::apply_theme(&cc.egui_ctx, true);
        Self::default()
    }

    fn apply_theme(ctx: &egui::Context, dark: bool) {
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = Vec2::new(8.0, 8.0);
        style.spacing.button_padding = Vec2::new(12.0, 6.5);
        style.spacing.window_margin = egui::Margin::same(12);

        let mut visuals = if dark {
            let mut v = egui::Visuals::dark();
            v.panel_fill = Color32::from_rgb(15, 18, 25);
            v.window_fill = Color32::from_rgb(22, 26, 36);
            v.extreme_bg_color = Color32::from_rgb(10, 12, 18);
            v.faint_bg_color = Color32::from_rgb(24, 28, 38);
            v.selection.bg_fill = Color32::from_rgb(79, 70, 229);
            v.selection.stroke = Stroke::new(1.0_f32, Color32::from_rgb(224, 231, 255));

            v.widgets.noninteractive.bg_fill = Color32::from_rgb(24, 28, 38);
            v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(42, 49, 66));
            v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(212, 218, 230));

            v.widgets.inactive.bg_fill = Color32::from_rgb(32, 38, 52);
            v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(52, 61, 82));
            v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(226, 232, 240));

            v.widgets.hovered.bg_fill = Color32::from_rgb(45, 53, 72);
            v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(99, 102, 241));
            v.widgets.hovered.fg_stroke = Stroke::new(1.5_f32, Color32::WHITE);

            v.widgets.active.bg_fill = Color32::from_rgb(79, 70, 229);
            v.widgets.active.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(129, 140, 248));
            v.widgets.active.fg_stroke = Stroke::new(1.5_f32, Color32::WHITE);
            v
        } else {
            let mut v = egui::Visuals::light();
            v.panel_fill = Color32::from_rgb(248, 250, 252);
            v.window_fill = Color32::from_rgb(255, 255, 255);
            v.extreme_bg_color = Color32::from_rgb(255, 255, 255);
            v.faint_bg_color = Color32::from_rgb(241, 245, 249);
            v.selection.bg_fill = Color32::from_rgb(79, 70, 229);
            v.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);

            v.widgets.noninteractive.bg_fill = Color32::from_rgb(255, 255, 255);
            v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(226, 232, 240));
            v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(30, 41, 59));

            v.widgets.inactive.bg_fill = Color32::from_rgb(241, 245, 249);
            v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(203, 213, 225));
            v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(15, 23, 42));

            v.widgets.hovered.bg_fill = Color32::from_rgb(226, 232, 240);
            v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(79, 70, 229));
            v.widgets.hovered.fg_stroke = Stroke::new(1.5_f32, Color32::from_rgb(15, 23, 42));

            v.widgets.active.bg_fill = Color32::from_rgb(79, 70, 229);
            v.widgets.active.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(67, 56, 202));
            v.widgets.active.fg_stroke = Stroke::new(1.5_f32, Color32::WHITE);
            v
        };

        let radius = CornerRadius::same(8);
        visuals.widgets.noninteractive.corner_radius = radius;
        visuals.widgets.inactive.corner_radius = radius;
        visuals.widgets.hovered.corner_radius = radius;
        visuals.widgets.active.corner_radius = radius;
        visuals.widgets.open.corner_radius = radius;

        style.visuals = visuals;
        ctx.set_style(style);
    }

    fn set_status(&mut self, msg: impl Into<String>, is_error: bool) {
        self.status_message = Some((msg.into(), is_error));
    }

    fn set_saved_status(&mut self, msg: impl Into<String>, path: PathBuf) {
        self.last_saved_path = Some(path);
        self.status_message = Some((msg.into(), false));
    }

    fn apply_single_pdf_to_all(&mut self, path: PathBuf) {
        self.global_file = Some(path.clone());
        let details = get_pdf_details(&path).ok();
        let pages = details.as_ref().map(|d| d.page_count).unwrap_or(0);

        if pages > 0 {
            self.split_page_count = pages;
            self.split_range_input = format!("1-{pages}");
            self.split_file = Some(path.clone());

            self.org_page_count = pages;
            self.org_sequence_input = format!("1-{pages}");
            self.org_blank_after_page = pages as u32;
            self.org_file = Some(path.clone());

            self.rotate_page_count = pages;
            self.rotate_file = Some(path.clone());

            self.delete_page_count = pages;
            self.delete_file = Some(path.clone());

            self.wm_page_count = pages;
            self.wm_file = Some(path.clone());

            self.pn_page_count = pages;
            self.pn_file = Some(path.clone());

            self.cs_page_count = pages;
            self.cs_file = Some(path.clone());
        }

        self.compress_file = Some(path.clone());
        self.compress_result = None;

        self.extract_img_file = Some(path.clone());
        self.extract_img_count = None;

        self.sec_file = Some(path.clone());
        self.dec_file = Some(path.clone());

        self.san_file = Some(path.clone());
        self.san_report = None;

        if let Ok(m) = read_metadata(&path) {
            self.metadata = m;
            self.meta_file = Some(path.clone());
        }

        if let Some(d) = details {
            self.info_details = Some(d);
            self.info_file = Some(path);
            self.extracted_text.clear();
        }
    }

    fn load_pdf_into_active_tab(&mut self, path: PathBuf) {
        if self.current_tab == Tab::Merge {
            if self.duplex_mode {
                if self.duplex_front.is_none() {
                    self.duplex_front = Some(path);
                } else {
                    self.duplex_back = Some(path);
                }
            } else {
                self.merge_files.push(path);
            }
            return;
        }

        if self.sync_single_file {
            self.apply_single_pdf_to_all(path);
            return;
        }

        self.global_file = Some(path.clone());
        match self.current_tab {
            Tab::Split => {
                if let Ok(d) = get_pdf_details(&path) {
                    self.split_page_count = d.page_count;
                    self.split_range_input = format!("1-{}", d.page_count);
                    self.split_file = Some(path);
                }
            }
            Tab::Organize => {
                if let Ok(d) = get_pdf_details(&path) {
                    self.org_page_count = d.page_count;
                    self.org_sequence_input = format!("1-{}", d.page_count);
                    self.org_blank_after_page = d.page_count as u32;
                    self.org_file = Some(path);
                }
            }
            Tab::Rotate => {
                if let Ok(d) = get_pdf_details(&path) {
                    self.rotate_page_count = d.page_count;
                    self.rotate_file = Some(path);
                }
            }
            Tab::Delete => {
                if let Ok(d) = get_pdf_details(&path) {
                    self.delete_page_count = d.page_count;
                    self.delete_file = Some(path);
                }
            }
            Tab::Watermark => {
                if let Ok(d) = get_pdf_details(&path) {
                    self.wm_page_count = d.page_count;
                    self.wm_file = Some(path);
                }
            }
            Tab::PageNumbers => {
                if let Ok(d) = get_pdf_details(&path) {
                    self.pn_page_count = d.page_count;
                    self.pn_file = Some(path);
                }
            }
            Tab::CropScale => {
                if let Ok(d) = get_pdf_details(&path) {
                    self.cs_page_count = d.page_count;
                    self.cs_file = Some(path);
                }
            }
            Tab::Compress => {
                self.compress_file = Some(path);
                self.compress_result = None;
            }
            Tab::ExtractImages => {
                self.extract_img_file = Some(path);
                self.extract_img_count = None;
            }
            Tab::Security => {
                self.sec_file = Some(path.clone());
                self.dec_file = Some(path);
            }
            Tab::Sanitize => {
                self.san_file = Some(path);
                self.san_report = None;
            }
            Tab::Metadata => {
                if let Ok(m) = read_metadata(&path) {
                    self.metadata = m;
                    self.meta_file = Some(path);
                }
            }
            Tab::TextInfo => {
                if let Ok(d) = get_pdf_details(&path) {
                    self.info_details = Some(d);
                    self.info_file = Some(path);
                    self.extracted_text.clear();
                }
            }
            Tab::Merge | Tab::ImagesToPdf => {}
        }
    }

    fn render_sidebar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let dark = self.dark_mode;
        let accent = accent_color(dark);

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            egui::Frame::new()
                .fill(accent)
                .corner_radius(CornerRadius::same(8))
                .inner_margin(egui::Margin::symmetric(8, 5))
                .show(ui, |ui| {
                    ui.label(RichText::new("PDF").strong().color(Color32::WHITE).size(13.5));
                });
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.label(RichText::new("Werkzeugkasten").strong().size(16.0));
                ui.label(RichText::new("Pro Edition • 100% Lokal").small().weak());
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let theme_icon = if self.dark_mode { "☀️" } else { "🌙" };
                if ui
                    .button(RichText::new(theme_icon).size(14.0))
                    .on_hover_text("Design umschalten (Dunkel / Hell)")
                    .clicked()
                {
                    self.dark_mode = !self.dark_mode;
                    Self::apply_theme(ctx, self.dark_mode);
                }
            });
        });

        ui.add_space(10.0);

        let search_empty = self.sidebar_search.is_empty();
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.sidebar_search)
                    .hint_text("🔎 Werkzeug suchen...")
                    .desired_width(ui.available_width() - if search_empty { 0.0 } else { 28.0 }),
            );
            if !self.sidebar_search.is_empty() && ui.small_button("✖").clicked() {
                self.sidebar_search.clear();
            }
        });

        ui.add_space(6.0);
        ui.separator();

        let query = self.sidebar_search.trim().to_lowercase();

        ScrollArea::vertical().show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 3.0;

            let categories: &[(&str, &[Tab])] = &[
                (
                    "SEITEN & STRUKTUR",
                    &[Tab::Merge, Tab::Split, Tab::Organize, Tab::Rotate, Tab::Delete],
                ),
                (
                    "STEMPEL & LAYOUT",
                    &[Tab::Watermark, Tab::PageNumbers, Tab::CropScale],
                ),
                (
                    "BILDER & GRÖSSE",
                    &[Tab::Compress, Tab::ImagesToPdf, Tab::ExtractImages],
                ),
                (
                    "SCHUTZ & ANALYSE",
                    &[Tab::Security, Tab::Sanitize, Tab::Metadata, Tab::TextInfo],
                ),
            ];

            for (cat_name, tabs) in categories {
                let matching_tabs: Vec<Tab> = tabs
                    .iter()
                    .copied()
                    .filter(|t| {
                        if query.is_empty() {
                            return true;
                        }
                        let (_, title, _, desc) = t.info();
                        title.to_lowercase().contains(&query) || desc.to_lowercase().contains(&query)
                    })
                    .collect();

                if matching_tabs.is_empty() {
                    continue;
                }

                ui.add_space(6.0);
                ui.label(
                    RichText::new(*cat_name)
                        .size(11.0)
                        .strong()
                        .color(if dark {
                            Color32::from_rgb(120, 132, 158)
                        } else {
                            Color32::from_rgb(100, 116, 139)
                        }),
                );
                ui.add_space(2.0);

                for tab in matching_tabs {
                    let (icon, title, _, _) = tab.info();
                    let is_selected = self.current_tab == tab;

                    let badge = match tab {
                        Tab::Merge if !self.merge_files.is_empty() => {
                            Some(format!("{}", self.merge_files.len()))
                        }
                        Tab::ImagesToPdf if !self.img_files.is_empty() => {
                            Some(format!("{}", self.img_files.len()))
                        }
                        _ => None,
                    };

                    let desired_size = Vec2::new(ui.available_width(), 32.0);
                    let (rect, resp) = ui.allocate_exact_size(desired_size, Sense::click());

                    if resp.clicked() {
                        self.current_tab = tab;
                    }

                    let bg_fill = if is_selected {
                        accent.gamma_multiply(if dark { 0.22 } else { 0.14 })
                    } else if resp.hovered() {
                        if dark {
                            Color32::from_rgb(28, 34, 48)
                        } else {
                            Color32::from_rgb(226, 232, 240)
                        }
                    } else {
                        Color32::TRANSPARENT
                    };

                    if bg_fill != Color32::TRANSPARENT {
                        ui.painter().rect_filled(rect, CornerRadius::same(7), bg_fill);
                    }

                    if is_selected {
                        let bar_rect = Rect::from_min_max(
                            Pos2::new(rect.left(), rect.top() + 5.0),
                            Pos2::new(rect.left() + 3.5, rect.bottom() - 5.0),
                        );
                        ui.painter()
                            .rect_filled(bar_rect, CornerRadius::same(2), accent);
                    }

                    let text_col = if is_selected {
                        if dark {
                            Color32::WHITE
                        } else {
                            accent
                        }
                    } else if dark {
                        Color32::from_rgb(203, 213, 225)
                    } else {
                        Color32::from_rgb(51, 65, 85)
                    };

                    ui.painter().text(
                        Pos2::new(rect.left() + 12.0, rect.center().y),
                        Align2::LEFT_CENTER,
                        format!("{icon}  {title}"),
                        FontId::proportional(13.5),
                        text_col,
                    );

                    if let Some(b) = badge {
                        let pill_rect = Rect::from_center_size(
                            Pos2::new(rect.right() - 18.0, rect.center().y),
                            Vec2::new(22.0, 18.0),
                        );
                        ui.painter()
                            .rect_filled(pill_rect, CornerRadius::same(9), accent);
                        ui.painter().text(
                            pill_rect.center(),
                            Align2::CENTER_CENTER,
                            b,
                            FontId::proportional(11.0),
                            Color32::WHITE,
                        );
                    }
                }
            }
            ui.add_space(10.0);
        });
    }

    fn render_workspace_header(&mut self, ui: &mut egui::Ui) {
        let dark = self.dark_mode;
        let (icon, title, category, desc) = self.current_tab.info();
        let accent = accent_color(dark);

        egui::Frame::new()
            .fill(card_bg(dark))
            .stroke(Stroke::new(1.0_f32, card_border(dark)))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(egui::Margin::symmetric(18, 14))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.horizontal(|ui| {
                            egui::Frame::new()
                                .fill(accent.gamma_multiply(0.16))
                                .corner_radius(CornerRadius::same(5))
                                .inner_margin(egui::Margin::symmetric(7, 2))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(category)
                                            .size(10.5)
                                            .strong()
                                            .color(accent),
                                    );
                                });
                            if self.current_tab != Tab::Merge && self.current_tab != Tab::ImagesToPdf {
                                ui.checkbox(
                                    &mut self.sync_single_file,
                                    RichText::new("🔗 Datei-Sync über alle Werkzeuge").small().weak(),
                                )
                                .on_hover_text(
                                    "Wenn aktiv, bleibt eine geladene PDF beim Wechsel zwischen den Werkzeugen automatisch geladen.",
                                );
                            }
                        });
                        ui.heading(RichText::new(format!("{icon} {title}")).size(20.0).strong());
                        ui.label(RichText::new(desc).weak().size(13.0));
                    });
                });
            });
        ui.add_space(10.0);
    }

    fn render_merge_tab(&mut self, ui: &mut egui::Ui) {
        let dark = self.dark_mode;
        let accent = accent_color(dark);

        ui_card(ui, dark, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.duplex_mode,
                    false,
                    "📚 Standard-Zusammenführung (Beliebig viele PDFs)",
                );
                ui.selectable_value(
                    &mut self.duplex_mode,
                    true,
                    "🔄 Duplex-Scanner Mischer (Vorder- & Rückseiten)",
                );
            });
        });

        ui.add_space(10.0);

        if !self.duplex_mode {
            ui_card(ui, dark, |ui| {
                ui.horizontal(|ui| {
                    let add_btn = egui::Button::new(
                        RichText::new("➕ PDFs hinzufügen...")
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(accent);
                    if ui.add(add_btn).clicked() {
                        if let Some(files) = rfd::FileDialog::new()
                            .add_filter("PDF Dokumente", &["pdf"])
                            .pick_files()
                        {
                            self.merge_files.extend(files);
                        }
                    }
                    if self.merge_files.len() >= 2 && ui.button("🔃 Reihenfolge umkehren").clicked() {
                        self.merge_files.reverse();
                    }
                    if self.merge_files.len() >= 2 && ui.button("🔤 A–Z sortieren").clicked() {
                        self.merge_files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
                    }
                    if !self.merge_files.is_empty() && ui.button("🗑️ Liste leeren").clicked() {
                        self.merge_files.clear();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("{} Dateien ausgewählt", self.merge_files.len()))
                                .weak(),
                        );
                    });
                });

                ui.add_space(8.0);

                ScrollArea::vertical().max_height(310.0).show(ui, |ui| {
                    if self.merge_files.is_empty() {
                        egui::Frame::new()
                            .fill(subtle_bg(dark))
                            .stroke(Stroke::new(1.0_f32, card_border(dark)))
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(egui::Margin::symmetric(16, 28))
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.vertical_centered(|ui| {
                                    ui.label(RichText::new("📂 Noch keine PDFs in der Stapelliste").size(15.0).strong());
                                    ui.label(
                                        RichText::new("Klicke auf '➕ PDFs hinzufügen...' oder ziehe mehrere PDFs per Drag & Drop in dieses Fenster.")
                                            .weak(),
                                    );
                                });
                            });
                    } else {
                        let mut to_remove = None;
                        let mut move_up = None;
                        let mut move_down = None;

                        for (idx, file) in self.merge_files.iter().enumerate() {
                            egui::Frame::new()
                                .fill(subtle_bg(dark))
                                .corner_radius(CornerRadius::same(7))
                                .inner_margin(egui::Margin::symmetric(12, 8))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(format!("#{}", idx + 1))
                                                .strong()
                                                .color(accent),
                                        );
                                        let name = file
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("PDF");
                                        ui.label(RichText::new(name).strong());

                                        let size_kb = std::fs::metadata(file)
                                            .map(|m| m.len() as f64 / 1024.0)
                                            .unwrap_or(0.0);
                                        ui.label(RichText::new(format!("({size_kb:.1} KB)")).weak().small());

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui
                                                    .small_button("✖")
                                                    .on_hover_text("Entfernen")
                                                    .clicked()
                                                {
                                                    to_remove = Some(idx);
                                                }
                                                if idx + 1 < self.merge_files.len()
                                                    && ui
                                                        .small_button("⬇")
                                                        .on_hover_text("Nach unten")
                                                        .clicked()
                                                {
                                                    move_down = Some(idx);
                                                }
                                                if idx > 0
                                                    && ui
                                                        .small_button("⬆")
                                                        .on_hover_text("Nach oben")
                                                        .clicked()
                                                {
                                                    move_up = Some(idx);
                                                }
                                            },
                                        );
                                    });
                                });
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
            });

            ui.add_space(12.0);
            if primary_button(
                ui,
                dark,
                self.merge_files.len() >= 2,
                "💾 PDFs zusammenführen & speichern...",
            ) {
                if let Some(dest) = rfd::FileDialog::new()
                    .add_filter("PDF Dokument", &["pdf"])
                    .set_file_name("zusammengefuehrt.pdf")
                    .save_file()
                {
                    match merge_pdfs(&self.merge_files, &dest) {
                        Ok(pages) => self.set_saved_status(
                            format!("Erfolgreich zusammengeführt ({pages} Seiten): {}", dest.display()),
                            dest,
                        ),
                        Err(e) => self.set_status(e, true),
                    }
                }
            }
        } else {
            ui_card(ui, dark, |ui| {
                ui.label(
                    RichText::new(
                        "Perfekt für Einzugsscanner ohne Duplex-Einheit: Erst alle Vorderseiten scannen, dann den Stapel umdrehen und alle Rückseiten scannen.",
                    )
                    .weak(),
                );
                ui.add_space(8.0);

                let (picked_f, clear_f) = render_file_dropzone(
                    ui,
                    dark,
                    &self.duplex_front,
                    None,
                    "📂 1. Vorderseiten-PDF wählen (1, 3, 5...)",
                );
                if clear_f {
                    self.duplex_front = None;
                } else if let Some(f) = picked_f {
                    self.duplex_front = Some(f);
                }

                ui.add_space(6.0);
                let (picked_b, clear_b) = render_file_dropzone(
                    ui,
                    dark,
                    &self.duplex_back,
                    None,
                    "📂 2. Rückseiten-PDF wählen (2, 4, 6...)",
                );
                if clear_b {
                    self.duplex_back = None;
                } else if let Some(f) = picked_b {
                    self.duplex_back = Some(f);
                }

                ui.add_space(8.0);
                ui.checkbox(
                    &mut self.duplex_reverse_back,
                    "Rückseiten-PDF automatisch rückwärts einmischen (wenn Stapel umgedreht gescannt wurde: N..4, 2)",
                );
            });

            ui.add_space(12.0);
            let ready = self.duplex_front.is_some() && self.duplex_back.is_some();
            if primary_button(
                ui,
                dark,
                ready,
                "🔄 Alternierend zusammenführen & speichern...",
            ) {
                if let Some(dest) = rfd::FileDialog::new()
                    .add_filter("PDF Dokument", &["pdf"])
                    .set_file_name("duplex_komplett.pdf")
                    .save_file()
                {
                    let front = self.duplex_front.as_ref().unwrap();
                    let back = self.duplex_back.as_ref().unwrap();
                    match interleave_pdfs(front, back, self.duplex_reverse_back, &dest) {
                        Ok(pages) => self.set_saved_status(
                            format!("Duplex-Zusammenführung erfolgreich ({pages} Seiten): {}", dest.display()),
                            dest,
                        ),
                        Err(e) => self.set_status(e, true),
                    }
                }
            }
        }
    }

    fn render_split_tab(&mut self, ui: &mut egui::Ui) {
        let dark = self.dark_mode;
        let accent = accent_color(dark);

        let (picked, cleared) = render_file_dropzone(
            ui,
            dark,
            &self.split_file,
            if self.split_file.is_some() {
                Some(self.split_page_count)
            } else {
                None
            },
            "📂 PDF auswählen...",
        );
        if cleared {
            self.split_file = None;
        } else if let Some(file) = picked {
            self.load_pdf_into_active_tab(file);
        }

        if self.split_file.is_some() {
            ui.add_space(10.0);
            let page_count = self.split_page_count;

            ui_card(ui, dark, |ui| {
                ui.label(RichText::new("Teilungs-Modus wählen:").strong());
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.split_mode,
                        SplitMode::Range,
                        "🎯 Bestimmte Seiten extrahieren",
                    );
                    ui.selectable_value(
                        &mut self.split_mode,
                        SplitMode::Chunks,
                        "📦 Alle N Seiten in Teil-PDFs",
                    );
                    ui.selectable_value(
                        &mut self.split_mode,
                        SplitMode::SinglePages,
                        "📄 Jede Seite einzeln speichern",
                    );
                });
                ui.separator();

                match self.split_mode {
                    SplitMode::Range => {
                        ui.label("Seitenbereich (z.B. '1-3, 5, 8-' oder 'gerade' / 'ungerade'):");
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.split_range_input);
                            if ui.small_button("Alle").clicked() {
                                self.split_range_input = format!("1-{page_count}");
                            }
                            if ui.small_button("Ungerade").clicked() {
                                self.split_range_input = "ungerade".to_string();
                            }
                            if ui.small_button("Gerade").clicked() {
                                self.split_range_input = "gerade".to_string();
                            }
                            if ui.small_button("1. Hälfte").clicked() {
                                let half = (page_count / 2).max(1);
                                self.split_range_input = format!("1-{half}");
                            }
                            if ui.small_button("2. Hälfte").clicked() {
                                let half = (page_count / 2) + 1;
                                self.split_range_input = format!("{half}-");
                            }
                        });

                        render_interactive_page_pills(
                            ui,
                            dark,
                            page_count,
                            &mut self.split_range_input,
                            accent,
                            "extrahiert",
                        );
                    }
                    SplitMode::Chunks => {
                        ui.horizontal(|ui| {
                            ui.label("Seiten pro Teil-PDF:");
                            ui.add(egui::DragValue::new(&mut self.split_chunk_size).range(1..=500));
                            let resulting_files =
                                (page_count + self.split_chunk_size - 1) / self.split_chunk_size.max(1);
                            ui.label(
                                RichText::new(format!("→ ergibt {resulting_files} PDF-Dateien"))
                                    .color(accent)
                                    .strong(),
                            );
                        });
                    }
                    SplitMode::SinglePages => {
                        ui.label(
                            RichText::new(format!(
                                "Erzeugt {page_count} einzelne PDF-Dateien im gewählten Zielordner."
                            ))
                            .weak(),
                        );
                    }
                }
            });

            ui.add_space(12.0);
            if primary_button(ui, dark, true, "✂️ Jetzt teilen / extrahieren...") {
                let input = self.split_file.as_ref().unwrap().clone();
                match self.split_mode {
                    SplitMode::SinglePages => {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            match split_all_pages(&input, &folder) {
                                Ok(count) => self.set_saved_status(
                                    format!("{count} Einzelseiten in '{}' gespeichert", folder.display()),
                                    folder,
                                ),
                                Err(e) => self.set_status(e, true),
                            }
                        }
                    }
                    SplitMode::Chunks => {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            match split_by_chunks(&input, self.split_chunk_size, &folder) {
                                Ok(count) => self.set_saved_status(
                                    format!("{count} Teil-PDFs in '{}' erstellt", folder.display()),
                                    folder,
                                ),
                                Err(e) => self.set_status(e, true),
                            }
                        }
                    }
                    SplitMode::Range => {
                        match parse_page_ranges(&self.split_range_input, self.split_page_count as u32) {
                            Ok(pages) => {
                                if let Some(dest) = rfd::FileDialog::new()
                                    .add_filter("PDF", &["pdf"])
                                    .set_file_name("extrahiert.pdf")
                                    .save_file()
                                {
                                    match extract_pages(&input, &pages, &dest) {
                                        Ok(n) => self.set_saved_status(
                                            format!("{n} Seiten erfolgreich extrahiert: {}", dest.display()),
                                            dest,
                                        ),
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
    }

    fn render_organize_tab(&mut self, ui: &mut egui::Ui) {
        let dark = self.dark_mode;
        let accent = accent_color(dark);

        let (picked, cleared) = render_file_dropzone(
            ui,
            dark,
            &self.org_file,
            if self.org_file.is_some() {
                Some(self.org_page_count)
            } else {
                None
            },
            "📂 PDF auswählen...",
        );
        if cleared {
            self.org_file = None;
        } else if let Some(file) = picked {
            self.load_pdf_into_active_tab(file);
        }

        if self.org_file.is_some() {
            ui.add_space(10.0);
            let page_count = self.org_page_count;

            let mut save_reorder = false;
            let mut save_reverse = false;

            ui_card(ui, dark, |ui| {
                ui.heading(RichText::new("🔀 1. Seiten neu sortieren, duplizieren & umkehren").size(15.5));
                ui.label("Gib die gewünschte Reihenfolge ein oder verschiebe Seiten direkt auf den visuellen Kacheln:");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.org_sequence_input);
                    if ui.small_button("🔄 Rückwärts (N..1)").clicked() {
                        self.org_sequence_input = format!("{page_count}-1");
                    }
                    if ui.small_button("1. Seite verdoppeln").clicked() {
                        self.org_sequence_input = format!("1, 1-{page_count}");
                    }
                    if ui.small_button("Zurücksetzen").clicked() {
                        self.org_sequence_input = format!("1-{page_count}");
                    }
                });

                if let Ok(mut seq) = parse_page_sequence(&self.org_sequence_input, page_count as u32) {
                    if seq.len() <= 80 {
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(format!(
                                "Visuelle Seiten-Reihenfolge ({} Seiten im Zieldokument):",
                                seq.len()
                            ))
                            .small()
                            .weak(),
                        );
                        let mut changed = false;
                        let mut swap_pair = None;
                        let mut dup_idx = None;
                        let mut rem_idx = None;

                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                            for (idx, &p) in seq.iter().enumerate() {
                                egui::Frame::new()
                                    .fill(subtle_bg(dark))
                                    .stroke(Stroke::new(1.0_f32, card_border(dark)))
                                    .corner_radius(CornerRadius::same(6))
                                    .inner_margin(egui::Margin::symmetric(6, 4))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing.x = 2.0;
                                            if idx > 0
                                                && ui
                                                    .small_button("◀")
                                                    .on_hover_text("Nach links schieben")
                                                    .clicked()
                                            {
                                                swap_pair = Some((idx, idx - 1));
                                            }
                                            ui.label(
                                                RichText::new(format!("S.{p}"))
                                                    .strong()
                                                    .color(accent),
                                            );
                                            if idx + 1 < seq.len()
                                                && ui
                                                    .small_button("▶")
                                                    .on_hover_text("Nach rechts schieben")
                                                    .clicked()
                                            {
                                                swap_pair = Some((idx, idx + 1));
                                            }
                                            if ui
                                                .small_button("➕")
                                                .on_hover_text("Diese Seite duplizieren")
                                                .clicked()
                                            {
                                                dup_idx = Some(idx);
                                            }
                                            if seq.len() > 1
                                                && ui
                                                    .small_button("✖")
                                                    .on_hover_text("Aus Reihenfolge entfernen")
                                                    .clicked()
                                            {
                                                rem_idx = Some(idx);
                                            }
                                        });
                                    });
                            }
                        });

                        if let Some((a, b)) = swap_pair {
                            seq.swap(a, b);
                            changed = true;
                        } else if let Some(i) = dup_idx {
                            let val = seq[i];
                            seq.insert(i + 1, val);
                            changed = true;
                        } else if let Some(i) = rem_idx {
                            seq.remove(i);
                            changed = true;
                        }

                        if changed {
                            self.org_sequence_input = seq
                                .iter()
                                .map(|n| n.to_string())
                                .collect::<Vec<_>>()
                                .join(", ");
                        }
                    }
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if primary_button(ui, dark, true, "💾 Neue Reihenfolge speichern...") {
                        save_reorder = true;
                    }
                    if ui
                        .add(
                            egui::Button::new("🔃 Komplett umkehren & speichern...")
                                .min_size(Vec2::new(220.0, 38.0)),
                        )
                        .clicked()
                    {
                        save_reverse = true;
                    }
                });
            });

            if save_reorder {
                let input = self.org_file.as_ref().unwrap().clone();
                match parse_page_sequence(&self.org_sequence_input, self.org_page_count as u32) {
                    Ok(seq) => {
                        if let Some(dest) = rfd::FileDialog::new()
                            .add_filter("PDF", &["pdf"])
                            .set_file_name("sortiert.pdf")
                            .save_file()
                        {
                            match reorder_pages(&input, &seq, &dest) {
                                Ok(n) => self.set_saved_status(
                                    format!(
                                        "PDF mit {n} Seiten in neuer Reihenfolge gespeichert: {}",
                                        dest.display()
                                    ),
                                    dest,
                                ),
                                Err(e) => self.set_status(e, true),
                            }
                        }
                    }
                    Err(e) => self.set_status(e, true),
                }
            }

            if save_reverse {
                let input = self.org_file.as_ref().unwrap().clone();
                if let Some(dest) = rfd::FileDialog::new()
                    .add_filter("PDF", &["pdf"])
                    .set_file_name("umgekehrt.pdf")
                    .save_file()
                {
                    match reverse_pages(&input, &dest) {
                        Ok(n) => self.set_saved_status(
                            format!("{n} Seiten in umgekehrter Reihenfolge gespeichert: {}", dest.display()),
                            dest,
                        ),
                        Err(e) => self.set_status(e, true),
                    }
                }
            }

            ui.add_space(12.0);
            let mut insert_blanks = false;
            ui_card(ui, dark, |ui| {
                ui.heading(RichText::new("📄 2. Leerseiten einfügen").size(15.5));
                ui.horizontal(|ui| {
                    ui.label("Anzahl Leerseiten:");
                    ui.add(egui::DragValue::new(&mut self.org_blank_count).range(1..=50));
                    ui.add_space(12.0);
                    ui.label("Einfügen nach Seite (0 = ganz am Anfang):");
                    ui.add(
                        egui::DragValue::new(&mut self.org_blank_after_page)
                            .range(0..=(self.org_page_count as u32)),
                    );
                });
                ui.checkbox(&mut self.org_blank_landscape, "DIN A4 Querformat statt Hochformat");

                ui.add_space(6.0);
                if ui
                    .add(
                        egui::Button::new(RichText::new("➕ Leerseiten einfügen & speichern...").strong())
                            .min_size(Vec2::new(250.0, 34.0)),
                    )
                    .clicked()
                {
                    insert_blanks = true;
                }
            });

            if insert_blanks {
                let input = self.org_file.as_ref().unwrap().clone();
                let (w, h) = if self.org_blank_landscape {
                    (841.89, 595.28)
                } else {
                    (595.28, 841.89)
                };
                if let Some(dest) = rfd::FileDialog::new()
                    .add_filter("PDF", &["pdf"])
                    .set_file_name("mit_leerseiten.pdf")
                    .save_file()
                {
                    match insert_blank_pages(
                        &input,
                        self.org_blank_after_page,
                        self.org_blank_count,
                        w,
                        h,
                        &dest,
                    ) {
                        Ok(total) => self.set_saved_status(
                            format!("Leerseiten eingefügt (Gesamt {total} Seiten): {}", dest.display()),
                            dest,
                        ),
                        Err(e) => self.set_status(e, true),
                    }
                }
            }
        }
    }

    fn render_rotate_tab(&mut self, ui: &mut egui::Ui) {
        let dark = self.dark_mode;
        let accent = accent_color(dark);

        let (picked, cleared) = render_file_dropzone(
            ui,
            dark,
            &self.rotate_file,
            if self.rotate_file.is_some() {
                Some(self.rotate_page_count)
            } else {
                None
            },
            "📂 PDF auswählen...",
        );
        if cleared {
            self.rotate_file = None;
        } else if let Some(file) = picked {
            self.load_pdf_into_active_tab(file);
        }

        if self.rotate_file.is_some() {
            ui.add_space(10.0);
            let page_count = self.rotate_page_count;

            ui_card(ui, dark, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Drehwinkel:").strong());
                    ui.selectable_value(&mut self.rotate_angle, 90, "↻ 90° (Im Uhrzeigersinn)");
                    ui.selectable_value(&mut self.rotate_angle, 180, "↕ 180° (Kopfstand)");
                    ui.selectable_value(&mut self.rotate_angle, 270, "↺ 270° (Gegen Uhrzeigersinn)");
                });

                ui.add_space(8.0);
                ui.label("Betroffene Seiten ('alle', 'gerade', 'ungerade' oder z.B. '1, 3, 5-7'):");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.rotate_pages_input);
                    if ui.small_button("Alle").clicked() {
                        self.rotate_pages_input = "alle".to_string();
                    }
                    if ui.small_button("Ungerade").clicked() {
                        self.rotate_pages_input = "ungerade".to_string();
                    }
                    if ui.small_button("Gerade").clicked() {
                        self.rotate_pages_input = "gerade".to_string();
                    }
                });

                render_interactive_page_pills(
                    ui,
                    dark,
                    page_count,
                    &mut self.rotate_pages_input,
                    accent,
                    "gedreht",
                );
            });

            ui.add_space(12.0);
            if primary_button(ui, dark, true, "🔄 Seiten drehen & speichern...") {
                let input = self.rotate_file.as_ref().unwrap().clone();
                let pages = match parse_page_ranges(&self.rotate_pages_input, self.rotate_page_count as u32) {
                    Ok(p) => Some(p),
                    Err(e) => {
                        self.set_status(e, true);
                        return;
                    }
                };

                if let Some(dest) = rfd::FileDialog::new()
                    .add_filter("PDF", &["pdf"])
                    .set_file_name("gedreht.pdf")
                    .save_file()
                {
                    match rotate_pdf(&input, pages.as_deref(), self.rotate_angle, &dest) {
                        Ok(n) => self.set_saved_status(
                            format!("{n} Seiten erfolgreich gedreht: {}", dest.display()),
                            dest,
                        ),
                        Err(e) => self.set_status(e, true),
                    }
                }
            }
        }
    }

    fn render_delete_tab(&mut self, ui: &mut egui::Ui) {
        let dark = self.dark_mode;

        let (picked, cleared) = render_file_dropzone(
            ui,
            dark,
            &self.delete_file,
            if self.delete_file.is_some() {
                Some(self.delete_page_count)
            } else {
                None
            },
            "📂 PDF auswählen...",
        );
        if cleared {
            self.delete_file = None;
        } else if let Some(file) = picked {
            self.load_pdf_into_active_tab(file);
        }

        if self.delete_file.is_some() {
            ui.add_space(10.0);
            let danger_red = Color32::from_rgb(225, 29, 72);
            let page_count = self.delete_page_count;

            ui_card(ui, dark, |ui| {
                ui.label("Zu löschende Seiten (z.B. '2, 4-6' oder 'gerade' / 'ungerade'):");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.delete_pages_input);
                    if ui.small_button("1. Seite").clicked() {
                        self.delete_pages_input = "1".to_string();
                    }
                    if ui.small_button("Letzte Seite").clicked() {
                        self.delete_pages_input = format!("{page_count}");
                    }
                    if ui.small_button("Gerade Seiten").clicked() {
                        self.delete_pages_input = "gerade".to_string();
                    }
                    if ui.small_button("Auswahl leeren").clicked() {
                        self.delete_pages_input.clear();
                    }
                });

                render_interactive_page_pills(
                    ui,
                    dark,
                    page_count,
                    &mut self.delete_pages_input,
                    danger_red,
                    "zum Löschen markiert",
                );
            });

            ui.add_space(12.0);
            let has_selection = !self.delete_pages_input.trim().is_empty();
            if primary_button(
                ui,
                dark,
                has_selection,
                "🗑️ Markierte Seiten löschen & speichern...",
            ) {
                let input = self.delete_file.as_ref().unwrap().clone();
                match parse_page_ranges(&self.delete_pages_input, self.delete_page_count as u32) {
                    Ok(to_del) => {
                        if let Some(dest) = rfd::FileDialog::new()
                            .add_filter("PDF", &["pdf"])
                            .set_file_name("bereinigt.pdf")
                            .save_file()
                        {
                            match delete_pages_from_pdf(&input, &to_del, &dest) {
                                Ok(rem) => self.set_saved_status(
                                    format!(
                                        "{} Seiten gelöscht, {rem} verbleibend: {}",
                                        to_del.len(),
                                        dest.display()
                                    ),
                                    dest,
                                ),
                                Err(e) => self.set_status(e, true),
                            }
                        }
                    }
                    Err(e) => self.set_status(e, true),
                }
            }
        }
    }

    fn render_watermark_tab(&mut self, ui: &mut egui::Ui) {
        let dark = self.dark_mode;

        let (picked, cleared) = render_file_dropzone(
            ui,
            dark,
            &self.wm_file,
            if self.wm_file.is_some() {
                Some(self.wm_page_count)
            } else {
                None
            },
            "📂 PDF auswählen...",
        );
        if cleared {
            self.wm_file = None;
        } else if let Some(file) = picked {
            self.load_pdf_into_active_tab(file);
        }

        if self.wm_file.is_some() {
            ui.add_space(10.0);

            ui.columns(2, |cols| {
                ui_card(&mut cols[0], dark, |ui| {
                    ui.label(RichText::new("Stempel / Wasserzeichen-Text:").strong());
                    ui.text_edit_singleline(&mut self.wm_options.text);

                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("Vorlagen:").small().weak());
                        for preset in [
                            "VERTRAULICH",
                            "ENTWURF",
                            "KOPIE",
                            "BEZAHLT",
                            "ORIGINAL",
                            "NICHT FREIGEGEBEN",
                        ] {
                            if ui.small_button(preset).clicked() {
                                self.wm_options.text = preset.to_string();
                            }
                        }
                    });

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label("Schriftgröße:");
                        ui.add(
                            egui::Slider::new(&mut self.wm_options.font_size, 16.0..=110.0)
                                .suffix(" pt"),
                        );
                    });

                    ui.add_space(4.0);
                    ui.label("Farbton:");
                    ui.horizontal_wrapped(|ui| {
                        let presets = [
                            (0, "Dezent Grau", (0.72, 0.72, 0.72)),
                            (1, "Signalrot", (0.85, 0.22, 0.22)),
                            (2, "Stempelblau", (0.18, 0.42, 0.82)),
                            (3, "Freigabegrün", (0.18, 0.65, 0.34)),
                            (4, "Anthrazit", (0.25, 0.25, 0.25)),
                        ];
                        for (idx, label, rgb) in presets {
                            if ui
                                .selectable_label(self.wm_color_preset == idx, label)
                                .clicked()
                            {
                                self.wm_color_preset = idx;
                                self.wm_options.color_rgb = rgb;
                            }
                        }
                    });

                    ui.add_space(4.0);
                    ui.label("Position:");
                    ui.horizontal_wrapped(|ui| {
                        ui.selectable_value(
                            &mut self.wm_options.position,
                            WatermarkPosition::DiagonalCenter,
                            "↗ Diagonal (45°)",
                        );
                        ui.selectable_value(
                            &mut self.wm_options.position,
                            WatermarkPosition::TopCenter,
                            "⬆ Oben Mitte",
                        );
                        ui.selectable_value(
                            &mut self.wm_options.position,
                            WatermarkPosition::Center,
                            "⏺ Zentriert",
                        );
                        ui.selectable_value(
                            &mut self.wm_options.position,
                            WatermarkPosition::BottomCenter,
                            "⬇ Unten Mitte",
                        );
                    });

                    ui.add_space(4.0);
                    ui.checkbox(
                        &mut self.wm_options.underneath,
                        "Hinter dem Seiteninhalt platzieren (Hintergrund)",
                    );

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("Seiten ('alle', '1', '1-3'):");
                        ui.text_edit_singleline(&mut self.wm_pages_input);
                    });
                });

                ui_card(&mut cols[1], dark, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("👁️ Live DIN-A4 Vorschau").strong());
                        ui.add_space(6.0);
                        let page_size = Vec2::new(190.0, 268.0);
                        let (rect, _) = ui.allocate_exact_size(page_size, Sense::hover());
                        let painter = ui.painter_at(rect);

                        painter.rect_filled(rect, CornerRadius::same(4), Color32::WHITE);
                        painter.rect_stroke(
                            rect,
                            CornerRadius::same(4),
                            Stroke::new(1.0_f32, Color32::from_rgb(180, 190, 205)),
                            StrokeKind::Outside,
                        );

                        let (r, g, b) = self.wm_options.color_rgb;
                        let wm_col = Color32::from_rgb(
                            (r * 255.0) as u8,
                            (g * 255.0) as u8,
                            (b * 255.0) as u8,
                        );

                        let draw_simulated_lines = |p: &egui::Painter| {
                            let line_col = Color32::from_rgb(218, 224, 233);
                            for i in 0..11 {
                                let y = rect.top() + 34.0 + i as f32 * 18.0;
                                let w = if i % 4 == 3 { 105.0 } else { 146.0 };
                                p.line_segment(
                                    [Pos2::new(rect.left() + 22.0, y), Pos2::new(rect.left() + 22.0 + w, y)],
                                    Stroke::new(3.0_f32, line_col),
                                );
                            }
                        };

                        if !self.wm_options.underneath {
                            draw_simulated_lines(&painter);
                        }

                        let pos = match self.wm_options.position {
                            WatermarkPosition::TopCenter => Pos2::new(rect.center().x, rect.top() + 22.0),
                            WatermarkPosition::BottomCenter => {
                                Pos2::new(rect.center().x, rect.bottom() - 22.0)
                            }
                            WatermarkPosition::Center | WatermarkPosition::DiagonalCenter => {
                                rect.center()
                            }
                        };

                        let scaled_font = (self.wm_options.font_size * 0.28).clamp(8.0, 26.0);
                        painter.text(
                            pos,
                            Align2::CENTER_CENTER,
                            &self.wm_options.text,
                            FontId::proportional(scaled_font),
                            wm_col,
                        );

                        if self.wm_options.underneath {
                            draw_simulated_lines(&painter);
                        }
                    });
                });
            });

            ui.add_space(12.0);
            if primary_button(ui, dark, true, "💧 Wasserzeichen anwenden & speichern...") {
                let input = self.wm_file.as_ref().unwrap().clone();
                let target_pages = match parse_page_ranges(&self.wm_pages_input, self.wm_page_count as u32) {
                    Ok(p) => p,
                    Err(e) => {
                        self.set_status(e, true);
                        return;
                    }
                };

                if let Some(dest) = rfd::FileDialog::new()
                    .add_filter("PDF", &["pdf"])
                    .set_file_name("wasserzeichen.pdf")
                    .save_file()
                {
                    match add_watermark(&input, &self.wm_options, Some(&target_pages), &dest) {
                        Ok(n) => self.set_saved_status(
                            format!("Wasserzeichen auf {n} Seiten eingefügt: {}", dest.display()),
                            dest,
                        ),
                        Err(e) => self.set_status(e, true),
                    }
                }
            }
        }
    }

    fn render_page_numbers_tab(&mut self, ui: &mut egui::Ui) {
        let dark = self.dark_mode;
        let accent = accent_color(dark);

        let (picked, cleared) = render_file_dropzone(
            ui,
            dark,
            &self.pn_file,
            if self.pn_file.is_some() {
                Some(self.pn_page_count)
            } else {
                None
            },
            "📂 PDF auswählen...",
        );
        if cleared {
            self.pn_file = None;
        } else if let Some(file) = picked {
            self.load_pdf_into_active_tab(file);
        }

        if self.pn_file.is_some() {
            ui.add_space(10.0);
            let total = self.pn_page_count.max(1);

            ui.columns(2, |cols| {
                ui_card(&mut cols[0], dark, |ui| {
                    ui.label(RichText::new("Nummerierungs-Format:").strong());
                    ui.horizontal_wrapped(|ui| {
                        ui.selectable_value(
                            &mut self.pn_options.format,
                            PageNumberFormat::PageOfTotal,
                            "Seite 1 von N",
                        );
                        ui.selectable_value(
                            &mut self.pn_options.format,
                            PageNumberFormat::SlashTotal,
                            "1 / N",
                        );
                        ui.selectable_value(
                            &mut self.pn_options.format,
                            PageNumberFormat::Simple,
                            "Nur Zahl (1, 2, 3)",
                        );
                        ui.selectable_value(
                            &mut self.pn_options.format,
                            PageNumberFormat::Bates,
                            "Bates (DOK-000001)",
                        );
                    });

                    if self.pn_options.format == PageNumberFormat::Bates {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label("Bates-Präfix:");
                            ui.text_edit_singleline(&mut self.pn_options.bates_prefix);
                        });
                    }

                    ui.add_space(8.0);
                    ui.label(RichText::new("Position auf der Seite:").strong());
                    egui::Grid::new("pn_pos_grid")
                        .num_columns(3)
                        .spacing([8.0, 6.0])
                        .show(ui, |ui| {
                            ui.selectable_value(
                                &mut self.pn_options.position,
                                PageNumberPosition::TopLeft,
                                "↖ Oben Links",
                            );
                            ui.selectable_value(
                                &mut self.pn_options.position,
                                PageNumberPosition::TopCenter,
                                "⬆ Oben Mitte",
                            );
                            ui.selectable_value(
                                &mut self.pn_options.position,
                                PageNumberPosition::TopRight,
                                "↗ Oben Rechts",
                            );
                            ui.end_row();

                            ui.selectable_value(
                                &mut self.pn_options.position,
                                PageNumberPosition::BottomLeft,
                                "↙ Unten Links",
                            );
                            ui.selectable_value(
                                &mut self.pn_options.position,
                                PageNumberPosition::BottomCenter,
                                "⬇ Unten Mitte",
                            );
                            ui.selectable_value(
                                &mut self.pn_options.position,
                                PageNumberPosition::BottomRight,
                                "↘ Unten Rechts",
                            );
                            ui.end_row();
                        });

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label("Startnummer:");
                        ui.add(egui::DragValue::new(&mut self.pn_options.start_number).range(1..=99999));
                        ui.add_space(10.0);
                        ui.label("Schrift:");
                        ui.add(
                            egui::DragValue::new(&mut self.pn_options.font_size)
                                .range(6.0..=36.0)
                                .suffix(" pt"),
                        );
                        ui.add_space(10.0);
                        ui.label("Rand:");
                        ui.add(
                            egui::DragValue::new(&mut self.pn_options.margin_pt)
                                .range(12.0..=100.0)
                                .suffix(" pt"),
                        );
                    });

                    ui.add_space(4.0);
                    ui.checkbox(
                        &mut self.pn_options.skip_first_page,
                        "Erste Seite (Deckblatt) nicht nummerieren",
                    );
                });

                ui_card(&mut cols[1], dark, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("👁️ Live Positions-Vorschau").strong());
                        ui.add_space(6.0);
                        let page_size = Vec2::new(190.0, 268.0);
                        let (rect, _) = ui.allocate_exact_size(page_size, Sense::hover());
                        let painter = ui.painter_at(rect);

                        painter.rect_filled(rect, CornerRadius::same(4), Color32::WHITE);
                        painter.rect_stroke(
                            rect,
                            CornerRadius::same(4),
                            Stroke::new(1.0_f32, Color32::from_rgb(180, 190, 205)),
                            StrokeKind::Outside,
                        );

                        let line_col = Color32::from_rgb(226, 232, 240);
                        for i in 0..9 {
                            let y = rect.top() + 45.0 + i as f32 * 18.0;
                            painter.line_segment(
                                [Pos2::new(rect.left() + 22.0, y), Pos2::new(rect.right() - 22.0, y)],
                                Stroke::new(3.0_f32, line_col),
                            );
                        }

                        let sample_label = match self.pn_options.format {
                            PageNumberFormat::PageOfTotal => {
                                format!("Seite {} von {}", self.pn_options.start_number, total)
                            }
                            PageNumberFormat::SlashTotal => {
                                format!("{} / {}", self.pn_options.start_number, total)
                            }
                            PageNumberFormat::Simple => format!("{}", self.pn_options.start_number),
                            PageNumberFormat::Bates => format!(
                                "{}{:06}",
                                self.pn_options.bates_prefix, self.pn_options.start_number
                            ),
                        };

                        let m = (self.pn_options.margin_pt * 0.32).clamp(6.0, 32.0);
                        let (pos, align) = match self.pn_options.position {
                            PageNumberPosition::TopLeft => (Pos2::new(rect.left() + m, rect.top() + m), Align2::LEFT_TOP),
                            PageNumberPosition::TopCenter => (Pos2::new(rect.center().x, rect.top() + m), Align2::CENTER_TOP),
                            PageNumberPosition::TopRight => (Pos2::new(rect.right() - m, rect.top() + m), Align2::RIGHT_TOP),
                            PageNumberPosition::BottomLeft => (Pos2::new(rect.left() + m, rect.bottom() - m), Align2::LEFT_BOTTOM),
                            PageNumberPosition::BottomCenter => (Pos2::new(rect.center().x, rect.bottom() - m), Align2::CENTER_BOTTOM),
                            PageNumberPosition::BottomRight => (Pos2::new(rect.right() - m, rect.bottom() - m), Align2::RIGHT_BOTTOM),
                        };

                        painter.text(
                            pos,
                            align,
                            sample_label,
                            FontId::proportional(11.5),
                            accent,
                        );
                    });
                });
            });

            ui.add_space(12.0);
            if primary_button(ui, dark, true, "🔢 Seitenzahlen einfügen & speichern...") {
                let input = self.pn_file.as_ref().unwrap().clone();
                if let Some(dest) = rfd::FileDialog::new()
                    .add_filter("PDF", &["pdf"])
                    .set_file_name("nummeriert.pdf")
                    .save_file()
                {
                    match add_page_numbers(&input, &self.pn_options, &dest) {
                        Ok(n) => self.set_saved_status(
                            format!("Seitenzahlen auf {n} Seiten eingefügt: {}", dest.display()),
                            dest,
                        ),
                        Err(e) => self.set_status(e, true),
                    }
                }
            }
        }
    }

    fn render_crop_scale_tab(&mut self, ui: &mut egui::Ui) {
        let dark = self.dark_mode;
        let accent = accent_color(dark);

        let (picked, cleared) = render_file_dropzone(
            ui,
            dark,
            &self.cs_file,
            if self.cs_file.is_some() {
                Some(self.cs_page_count)
            } else {
                None
            },
            "📂 PDF auswählen...",
        );
        if cleared {
            self.cs_file = None;
        } else if let Some(file) = picked {
            self.load_pdf_into_active_tab(file);
        }

        if self.cs_file.is_some() {
            ui.add_space(10.0);

            ui.columns(2, |cols| {
                ui_card(&mut cols[0], dark, |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut self.cs_mode,
                            CropScaleMode::MarginCrop,
                            "✂️ Ränder beschneiden (mm)",
                        );
                        ui.selectable_value(
                            &mut self.cs_mode,
                            CropScaleMode::FormatScale,
                            "📐 Auf DIN-Format skalieren",
                        );
                    });
                    ui.separator();

                    match self.cs_mode {
                        CropScaleMode::MarginCrop => {
                            ui.checkbox(
                                &mut self.cs_uniform_mm,
                                "Alle 4 Seitenränder gleichmäßig beschneiden",
                            );
                            ui.add_space(6.0);
                            if self.cs_uniform_mm {
                                ui.horizontal(|ui| {
                                    ui.label("Rand rundum:");
                                    if ui
                                        .add(
                                            egui::Slider::new(&mut self.cs_top_mm, 0.0..=70.0)
                                                .suffix(" mm"),
                                        )
                                        .changed()
                                    {
                                        self.cs_bottom_mm = self.cs_top_mm;
                                        self.cs_left_mm = self.cs_top_mm;
                                        self.cs_right_mm = self.cs_top_mm;
                                    }
                                });
                            } else {
                                egui::Grid::new("crop_grid")
                                    .num_columns(4)
                                    .spacing([12.0, 8.0])
                                    .show(ui, |ui| {
                                        ui.label("Oben:");
                                        ui.add(
                                            egui::DragValue::new(&mut self.cs_top_mm)
                                                .range(0.0..=80.0)
                                                .suffix(" mm"),
                                        );
                                        ui.label("Unten:");
                                        ui.add(
                                            egui::DragValue::new(&mut self.cs_bottom_mm)
                                                .range(0.0..=80.0)
                                                .suffix(" mm"),
                                        );
                                        ui.end_row();

                                        ui.label("Links:");
                                        ui.add(
                                            egui::DragValue::new(&mut self.cs_left_mm)
                                                .range(0.0..=80.0)
                                                .suffix(" mm"),
                                        );
                                        ui.label("Rechts:");
                                        ui.add(
                                            egui::DragValue::new(&mut self.cs_right_mm)
                                                .range(0.0..=80.0)
                                                .suffix(" mm"),
                                        );
                                        ui.end_row();
                                    });
                            }
                        }
                        CropScaleMode::FormatScale => {
                            ui.label("Zielformat wählen (Inhalt wird proportional eingepasst):");
                            ui.add_space(6.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.selectable_value(
                                    &mut self.cs_target_format,
                                    TargetPageFormat::A4Portrait,
                                    "DIN A4 Hoch (210×297 mm)",
                                );
                                ui.selectable_value(
                                    &mut self.cs_target_format,
                                    TargetPageFormat::A4Landscape,
                                    "DIN A4 Quer (297×210 mm)",
                                );
                                ui.selectable_value(
                                    &mut self.cs_target_format,
                                    TargetPageFormat::A3Portrait,
                                    "DIN A3 (297×420 mm)",
                                );
                                ui.selectable_value(
                                    &mut self.cs_target_format,
                                    TargetPageFormat::A5Portrait,
                                    "DIN A5 (148×210 mm)",
                                );
                                ui.selectable_value(
                                    &mut self.cs_target_format,
                                    TargetPageFormat::LetterPortrait,
                                    "US Letter",
                                );
                            });
                        }
                    }

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label("Seiten ('alle' oder z.B. '1-3'):");
                        ui.text_edit_singleline(&mut self.cs_pages_input);
                    });
                });

                ui_card(&mut cols[1], dark, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("👁️ Schnittbereich-Vorschau").strong());
                        ui.add_space(6.0);
                        let page_size = Vec2::new(180.0, 254.0);
                        let (rect, _) = ui.allocate_exact_size(page_size, Sense::hover());
                        let painter = ui.painter_at(rect);

                        painter.rect_filled(rect, CornerRadius::same(4), Color32::WHITE);
                        painter.rect_stroke(
                            rect,
                            CornerRadius::same(4),
                            Stroke::new(1.0_f32, Color32::from_rgb(180, 190, 205)),
                            StrokeKind::Outside,
                        );

                        if self.cs_mode == CropScaleMode::MarginCrop {
                            let scale = 180.0 / 210.0;
                            let t = (self.cs_top_mm * scale).min(110.0);
                            let b = (self.cs_bottom_mm * scale).min(110.0);
                            let l = (self.cs_left_mm * scale).min(80.0);
                            let r = (self.cs_right_mm * scale).min(80.0);

                            let inner = Rect::from_min_max(
                                Pos2::new(rect.left() + l, rect.top() + t),
                                Pos2::new(rect.right() - r, rect.bottom() - b),
                            );

                            painter.rect_filled(
                                rect,
                                CornerRadius::same(4),
                                Color32::from_rgba_unmultiplied(225, 29, 72, 42),
                            );
                            painter.rect_filled(inner, CornerRadius::ZERO, Color32::WHITE);
                            painter.rect_stroke(
                                inner,
                                CornerRadius::ZERO,
                                Stroke::new(1.5_f32, accent),
                                StrokeKind::Inside,
                            );
                        } else {
                            painter.text(
                                rect.center(),
                                Align2::CENTER_CENTER,
                                "Proportional\nzentriert",
                                FontId::proportional(13.0),
                                accent,
                            );
                        }
                    });
                });
            });

            ui.add_space(12.0);
            if primary_button(ui, dark, true, "📐 Änderungen anwenden & speichern...") {
                let input = self.cs_file.as_ref().unwrap().clone();
                let pages = match parse_page_ranges(&self.cs_pages_input, self.cs_page_count as u32) {
                    Ok(p) => p,
                    Err(e) => {
                        self.set_status(e, true);
                        return;
                    }
                };

                if let Some(dest) = rfd::FileDialog::new()
                    .add_filter("PDF", &["pdf"])
                    .set_file_name("angepasst.pdf")
                    .save_file()
                {
                    let res = match self.cs_mode {
                        CropScaleMode::MarginCrop => crop_margins_mm(
                            &input,
                            self.cs_top_mm,
                            self.cs_bottom_mm,
                            self.cs_left_mm,
                            self.cs_right_mm,
                            Some(&pages),
                            &dest,
                        ),
                        CropScaleMode::FormatScale => {
                            scale_pages_to_format(&input, self.cs_target_format, Some(&pages), &dest)
                        }
                    };
                    match res {
                        Ok(n) => self.set_saved_status(
                            format!("{n} Seiten erfolgreich angepasst: {}", dest.display()),
                            dest,
                        ),
                        Err(e) => self.set_status(e, true),
                    }
                }
            }
        }
    }

    fn render_compress_tab(&mut self, ui: &mut egui::Ui) {
        let dark = self.dark_mode;

        let (picked, cleared) = render_file_dropzone(
            ui,
            dark,
            &self.compress_file,
            None,
            "📂 PDF auswählen...",
        );
        if cleared {
            self.compress_file = None;
            self.compress_result = None;
        } else if let Some(file) = picked {
            self.load_pdf_into_active_tab(file);
        }

        if self.compress_file.is_some() {
            ui.add_space(10.0);
            ui_card(ui, dark, |ui| {
                ui.checkbox(
                    &mut self.compress_max,
                    "Maximale Kompression (Unbenutzte Objekte bereinigen + Flate-Streams Level 9)",
                );
            });

            ui.add_space(12.0);
            if primary_button(ui, dark, true, "🗜️ Jetzt komprimieren & speichern...") {
                let input = self.compress_file.as_ref().unwrap().clone();
                if let Some(dest) = rfd::FileDialog::new()
                    .add_filter("PDF", &["pdf"])
                    .set_file_name("komprimiert.pdf")
                    .save_file()
                {
                    match compress_pdf(&input, &dest, self.compress_max) {
                        Ok(res) => {
                            self.set_saved_status(
                                format!(
                                    "Kompression fertig! {:.1} KB -> {:.1} KB ({:.1}% Ersparnis)",
                                    res.original_size as f64 / 1024.0,
                                    res.compressed_size as f64 / 1024.0,
                                    res.ratio_percent
                                ),
                                dest,
                            );
                            self.compress_result = Some(res);
                        }
                        Err(e) => self.set_status(e, true),
                    }
                }
            }

            if let Some(res) = &self.compress_result {
                let orig_size = res.original_size;
                let comp_size = res.compressed_size;
                let ratio = res.ratio_percent;
                ui.add_space(12.0);
                ui_card(ui, dark, |ui| {
                    ui.label(RichText::new("📊 Kompressions-Ergebnis").strong().size(15.0));
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(format!("Vorher: {:.2} KB", orig_size as f64 / 1024.0));
                        ui.label("→");
                        ui.label(
                            RichText::new(format!("Nachher: {:.2} KB", comp_size as f64 / 1024.0))
                                .strong(),
                        );
                        ui.label(
                            RichText::new(format!("(-{:.1}%)", ratio))
                                .color(Color32::from_rgb(34, 197, 94))
                                .strong(),
                        );
                    });
                    let frac = if orig_size > 0 {
                        (comp_size as f32 / orig_size as f32).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    ui.add(
                        egui::ProgressBar::new(frac)
                            .text(format!("{:.1}% der ursprünglichen Dateigröße", frac * 100.0)),
                    );
                });
            }
        }
    }

    fn render_images_to_pdf_tab(&mut self, ui: &mut egui::Ui) {
        let dark = self.dark_mode;
        let accent = accent_color(dark);

        ui_card(ui, dark, |ui| {
            ui.horizontal(|ui| {
                let add_btn = egui::Button::new(
                    RichText::new("➕ Bilder hinzufügen...")
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(accent);
                if ui.add(add_btn).clicked() {
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

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Seitenlayout:").strong());
                ui.selectable_value(
                    &mut self.img_layout,
                    ImagePageLayout::A4Portrait,
                    "DIN A4 Hochformat",
                );
                ui.selectable_value(
                    &mut self.img_layout,
                    ImagePageLayout::A4Landscape,
                    "DIN A4 Querformat",
                );
                ui.selectable_value(
                    &mut self.img_layout,
                    ImagePageLayout::OriginalSize,
                    "Original-Bildgröße",
                );
            });
            if self.img_layout != ImagePageLayout::OriginalSize {
                ui.horizontal(|ui| {
                    ui.label("Seitenrand:");
                    ui.add(egui::Slider::new(&mut self.img_margin_pt, 0.0..=72.0).suffix(" pt"));
                });
            }

            ui.add_space(6.0);
            ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                if self.img_files.is_empty() {
                    egui::Frame::new()
                        .fill(subtle_bg(dark))
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(egui::Margin::symmetric(16, 22))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new(
                                        "Keine Bilder ausgewählt. Klicke auf '➕ Bilder hinzufügen...' oder ziehe Bilder in das Fenster.",
                                    )
                                    .weak(),
                                );
                            });
                        });
                } else {
                    let mut to_remove = None;
                    let mut move_up = None;
                    let mut move_down = None;
                    for (idx, img) in self.img_files.iter().enumerate() {
                        egui::Frame::new()
                            .fill(subtle_bg(dark))
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(egui::Margin::symmetric(10, 6))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!("#{}", idx + 1))
                                            .strong()
                                            .color(accent),
                                    );
                                    ui.label(
                                        RichText::new(
                                            img.file_name().unwrap_or_default().to_string_lossy(),
                                        )
                                        .strong(),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.small_button("✖").clicked() {
                                                to_remove = Some(idx);
                                            }
                                            if idx + 1 < self.img_files.len()
                                                && ui.small_button("⬇").clicked()
                                            {
                                                move_down = Some(idx);
                                            }
                                            if idx > 0 && ui.small_button("⬆").clicked() {
                                                move_up = Some(idx);
                                            }
                                        },
                                    );
                                });
                            });
                    }
                    if let Some(idx) = to_remove {
                        self.img_files.remove(idx);
                    } else if let Some(idx) = move_up {
                        self.img_files.swap(idx, idx - 1);
                    } else if let Some(idx) = move_down {
                        self.img_files.swap(idx, idx + 1);
                    }
                }
            });
        });

        ui.add_space(12.0);
        if primary_button(ui, dark, !self.img_files.is_empty(), "📄 Als PDF speichern...") {
            if let Some(dest) = rfd::FileDialog::new()
                .add_filter("PDF", &["pdf"])
                .set_file_name("bilder.pdf")
                .save_file()
            {
                match convert_images_to_pdf(
                    &self.img_files,
                    self.img_layout,
                    self.img_margin_pt,
                    &dest,
                ) {
                    Ok(n) => self.set_saved_status(
                        format!("{n} Bilder erfolgreich als PDF gespeichert: {}", dest.display()),
                        dest,
                    ),
                    Err(e) => self.set_status(e, true),
                }
            }
        }
    }

    fn render_extract_images_tab(&mut self, ui: &mut egui::Ui) {
        let dark = self.dark_mode;

        let (picked, cleared) = render_file_dropzone(
            ui,
            dark,
            &self.extract_img_file,
            None,
            "📂 PDF auswählen...",
        );
        if cleared {
            self.extract_img_file = None;
            self.extract_img_count = None;
        } else if let Some(file) = picked {
            self.load_pdf_into_active_tab(file);
        }

        if self.extract_img_file.is_some() {
            ui.add_space(12.0);
            if primary_button(
                ui,
                dark,
                true,
                "🔍 Zielordner wählen & Bilder extrahieren...",
            ) {
                let input = self.extract_img_file.as_ref().unwrap().clone();
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    match extract_images_from_pdf(&input, &folder) {
                        Ok(count) => {
                            self.extract_img_count = Some(count);
                            self.set_saved_status(
                                format!("{count} Bilder extrahiert nach: {}", folder.display()),
                                folder,
                            );
                        }
                        Err(e) => self.set_status(e, true),
                    }
                }
            }

            if let Some(count) = self.extract_img_count {
                ui.add_space(10.0);
                ui_card(ui, dark, |ui| {
                    ui.label(
                        RichText::new(format!(
                            "✅ Insgesamt {count} eingebettete Bilder gefunden und exportiert."
                        ))
                        .strong(),
                    );
                });
            }
        }
    }

    fn render_security_tab(&mut self, ui: &mut egui::Ui) {
        let dark = self.dark_mode;
        let mut picked_sec = None;
        let mut clear_sec = false;
        let mut do_encrypt = false;

        ui_card(ui, dark, |ui| {
            ui.heading(RichText::new("🔒 1. PDF Verschlüsseln & Rechte setzen").size(15.5));
            let (picked, cleared) = render_file_dropzone(
                ui,
                dark,
                &self.sec_file,
                None,
                "📂 Zu schützende PDF wählen...",
            );
            picked_sec = picked;
            clear_sec = cleared;

            if self.sec_file.is_some() {
                ui.add_space(8.0);
                egui::Grid::new("sec_pwd_grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Passwort zum Öffnen (User):");
                        ui.add(egui::TextEdit::singleline(&mut self.sec_user_pwd).password(true));
                        ui.end_row();

                        ui.label("Besitzer-Passwort (Owner):");
                        ui.add(egui::TextEdit::singleline(&mut self.sec_owner_pwd).password(true));
                        ui.end_row();
                    });

                ui.add_space(6.0);
                ui.label(RichText::new("Erlaubte Aktionen für Leser:").strong());
                ui.horizontal_wrapped(|ui| {
                    ui.checkbox(&mut self.sec_perms.allow_print, "🖨️ Drucken");
                    ui.checkbox(&mut self.sec_perms.allow_copy, "📋 Text/Bilder kopieren");
                    ui.checkbox(&mut self.sec_perms.allow_modify, "✏️ Inhalt bearbeiten");
                    ui.checkbox(&mut self.sec_perms.allow_annotations, "💬 Kommentare & Formulare");
                });

                ui.add_space(8.0);
                if primary_button(ui, dark, true, "🔒 Verschlüsseln & speichern...") {
                    do_encrypt = true;
                }
            }
        });

        if clear_sec {
            self.sec_file = None;
        } else if let Some(file) = picked_sec {
            self.load_pdf_into_active_tab(file);
        }

        if do_encrypt {
            let input = self.sec_file.as_ref().unwrap().clone();
            if let Some(dest) = rfd::FileDialog::new()
                .add_filter("PDF", &["pdf"])
                .set_file_name("geschuetzt.pdf")
                .save_file()
            {
                match encrypt_pdf(
                    &input,
                    &self.sec_user_pwd,
                    &self.sec_owner_pwd,
                    self.sec_use_aes256,
                    &self.sec_perms,
                    &dest,
                ) {
                    Ok(()) => self.set_saved_status(
                        format!("PDF erfolgreich verschlüsselt: {}", dest.display()),
                        dest,
                    ),
                    Err(e) => self.set_status(e, true),
                }
            }
        }

        ui.add_space(12.0);

        let mut do_decrypt = false;
        ui_card(ui, dark, |ui| {
            ui.heading(RichText::new("🔓 2. Passwort Entfernen / Entschlüsseln").size(15.5));
            let (picked, cleared) = render_file_dropzone(
                ui,
                dark,
                &self.dec_file,
                None,
                "📂 Geschützte PDF wählen...",
            );
            if cleared {
                self.dec_file = None;
            } else if let Some(file) = picked {
                self.dec_file = Some(file);
            }

            if self.dec_file.is_some() {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label("Aktuelles Passwort:");
                    ui.add(egui::TextEdit::singleline(&mut self.dec_pwd).password(true));
                });

                ui.add_space(8.0);
                if ui
                    .add(
                        egui::Button::new(RichText::new("🔓 Entschlüsseln & speichern...").strong())
                            .min_size(Vec2::new(240.0, 36.0)),
                    )
                    .clicked()
                {
                    do_decrypt = true;
                }
            }
        });

        if do_decrypt {
            let input = self.dec_file.as_ref().unwrap().clone();
            if let Some(dest) = rfd::FileDialog::new()
                .add_filter("PDF", &["pdf"])
                .set_file_name("entschluesselt.pdf")
                .save_file()
            {
                match decrypt_pdf(&input, &self.dec_pwd, &dest) {
                    Ok(()) => self.set_saved_status(
                        format!("PDF erfolgreich entschlüsselt: {}", dest.display()),
                        dest,
                    ),
                    Err(e) => self.set_status(e, true),
                }
            }
        }
    }

    fn render_sanitize_tab(&mut self, ui: &mut egui::Ui) {
        let dark = self.dark_mode;

        let (picked, cleared) = render_file_dropzone(
            ui,
            dark,
            &self.san_file,
            None,
            "📂 PDF auswählen...",
        );
        if cleared {
            self.san_file = None;
            self.san_report = None;
        } else if let Some(file) = picked {
            self.load_pdf_into_active_tab(file);
        }

        if self.san_file.is_some() {
            ui.add_space(10.0);
            ui_card(ui, dark, |ui| {
                ui.label(RichText::new("Zu entfernende Elemente auswählen:").strong());
                ui.checkbox(
                    &mut self.san_options.remove_metadata,
                    "🕵️ Alle Dokument-Metadaten & XMP-Streams restlos löschen (Autor, Erstellungsprogramm, Historie)",
                );
                ui.checkbox(
                    &mut self.san_options.remove_annotations,
                    "💬 Alle Kommentare, Notizen, Markierungen & Hyperlinks entfernen",
                );
                ui.checkbox(
                    &mut self.san_options.remove_forms,
                    "📋 Interaktive Formularfelder (AcroForm) entfernen",
                );
                ui.checkbox(
                    &mut self.san_options.remove_scripts_actions,
                    "⚡ Eingebettetes JavaScript & automatische Öffnungs-Aktionen entfernen",
                );
                ui.checkbox(
                    &mut self.san_options.remove_outlines,
                    "🔖 Lesezeichen / Inhaltsverzeichnis (Outlines) entfernen",
                );
            });

            ui.add_space(12.0);
            if primary_button(ui, dark, true, "🛡️ PDF bereinigen & speichern...") {
                let input = self.san_file.as_ref().unwrap().clone();
                if let Some(dest) = rfd::FileDialog::new()
                    .add_filter("PDF", &["pdf"])
                    .set_file_name("bereinigt_datenschutz.pdf")
                    .save_file()
                {
                    match sanitize_pdf(&input, &self.san_options, &dest) {
                        Ok(rep) => {
                            self.set_saved_status(
                                format!(
                                    "PDF bereinigt ({} ungenutzte Objekte entfernt): {}",
                                    rep.pruned_objects,
                                    dest.display()
                                ),
                                dest,
                            );
                            self.san_report = Some(rep);
                        }
                        Err(e) => self.set_status(e, true),
                    }
                }
            }

            if let Some(rep) = self.san_report.clone() {
                ui.add_space(12.0);
                ui_card(ui, dark, |ui| {
                    ui.label(RichText::new("🛡️ Bereinigungs-Bericht").strong());
                    ui.label(format!(
                        "• Metadaten entfernt: {}",
                        if rep.metadata_cleared {
                            "Ja"
                        } else {
                            "Keine vorhanden / unverändert"
                        }
                    ));
                    ui.label(format!(
                        "• Seiten mit entfernten Annotationen: {}",
                        rep.removed_annotations_pages
                    ));
                    ui.label(format!(
                        "• Formularstrukturen entfernt: {}",
                        if rep.forms_removed { "Ja" } else { "Nein" }
                    ));
                    ui.label(format!(
                        "• Skripte/Aktionen entfernt: {}",
                        if rep.scripts_removed { "Ja" } else { "Nein" }
                    ));
                    ui.label(format!("• Bereinigte verwaiste PDF-Objekte: {}", rep.pruned_objects));
                });
            }
        }
    }

    fn render_metadata_tab(&mut self, ui: &mut egui::Ui) {
        let dark = self.dark_mode;

        let (picked, cleared) = render_file_dropzone(
            ui,
            dark,
            &self.meta_file,
            None,
            "📂 PDF auswählen...",
        );
        if cleared {
            self.meta_file = None;
        } else if let Some(file) = picked {
            self.load_pdf_into_active_tab(file);
        }

        if self.meta_file.is_some() {
            ui.add_space(10.0);
            ui_card(ui, dark, |ui| {
                egui::Grid::new("meta_grid")
                    .num_columns(2)
                    .spacing([15.0, 10.0])
                    .show(ui, |ui| {
                        ui.label("Titel:");
                        ui.add(egui::TextEdit::singleline(&mut self.metadata.title).desired_width(440.0));
                        ui.end_row();

                        ui.label("Autor:");
                        ui.add(egui::TextEdit::singleline(&mut self.metadata.author).desired_width(440.0));
                        ui.end_row();

                        ui.label("Thema / Betreff:");
                        ui.add(egui::TextEdit::singleline(&mut self.metadata.subject).desired_width(440.0));
                        ui.end_row();

                        ui.label("Stichwörter:");
                        ui.add(egui::TextEdit::singleline(&mut self.metadata.keywords).desired_width(440.0));
                        ui.end_row();

                        ui.label("Ersteller (Creator):");
                        ui.add(egui::TextEdit::singleline(&mut self.metadata.creator).desired_width(440.0));
                        ui.end_row();

                        ui.label("Producer:");
                        ui.add(egui::TextEdit::singleline(&mut self.metadata.producer).desired_width(440.0));
                        ui.end_row();
                    });

                ui.add_space(8.0);
                if ui.small_button("🧹 Alle Felder leeren").clicked() {
                    self.metadata = PdfMetadata::default();
                }
            });

            ui.add_space(12.0);
            if primary_button(ui, dark, true, "💾 Metadaten speichern...") {
                let input = self.meta_file.as_ref().unwrap().clone();
                if let Some(dest) = rfd::FileDialog::new()
                    .add_filter("PDF", &["pdf"])
                    .set_file_name("metadaten.pdf")
                    .save_file()
                {
                    match write_metadata(&input, &self.metadata, &dest) {
                        Ok(()) => self.set_saved_status(
                            format!("Metadaten erfolgreich aktualisiert: {}", dest.display()),
                            dest,
                        ),
                        Err(e) => self.set_status(e, true),
                    }
                }
            }
        }
    }

    fn render_text_info_tab(&mut self, ui: &mut egui::Ui) {
        let dark = self.dark_mode;
        let accent = accent_color(dark);

        let (picked, cleared) = render_file_dropzone(
            ui,
            dark,
            &self.info_file,
            self.info_details.as_ref().map(|d| d.page_count),
            "📂 PDF analysieren...",
        );
        if cleared {
            self.info_file = None;
            self.info_details = None;
            self.extracted_text.clear();
        } else if let Some(file) = picked {
            self.load_pdf_into_active_tab(file);
        }

        if let Some(details) = self.info_details.clone() {
            ui.add_space(10.0);
            ui_card(ui, dark, |ui| {
                ui.label(RichText::new("📄 Dokument-Diagnose").strong().size(15.0));
                ui.add_space(4.0);
                egui::Grid::new("info_diag_grid")
                    .num_columns(4)
                    .spacing([24.0, 8.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Seitenanzahl:").weak());
                        ui.label(RichText::new(format!("{}", details.page_count)).strong());
                        ui.label(RichText::new("Seitenformat:").weak());
                        ui.label(RichText::new(&details.page_format_label).strong());
                        ui.end_row();

                        ui.label(RichText::new("Dateigröße:").weak());
                        ui.label(format!(
                            "{:.2} KB ({:.2} MB)",
                            details.file_size_bytes as f64 / 1024.0,
                            details.file_size_bytes as f64 / 1_048_576.0
                        ));
                        ui.label(RichText::new("PDF-Version:").weak());
                        ui.label(RichText::new(&details.version).strong());
                        ui.end_row();

                        ui.label(RichText::new("PDF-Objekte:").weak());
                        ui.label(format!("{}", details.object_count));
                        ui.label(RichText::new("Bilder / Schriften:").weak());
                        ui.label(format!(
                            "{} Bilder / {} Fonts",
                            details.image_count, details.font_count
                        ));
                        ui.end_row();

                        ui.label(RichText::new("Verschlüsselt:").weak());
                        ui.label(if details.is_encrypted { "🔒 Ja" } else { "🔓 Nein" });
                        ui.label(RichText::new("Formulare / Lesezeichen:").weak());
                        ui.label(format!(
                            "{} / {}",
                            if details.has_forms { "Ja" } else { "Nein" },
                            if details.has_outlines { "Ja" } else { "Nein" }
                        ));
                        ui.end_row();
                    });
            });

            ui.add_space(10.0);
            let mut extract_clicked = false;
            ui_card(ui, dark, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Seiten für Text-Extraktion ('alle' oder z.B. '1-3'):");
                    ui.text_edit_singleline(&mut self.text_page_range);
                    let btn = egui::Button::new(
                        RichText::new("🔍 Volltext auslesen")
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(accent);
                    if ui.add(btn).clicked() {
                        extract_clicked = true;
                    }
                });
            });

            if extract_clicked {
                let input = self.info_file.as_ref().unwrap().clone();
                match parse_page_ranges(&self.text_page_range, details.page_count as u32) {
                    Ok(pages) => match extract_text(&input, &pages) {
                        Ok(txt) => {
                            self.extracted_text = txt;
                            self.set_status("Text erfolgreich extrahiert", false);
                        }
                        Err(e) => self.set_status(e, true),
                    },
                    Err(e) => self.set_status(e, true),
                }
            }

            if !self.extracted_text.is_empty() {
                let word_count = self.extracted_text.split_whitespace().count();
                let char_count = self.extracted_text.chars().count();
                let mut copied = false;
                let mut saved_txt = None;

                ui.add_space(10.0);
                ui_card(ui, dark, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("📊 {word_count} Wörter • {char_count} Zeichen"))
                                .strong(),
                        );
                        if ui.small_button("📋 Kopieren").clicked() {
                            ui.ctx().copy_text(self.extracted_text.clone());
                            copied = true;
                        }
                        if ui.small_button("💾 Als .txt speichern").clicked() {
                            if let Some(dest) = rfd::FileDialog::new()
                                .add_filter("Textdatei", &["txt"])
                                .set_file_name("extrahiert.txt")
                                .save_file()
                            {
                                if std::fs::write(&dest, &self.extracted_text).is_ok() {
                                    saved_txt = Some(dest);
                                }
                            }
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.text_search_query)
                                    .hint_text("🔎 Im Text filtern..."),
                            );
                        });
                    });

                    let display_text = if self.text_search_query.trim().is_empty() {
                        self.extracted_text.clone()
                    } else {
                        let q = self.text_search_query.to_lowercase();
                        self.extracted_text
                            .lines()
                            .filter(|line| line.to_lowercase().contains(&q))
                            .collect::<Vec<_>>()
                            .join("\n")
                    };

                    ui.add_space(4.0);
                    ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                        let mut read_only_view = display_text;
                        ui.add(
                            egui::TextEdit::multiline(&mut read_only_view)
                                .desired_width(f32::INFINITY)
                                .font(egui::TextStyle::Monospace),
                        );
                    });
                });

                if copied {
                    self.set_status("Text in die Zwischenablage kopiert", false);
                }
                if let Some(dest) = saved_txt {
                    self.set_saved_status(format!("Text gespeichert: {}", dest.display()), dest);
                }
            }
        }
    }
}

fn format_pages_compact(pages: &[u32]) -> String {
    if pages.is_empty() {
        return String::new();
    }
    let mut parts = Vec::new();
    let mut i = 0;
    while i < pages.len() {
        let start = pages[i];
        let mut end = start;
        while i + 1 < pages.len() && pages[i + 1] == end + 1 {
            end = pages[i + 1];
            i += 1;
        }
        if end > start {
            parts.push(format!("{start}-{end}"));
        } else {
            parts.push(format!("{start}"));
        }
        i += 1;
    }
    parts.join(", ")
}

fn reveal_in_explorer(path: &Path) {
    if path.is_dir() {
        let _ = Command::new("explorer").arg(path).spawn();
    } else {
        let _ = Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn();
    }
}

fn open_default_app(path: &Path) {
    let _ = Command::new("cmd")
        .args(["/C", "start", "", &path.to_string_lossy()])
        .spawn();
}

impl eframe::App for PdfToolkitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dropped_paths: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|df| df.path.clone())
                .collect()
        });

        for path in dropped_paths {
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            if ext == "pdf" {
                self.load_pdf_into_active_tab(path);
            } else if matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "bmp") {
                self.current_tab = Tab::ImagesToPdf;
                self.img_files.push(path);
            }
        }

        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(36.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    if let Some((msg, is_error)) = &self.status_message {
                        let (icon, color) = if *is_error {
                            ("⚠️", Color32::from_rgb(239, 68, 68))
                        } else {
                            ("✅", Color32::from_rgb(34, 197, 94))
                        };
                        ui.label(RichText::new(format!("{icon} {msg}")).color(color).strong());
                    } else {
                        ui.label(
                            RichText::new("🟢 Bereit • PDF-Dateien oder Bilder einfach per Drag & Drop in das Fenster ziehen.")
                                .weak(),
                        );
                    }

                    if let Some(saved) = self.last_saved_path.clone() {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("📂 Im Explorer zeigen").clicked() {
                                reveal_in_explorer(&saved);
                            }
                            if saved.is_file() && ui.small_button("🚀 Datei öffnen").clicked() {
                                open_default_app(&saved);
                            }
                            if saved.extension().and_then(|e| e.to_str()) == Some("pdf")
                                && ui
                                    .small_button("➡️ Als Eingabe weiterverwenden")
                                    .on_hover_text("Lädt die gespeicherte PDF direkt für den nächsten Bearbeitungsschritt")
                                    .clicked()
                            {
                                self.load_pdf_into_active_tab(saved);
                            }
                        });
                    }
                });
            });

        egui::SidePanel::left("sidebar")
            .resizable(false)
            .default_width(250.0)
            .show(ctx, |ui| {
                self.render_sidebar(ctx, ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                self.render_workspace_header(ui);
                match self.current_tab {
                    Tab::Merge => self.render_merge_tab(ui),
                    Tab::Split => self.render_split_tab(ui),
                    Tab::Organize => self.render_organize_tab(ui),
                    Tab::Rotate => self.render_rotate_tab(ui),
                    Tab::Delete => self.render_delete_tab(ui),
                    Tab::Watermark => self.render_watermark_tab(ui),
                    Tab::PageNumbers => self.render_page_numbers_tab(ui),
                    Tab::CropScale => self.render_crop_scale_tab(ui),
                    Tab::Compress => self.render_compress_tab(ui),
                    Tab::ImagesToPdf => self.render_images_to_pdf_tab(ui),
                    Tab::ExtractImages => self.render_extract_images_tab(ui),
                    Tab::Security => self.render_security_tab(ui),
                    Tab::Sanitize => self.render_sanitize_tab(ui),
                    Tab::Metadata => self.render_metadata_tab(ui),
                    Tab::TextInfo => self.render_text_info_tab(ui),
                }
                ui.add_space(16.0);
            });
        });

        let is_hovering_files = ctx.input(|i| !i.raw.hovered_files.is_empty());
        if is_hovering_files {
            let screen_rect = ctx.screen_rect();
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("drag_drop_overlay"),
            ));
            painter.rect_filled(
                screen_rect,
                CornerRadius::ZERO,
                Color32::from_rgba_unmultiplied(15, 18, 28, 210),
            );
            let inner = screen_rect.shrink(28.0);
            painter.rect_stroke(
                inner,
                CornerRadius::same(16),
                Stroke::new(3.0_f32, accent_color(self.dark_mode)),
                StrokeKind::Inside,
            );
            let (_, title, _, _) = self.current_tab.info();
            painter.text(
                inner.center(),
                Align2::CENTER_CENTER,
                format!("📥 Dateien hier loslassen\n→ Direkt in '{}' laden", title),
                FontId::proportional(24.0),
                Color32::WHITE,
            );
        }
    }
}
