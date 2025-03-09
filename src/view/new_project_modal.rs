use egui::Modal;

use crate::model::{detection_app, images::Images};

pub fn display_new_projet_modal(ctx: &egui::Context, app: &mut detection_app::PoreDetectionApp) {
    if app.new_project_model_open {
        Modal::new("new_project_modal".into()).show(ctx, |ui| {
            ui.heading("Are you sure?");

            ui.label("This will delete all your current data and start a new project.");

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    app.new_project_model_open = false;
                }

                ui.add_space(8.0);

                if ui.button("New Project").clicked() {
                    app.new_project_model_open = false;

                    // Reset the app state
                    app.image_to_display = None;
                    app.region_selector = (None, None);
                    app.region = (None, None);
                    app.receiver = None;
                    app.image_paths = None;
                    app.images = Images {
                        images: vec![],
                        selected: None,
                        prev_selected: None,
                    };
                    app.export_window_open = false;
                    app.debug_window_open = false;
                    app.debug_info = Default::default();
                    app.shortcut_window_open = false;
                    app.export_decimal_format = Default::default();
                }
            });
        });
    }
}
