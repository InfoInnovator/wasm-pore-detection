use egui::Modal;

use crate::{model::detection_app, PoreDetectionApp};

pub fn display_new_project_modal(ctx: &egui::Context, app: &mut detection_app::PoreDetectionApp) {
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
                    *app = PoreDetectionApp::default();
                }
            });
        });
    }
}
