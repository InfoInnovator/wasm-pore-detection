use std::{path::PathBuf, sync::mpsc};

use egui::{Pos2, TextureHandle};
use egui_extras::install_image_loaders;
use egui_plot::PlotPoint;
use image::DynamicImage;
use imageproc::drawing::Canvas;

use super::images::Images;

#[derive(Default)]
pub struct PoreDetectionApp {
    pub threshold: i16,
    pub minimal_pore_size_low: f32,
    pub minimal_pore_size_high: f32,
    pub image_to_display: Option<TextureHandle>,
    pub region_selector: (Option<Pos2>, Option<Pos2>),
    pub region: (Option<Pos2>, Option<Pos2>),
    pub receiver: Option<mpsc::Receiver<(Vec<PlotPoint>, f64)>>,
    pub image_paths: Option<Vec<PathBuf>>,
    pub images: Images,
    pub export_window_open: bool,
}

impl PoreDetectionApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_image_loaders(&cc.egui_ctx);
        Self {
            minimal_pore_size_low: 0.0,
            minimal_pore_size_high: 1000.0,
            ..Default::default()
        }
    }

    fn load_texture(&mut self, ctx: &egui::Context, image: &DynamicImage) -> &TextureHandle {
        let rgba_image = image.to_rgba8();
        let size = [image.width() as _, image.height() as _];
        let pixels: &[u8] = rgba_image.as_raw();

        log::info!("loaded image data");

        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels);
        log::info!("created color image");
        let handle: &TextureHandle = self.image_to_display.insert({
            ctx.load_texture(
                "dynamic_image",
                color_image,
                egui::TextureOptions::default(),
            )
        });
        log::info!("fully loaded texture");
        handle
    }

    pub fn reload_image(&mut self, ctx: &egui::Context, selected_image: Option<usize>) {
        log::info!("reloading image");

        let mut selected = 0;
        if let Some(selected_image) = selected_image {
            self.images.selected = Some(selected_image);
            selected = selected_image;
        } else {
            self.images.selected = Some(0);
        }

        let image = image::open(self.images.images[selected].path.clone().unwrap())
            .expect("Could not load image from path");
        log::info!(
            "loaded image: {}",
            self.images.images[selected].path.clone().unwrap().display()
        );

        self.load_texture(ctx, &image);
        // self.image_to_display = Some(handle);

        log::info!("loaded texture");

        let (tx, rx) = mpsc::channel();
        self.receiver = Some(rx);

        log::info!("analyzing image");
        self.images.images[selected].analyze_image(
            tx,
            self.threshold,
            self.minimal_pore_size_low,
            self.minimal_pore_size_high,
        );
        log::info!("analyzed image");
    }

    pub fn receive_image_data(&mut self, ctx: &egui::Context) {
        // [TODO] move this to a another thread bc painting blocks the main thread for a short time
        // receive from channel
        if let Some(rec) = &self.receiver {
            if let Ok((black_pixels, density)) = rec.try_recv() {
                let selected_img = self.images.selected.unwrap_or(0);

                self.images.images[selected_img].black_pixels = Some(black_pixels.clone());
                self.images.images[selected_img].density = Some(density);

                // draw a green pixel for each black pixel that is part of a group with a size greater than the users minimal pore size
                if let Some(path) = &self.images.images[selected_img].path {
                    log::info!("Drawing green pixels on image: {:?}", path);

                    let image = image::open(path).unwrap();
                    let mut image = image.to_rgba8();
                    let green_pixel = image::Rgba([0, 255, 13, 204]);

                    if let (Some(region_start), Some(region_end)) = (
                        self.images.images[selected_img].region_start,
                        self.images.images[selected_img].region_end,
                    ) {
                        for pixel in black_pixels {
                            let y_start = image.height() - region_start.y as u32;
                            let y_end = image.height() - region_end.y as u32;

                            if pixel.x >= region_start.x
                                && pixel.x <= region_end.x
                                && pixel.y >= y_start.into()
                                && pixel.y <= y_end.into()
                            {
                                image.draw_pixel(pixel.x as u32, pixel.y as u32, green_pixel);
                            }
                        }
                    } else {
                        for pixel in black_pixels.clone() {
                            image.draw_pixel(pixel.x as u32, pixel.y as u32, green_pixel);
                        }
                    }

                    self.image_to_display = Some(
                        self.load_texture(ctx, &DynamicImage::ImageRgba8(image))
                            .clone(),
                    );
                }
            }
        }
    }
}
