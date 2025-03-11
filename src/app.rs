use crate::{
    model::detection_app::PoreDetectionApp,
    view::{
        debug_window, export_window, load_project_modal, new_project_modal, plot, shortcut_window,
        sidepanel, top_panel,
    },
};

impl eframe::App for PoreDetectionApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.receive_image_data(ctx);

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            log::info!("Right arrow key pressed");

            if let Some(selected) = self.images.selected {
                if selected < self.images.images.len() - 1 {
                    self.reload_image(Some(selected + 1));
                } else {
                    self.reload_image(None);
                }
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            log::info!("Left arrow key pressed");

            if let Some(selected) = self.images.selected {
                if selected > 0 {
                    self.reload_image(Some(selected - 1));
                } else {
                    self.reload_image(Some(self.images.images.len() - 1));
                }
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            log::info!("Enter key pressed");

            // apply options from previous selected image to current selected image
            if let Some(prev_img_i) = self.images.prev_selected {
                let prev_img = &self.images.images[prev_img_i].clone();

                if let (Some(region_start), Some(region_end)) =
                    (prev_img.region_start, prev_img.region_end)
                {
                    self.images.images[self.images.selected.unwrap()].region_start =
                        Some(region_start);
                    self.images.images[self.images.selected.unwrap()].region_end = Some(region_end);
                }

                self.images.images[self.images.selected.unwrap()].threshold = prev_img.threshold;
                self.images.images[self.images.selected.unwrap()].minimal_pore_size_low =
                    prev_img.minimal_pore_size_low;
                self.images.images[self.images.selected.unwrap()].minimal_pore_size_high =
                    prev_img.minimal_pore_size_high;
                self.images.images[self.images.selected.unwrap()].included_min_feature_size =
                    prev_img.included_min_feature_size;

                self.reload_image(self.images.selected);
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::D)) {
            self.debug_window_open = !self.debug_window_open;
        }

        top_panel::display_top_panel(ctx, self);

        shortcut_window::display_shortcut_window(ctx, self);

        new_project_modal::display_new_project_modal(ctx, self);

        load_project_modal::display_load_project_modal(ctx, self);

        sidepanel::display_sidepanel(ctx, self);

        export_window::display_export_window(ctx, self);

        debug_window::display_debug_window(ctx, self);

        plot::display_plot(ctx, self);

        ctx.request_repaint();
    }
}
