use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::mpsc,
};

use egui::{
    epaint::{self},
    Color32, Pos2, TextureHandle, Ui, Vec2,
};
use egui_extras::{Column, TableBuilder};
use egui_plot::{Line, PlotBounds, PlotImage, PlotPoint, PlotPoints, PlotResponse};
use image::{DynamicImage, Luma};
use imageproc::{
    definitions::{HasBlack, HasWhite},
    drawing::Canvas,
};
use rfd::FileDialog;

pub struct PoreDetectionApp {
    threshold: i16,

    minimal_pore_size: i16,

    selected_area: Option<PlotBounds>,

    selected_texture_handle: Option<TextureHandle>,

    region_selector_start: Option<Pos2>,

    region_selector_end: Option<Pos2>,

    region_rect_start: Option<PlotPoint>,

    region_rect_end: Option<PlotPoint>,

    black_pixels: Option<Vec<PlotPoint>>,

    density: Option<f64>,

    receiver: Option<mpsc::Receiver<(Vec<PlotPoint>, f64)>>,

    folder_path: Option<Vec<PathBuf>>,

    selected_image: Option<DynamicImage>,

    selected_image_path: Option<PathBuf>,
}

impl Default for PoreDetectionApp {
    fn default() -> Self {
        Self {
            threshold: 75,
            minimal_pore_size: 0,
            selected_area: None,
            selected_texture_handle: None,
            region_selector_start: None,
            region_selector_end: None,
            region_rect_start: None,
            region_rect_end: None,
            black_pixels: None,
            density: None,
            receiver: None,
            folder_path: None,
            selected_image: None,
            selected_image_path: None,
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

impl PoreDetectionApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    pub fn analyze_image(&mut self, image: DynamicImage, tx: mpsc::Sender<(Vec<PlotPoint>, f64)>) {
        let threshold = self.threshold;
        let minimal_pore_size = self.minimal_pore_size;

        std::thread::spawn(move || {
            // convert to grayscale
            let grayscale = image.grayscale().to_luma8();
            // apply threshold
            let grayscale_thresh = imageproc::contrast::threshold(
                &grayscale,
                threshold.try_into().unwrap(),
                imageproc::contrast::ThresholdType::Binary,
            );

            // find connected groups of white pixels
            let labels = imageproc::region_labelling::connected_components(
                &grayscale_thresh,
                imageproc::region_labelling::Connectivity::Eight,
                Luma::white(),
            );

            // count the pixels for each group/label
            let mut test: HashMap<u32, i32> = HashMap::new();
            labels.enumerate_pixels().for_each(|(_, _, p)| {
                let count = test.entry(p[0]).or_insert(0);
                *count += 1;
            });

            // draw a green pixel for each black pixel that is part of a group with a size greater than the users minimal pore size
            let mut black_pixels = Vec::new();
            labels.enumerate_pixels().for_each(|(x, y, p)| {
                if grayscale_thresh.get_pixel(x, y) == &Luma::black()
                    && test[&p[0]] > minimal_pore_size.into()
                {
                    black_pixels.push(PlotPoint::new(x, y));
                }
            });

            // calculate the density for the whole image
            let density = (1.0
                - (black_pixels.len() as f64 / (grayscale.width() * grayscale.height()) as f64))
                * 100.0;

            // send the black pixels and the density to the main thread
            if let Err(err) = tx.send((black_pixels.clone(), density)) {
                log::error!("Error sending data to another thread: {}", err);
            }
        });
    }
}

impl eframe::App for PoreDetectionApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // [TODO] move this to a another thread bc painting blocks the main thread for a short time
        // receive from channel
        if let Some(rec) = &self.receiver {
            if let Ok((black_pixels, density)) = rec.try_recv() {
                self.black_pixels = Some(black_pixels.clone());
                self.density = Some(density);

                // draw a green pixel for each black pixel that is part of a group with a size greater than the users minimal pore size
                if let Some(path) = self.selected_image_path.clone() {
                    log::info!("Drawing green pixels on image: {:?}", path);

                    let image = image::open(path).unwrap();
                    let mut image = image.to_rgba8();
                    let green_pixel = image::Rgba([0, 255, 13, 204]);
                    for pixel in black_pixels {
                        image.draw_pixel(pixel.x as u32, pixel.y as u32, green_pixel);
                    }

                    let texture_handle = Some(load_texture(ctx, &DynamicImage::ImageRgba8(image)));
                    self.selected_texture_handle = texture_handle;
                }
            }
        }

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
                                    // [TODO] use channels differently bc changing the value will create a new channel
                                    //        and the old receiver will be dropped, so the thread is sending on a closed
                                    //        channel
                                    let (tx, rx) = mpsc::channel();
                                    self.receiver = Some(rx);

                                    if let Some(image) = self.selected_image.clone() {
                                        self.analyze_image(image.clone(), tx);
                                    } else {
                                        log::warn!("No image selected");
                                    }
                                }
                            });
                        });
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Minimal Pore Size");
                            });
                            row.col(|ui| {
                                ui.style_mut().spacing.slider_width = 250.0;
                                let response =
                                    ui.add(egui::Slider::new(&mut self.minimal_pore_size, 0..=75));

                                if response.changed() {
                                    let (tx, rx) = mpsc::channel();
                                    self.receiver = Some(rx);

                                    // let image = image::open("assets/example_image.png").unwrap();
                                    if let Some(image) = self.selected_image.clone() {
                                        self.analyze_image(image.clone(), tx);
                                    }
                                }
                            });
                        });
                    });

                if ui.button("Reset Region").clicked() {
                    log::info!("Reset Region");
                    self.region_rect_start = None;
                    self.region_rect_end = None;
                }

                if ui.button("Apply to Batch").clicked() {
                    log::info!("Apply to Batch");
                }

                if ui.button("Download Results").clicked() {
                    log::info!("Download Results");
                }

                if let Some(density) = self.density {
                    ui.heading(format!("Density: {:.5}%", density));
                } else {
                    ui.heading("Density: -".to_string());
                }

                ui.separator();

                ui.heading("Image List");

                // [TODO] make async so the ui is not blocked
                if let Some(folder_path) = &self.folder_path {
                    for path in folder_path {
                        let path_str = path.file_name().unwrap().to_str().unwrap();

                        if ui.button(path_str).clicked() {
                            self.selected_image_path = Some(path.clone());
                            let image = image::open(path).unwrap();
                            self.selected_image = Some(image.clone());

                            let texture_handle = Some(load_texture(ctx, &image.clone()));
                            self.selected_texture_handle = texture_handle;

                            log::info!("Selected Image: {}", path_str);
                        }
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        if ui.button("Open Files").clicked() {
                            let path = FileDialog::new().pick_files();
                            if let Some(path) = path {
                                self.folder_path = Some(path);
                            }
                        }
                    });
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let plot_response = egui_plot::Plot::new("plot")
                .allow_zoom(true)
                .data_aspect(1.0)
                .show_axes(false)
                .allow_boxed_zoom(false)
                .show_grid(false)
                .show(ui, |plot_ui| {
                    if let Some(handle) = &self.selected_texture_handle {
                        plot_ui.add(PlotImage::new(
                            handle.id(),
                            PlotPoint::new(handle.size_vec2().x / 2.0, handle.size_vec2().y / 2.0),
                            Vec2::new(handle.size_vec2().x, handle.size_vec2().y),
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

            region_selection(self, ui, &plot_response);
        });
    }
}

fn region_selection(app: &mut PoreDetectionApp, ui: &mut Ui, plot_response: &PlotResponse<()>) {
    if app.region_selector_start.is_none()
        && plot_response.response.drag_started()
        && plot_response
            .response
            .dragged_by(egui::PointerButton::Secondary)
    {
        app.region_selector_start = plot_response.response.hover_pos();
    }

    if let Some(hover_pos) = plot_response.response.hover_pos() {
        app.region_selector_end = Some(hover_pos);
    }

    if let (Some(start), Some(end)) = (app.region_selector_start, app.region_selector_end) {
        let rect = epaint::Rect::from_two_pos(start, end);
        let selected_region =
            epaint::RectShape::stroke(rect, 0.0, epaint::Stroke::new(2.5, Color32::GREEN));
        ui.painter().rect_stroke(
            selected_region.rect,
            selected_region.rounding,
            selected_region.stroke,
        );

        if plot_response.response.drag_stopped() {
            let start = plot_response.transform.value_from_position(start);
            let end = plot_response.transform.value_from_position(end);

            app.region_rect_start = Some(start);
            app.region_rect_end = Some(end);

            app.region_selector_start = None;
            app.region_selector_end = None;
        }
    }
}
