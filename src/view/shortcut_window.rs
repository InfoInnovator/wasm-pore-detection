use crate::PoreDetectionApp;

pub fn display_shortcut_window(ctx: &egui::Context, app: &mut PoreDetectionApp) {
    if app.shortcut_window_open {
        egui::Window::new("Shortcuts")
            .open(&mut app.shortcut_window_open)
            .show(ctx, |ui| {
                ui.heading("Shortcuts");
                ui.label("Left Arrow: Previous image");
                ui.label("Right Arrow: Next image");
                ui.label("Enter: Apply region from previous image to current image");
                ui.label("Drag right mouse button: Select region (needs to be from top-left to bottom-right!)");
                ui.label("Scroll wheel: Zoom in/out");
                ui.label("Double click left mouse button: Reset zoom and center image");
            });
    }
}
