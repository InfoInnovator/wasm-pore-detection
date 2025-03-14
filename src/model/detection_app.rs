use egui::{Pos2, TextureHandle};
use egui_extras::install_image_loaders;
use image::DynamicImage;

use crate::view::{debug_window::DebugInfo, export_window::ExportDecimalFormat};

use super::images::Images;

#[derive(Default)]
pub struct PoreDetectionApp {
    pub image_to_display: Option<TextureHandle>,
    pub region_selector: (Option<Pos2>, Option<Pos2>),
    pub region: (Option<Pos2>, Option<Pos2>),
    pub images: Images,
    pub join_handle: Option<
        std::thread::JoinHandle<(
            std::vec::Vec<egui_plot::PlotPoint>,
            std::vec::Vec<egui_plot::PlotPoint>,
            f64,
        )>,
    >,
    pub export_window_open: bool,
    pub debug_window_open: bool,
    pub debug_info: DebugInfo,
    pub shortcut_window_open: bool,
    pub export_decimal_format: ExportDecimalFormat,
    pub new_project_model_open: bool,
    pub load_project_model_open: bool,
}

impl PoreDetectionApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_image_loaders(&cc.egui_ctx);
        Default::default()
    }

    pub fn reload_image(&mut self, selected_image: Option<usize>) {
        self.images.prev_selected = self.images.selected;

        let mut selected = 0;
        if let Some(selected_image) = selected_image {
            self.images.selected = Some(selected_image);
            selected = selected_image;
        } else {
            self.images.selected = Some(0);
        }

        self.join_handle = Some(self.images.images[selected].analyze_image());
    }

    pub fn receive_image_data(&mut self, ctx: &egui::Context) {
        // [TODO] move this to another thread bc painting blocks the main thread for a short time
        // receive from channel
        if let Some(handle) = &self.join_handle {
            if handle.is_finished() {
                let handle = self.join_handle.take().unwrap();
                let (green_pixels, white_pixels, density) = handle.join().unwrap();

                let selected_img = self.images.selected.unwrap_or(0);

                self.images.images[selected_img].green_pixels = Some(green_pixels.clone());
                self.images.images[selected_img].density = Some(density);

                // draw a green pixel for each black pixel that is part of a group with a size greater than the users minimal pore size
                if let Some(path) = &self.images.images[selected_img].path {
                    log::info!("Drawing green pixels on image: {:?}", path);

                    let image = self.images.images[selected_img].image.clone().unwrap();
                    let mut image = image.to_rgba8();
                    let green_pixel = image::Rgba([0, 255, 13, 204]);
                    let white_pixel = image::Rgba([255, 255, 255, 127]);

                    green_pixels.iter().for_each(|pixel| {
                        image.put_pixel(pixel.x as u32, pixel.y as u32, green_pixel);
                    });

                    white_pixels.iter().for_each(|pixel| {
                        image.put_pixel(pixel.x as u32, pixel.y as u32, white_pixel);
                    });

                    self.image_to_display =
                        Some(load_texture_into_ctx(ctx, &DynamicImage::ImageRgba8(image)));
                }
            }
        }
    }
}

pub fn load_texture_into_ctx(ctx: &egui::Context, image: &DynamicImage) -> TextureHandle {
    let rgba_image = image.to_rgba8();
    let size = [image.width() as _, image.height() as _];
    let pixels: &[u8] = rgba_image.as_raw();

    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels);
    ctx.load_texture(
        "dynamic_image",
        color_image,
        egui::TextureOptions::default(),
    )
}
