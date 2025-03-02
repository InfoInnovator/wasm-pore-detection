use crate::{
    model::detection_app::PoreDetectionApp,
    view::{plot, sidepanel, top_panel},
};

impl eframe::App for PoreDetectionApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.receive_image_data(ctx);

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            log::info!("Right arrow key pressed");

            if let Some(selected) = self.images.selected {
                if selected < self.images.images.len() - 1 {
                    self.images.selected = Some(selected + 1);
                    self.reload_image(ctx, Some(selected + 1));
                } else {
                    self.images.selected = Some(0);
                    self.reload_image(ctx, None);
                }
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            log::info!("Left arrow key pressed");

            if let Some(selected) = self.images.selected {
                if selected > 0 {
                    self.images.selected = Some(selected - 1);
                    self.reload_image(ctx, Some(selected - 1));
                } else {
                    self.images.selected = Some(self.images.images.len() - 1);
                    self.reload_image(ctx, Some(self.images.images.len() - 1));
                }
            }
        }

        top_panel::display_top_panel(ctx);

        sidepanel::display_sidepanel(ctx, self);

        plot::display_plot(ctx, self);

        ctx.request_repaint();
    }
}
