use egui::{epaint, Color32, Pos2, TextureHandle, Vec2, Vec2b};
use egui_extras::{Column, TableBuilder};
use egui_plot::{Line, PlotBounds, PlotImage, PlotPoint, PlotPoints};
use image::DynamicImage;
use imageproc::contrast::threshold;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,

    #[serde(skip)]
    threshold: i16,

    #[serde(skip)]
    minimal_pore_size: i16,

    #[serde(skip)]
    selected_area: Option<PlotBounds>,

    #[serde(skip)]
    selected_texture_handle: Option<TextureHandle>,

    #[serde(skip)]
    region_selector_start: Option<Pos2>,

    #[serde(skip)]
    region_selector_end: Option<Pos2>,

    #[serde(skip)]
    region_rect_start: Option<PlotPoint>,

    #[serde(skip)]
    region_rect_end: Option<PlotPoint>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            threshold: 0,
            minimal_pore_size: 0,
            selected_area: None,
            selected_texture_handle: None,
            region_selector_start: None,
            region_selector_end: None,
            region_rect_start: None,
            region_rect_end: None,
        }
    }
}

fn load_texture(ctx: &egui::Context, image: &DynamicImage) -> TextureHandle {
    let rgba_image = image.to_rgba8();
    let size = [image.width() as _, image.height() as _];
    let pixels: &[u8] = &rgba_image.into_raw();

    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels);
    ctx.load_texture(
        "dynamic_image",
        color_image,
        egui::TextureOptions::default(),
    )
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        let image = image::open("assets/example_image.png").unwrap();
        let texture_handle = Some(load_texture(&cc.egui_ctx, &image));

        Self {
            selected_texture_handle: texture_handle,
            ..Default::default()
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        // create floating window
        egui::SidePanel::new(egui::panel::Side::Left, "sidebar")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Options");

                TableBuilder::new(ui)
                    .column(Column::auto())
                    .column(Column::auto())
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("Name");
                        });
                        header.col(|ui| {
                            ui.heading("Value");
                        });
                    })
                    .body(|mut body| {
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Threshold");
                            });
                            row.col(|ui| {
                                ui.style_mut().spacing.slider_width = 250.0;

                                let response = ui.add(
                                    egui::Slider::new(&mut self.threshold, 0..=255).step_by(1.0),
                                );

                                if response.changed() {
                                    let image = image::open("assets/example_image.png").unwrap();

                                    let grayscale = image.grayscale().to_luma8();
                                    let grayscale_thresh = threshold(
                                        &grayscale,
                                        self.threshold.try_into().unwrap(),
                                        imageproc::contrast::ThresholdType::Binary,
                                    );

                                    let texture_handle =
                                        Some(load_texture(ctx, &grayscale_thresh.into()));
                                    self.selected_texture_handle = texture_handle;

                                    log::info!("Threshold changed to {}", self.threshold);
                                }
                            });
                        });
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Minimal Pore Size");
                            });
                            row.col(|ui| {
                                ui.style_mut().spacing.slider_width = 250.0;
                                ui.add(egui::Slider::new(&mut self.minimal_pore_size, 0..=250));
                            });
                        });
                    });

                if ui.button("Select Region").clicked() {
                    log::info!("Select Region");
                }

                if ui.button("Apply to Batch").clicked() {
                    log::info!("Apply to Batch");
                }

                if ui.button("Download Results").clicked() {
                    log::info!("Download Results");
                }

                ui.separator();

                ui.heading("Image List");
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let plot_response = egui_plot::Plot::new("plot")
                .allow_zoom(true)
                .data_aspect(1.0)
                .show_axes(false)
                .allow_boxed_zoom(false)
                .show_grid(false)
                .show(ui, |plot_ui| {
                    plot_ui.set_auto_bounds(Vec2b::new(false, false));

                    if let Some(handle) = &self.selected_texture_handle {
                        plot_ui.add(PlotImage::new(
                            handle.id(),
                            PlotPoint::new(0.5, 1.0 / handle.aspect_ratio() / 2.0),
                            Vec2::new(1.0, 1.0 / handle.aspect_ratio()),
                        ));
                    }

                    if let (Some(rect_start), Some(rect_end)) =
                        (self.region_rect_start, self.region_rect_end)
                    {
                        let rect_plot_points: PlotPoints =
                            egui_plot::PlotPoints::Owned(Vec::from([
                                rect_start,
                                PlotPoint::new(rect_end.x, rect_start.y),
                                rect_end,
                                PlotPoint::new(rect_start.x, rect_end.y),
                                rect_start,
                            ]));

                        plot_ui.line(Line::new(rect_plot_points));
                    }
                });

            // Region Selector
            {
                if self.region_selector_start.is_none()
                    && plot_response.response.drag_started()
                    && plot_response
                        .response
                        .dragged_by(egui::PointerButton::Secondary)
                {
                    self.region_selector_start = plot_response.response.hover_pos();
                }

                if let Some(hover_pos) = plot_response.response.hover_pos() {
                    self.region_selector_end = Some(hover_pos);
                }

                if let (Some(start), Some(end)) =
                    (self.region_selector_start, self.region_selector_end)
                {
                    let rect = epaint::Rect::from_two_pos(start, end);
                    let selected_region = epaint::RectShape::stroke(
                        rect,
                        0.0,
                        epaint::Stroke::new(1.25, Color32::GREEN),
                    );
                    ui.painter().rect_stroke(
                        selected_region.rect,
                        selected_region.rounding,
                        selected_region.stroke,
                    );

                    if plot_response.response.drag_stopped() {
                        let start = plot_response.transform.value_from_position(start);
                        let end = plot_response.transform.value_from_position(end);

                        self.region_rect_start = Some(start);
                        self.region_rect_end = Some(end);

                        self.region_selector_start = None;
                        self.region_selector_end = None;
                    }
                }
            }
        });
    }
}
