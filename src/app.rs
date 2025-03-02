use crate::{
    model::detection_app::PoreDetectionApp,
    view::{plot, shortcut_window, sidepanel, top_panel},
};

impl eframe::App for PoreDetectionApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.receive_image_data(ctx);

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            log::info!("Right arrow key pressed");

            if let Some(selected) = self.images.selected {
                if selected < self.images.images.len() - 1 {
                    self.reload_image(ctx, Some(selected + 1));
                } else {
                    self.reload_image(ctx, None);
                }
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            log::info!("Left arrow key pressed");

            if let Some(selected) = self.images.selected {
                if selected > 0 {
                    self.reload_image(ctx, Some(selected - 1));
                } else {
                    self.reload_image(ctx, Some(self.images.images.len() - 1));
                }
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            log::info!("Enter key pressed");

            // apply region from previous selected image to current selected image
            if let Some(prev_img_i) = self.images.prev_selected {
                let prev_img = &self.images.images[prev_img_i];

                if let (Some(region_start), Some(region_end)) =
                    (prev_img.region_start, prev_img.region_end)
                {
                    self.images.images[self.images.selected.unwrap()].region_start =
                        Some(region_start);
                    self.images.images[self.images.selected.unwrap()].region_end = Some(region_end);

                    self.reload_image(ctx, self.images.selected);
                }
            }
        }

        top_panel::display_top_panel(ctx, self);

        shortcut_window::display_shortcut_window(ctx, self);

        sidepanel::display_sidepanel(ctx, self);

        plot::display_plot(ctx, self);

        ctx.request_repaint();
    }
}
