use crate::{model::detection_app::load_texture_into_ctx, PoreDetectionApp};

#[derive(Default)]
pub struct DebugInfo {
    pub grayscale_handle: Option<egui::TextureHandle>,
    pub grayscale_thresh_handle: Option<egui::TextureHandle>,
}

pub fn display_debug_window(ctx: &egui::Context, app: &mut PoreDetectionApp) {
    egui::Window::new("Debug")
        .open(&mut app.debug_window_open)
        .show(ctx, |ui| {
            ui.heading("Debug");

            if ui.button("Show grayscale").clicked() {
                let selected_img = app.images.selected.unwrap_or(0);
                let image = app.images.images[selected_img].image.clone().unwrap();
                let grayscale = image.grayscale().to_luma8();
                let grayscale_dynamic = image::DynamicImage::ImageLuma8(grayscale);

                app.debug_info.grayscale_handle =
                    Some(load_texture_into_ctx(ctx, &grayscale_dynamic));
            }

            if ui.button("Show thresholded").clicked() {
                let selected_img = app.images.selected.unwrap_or(0);
                let image = app.images.images[selected_img].image.clone().unwrap();
                let grayscale = image.grayscale().to_luma8();
                let grayscale_thresh = imageproc::contrast::threshold(
                    &grayscale,
                    app.images.images[selected_img]
                        .threshold
                        .try_into()
                        .unwrap(),
                    imageproc::contrast::ThresholdType::Binary,
                );
                let grayscale_thresh_dynamic = image::DynamicImage::ImageLuma8(grayscale_thresh);

                app.debug_info.grayscale_thresh_handle =
                    Some(load_texture_into_ctx(ctx, &grayscale_thresh_dynamic));
            }
        });
}
