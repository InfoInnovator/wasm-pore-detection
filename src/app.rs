use crate::{
    model::detection_app::PoreDetectionApp,
    view::{plot, sidepanel, top_panel},
};

impl eframe::App for PoreDetectionApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.receive_image_data(ctx);

        top_panel::display_top_panel(ctx);

        sidepanel::display_sidepanel(ctx, self);

        plot::display_plot(ctx, self);
    }
}
