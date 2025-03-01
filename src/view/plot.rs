use std::sync::mpsc;

use egui::{
    epaint::{self},
    Color32, Stroke, Ui, Vec2,
};
use egui_plot::{Line, PlotImage, PlotPoint, PlotPoints, PlotResponse};
use image::GenericImageView;

use crate::PoreDetectionApp;

pub fn display_plot(ctx: &egui::Context, app: &mut PoreDetectionApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);

        let plot_response = egui_plot::Plot::new("plot")
            .data_aspect(1.0)
            .show_axes(false)
            .allow_zoom(false)
            .allow_boxed_zoom(false)
            .allow_scroll(false)
            .show_grid(false)
            .show(ui, |plot_ui| {
                if scroll != 0.0 {
                    plot_ui.zoom_bounds_around_hovered(Vec2::new(
                        (scroll / 100.0).exp(),
                        (scroll / 100.0).exp(),
                    ));
                }

                if let Some(handle) = &app.image_to_display {
                    plot_ui.add(PlotImage::new(
                        handle.id(),
                        PlotPoint::new(handle.size_vec2().x / 2.0, handle.size_vec2().y / 2.0),
                        Vec2::new(handle.size_vec2().x, handle.size_vec2().y),
                    ));
                }

                if let Some(selected_img) = app.images.selected {
                    if let (Some(rect_start), Some(rect_end)) = (
                        app.images.images[selected_img].region_start,
                        app.images.images[selected_img].region_end,
                    ) {
                        let rect_plot_points: PlotPoints<'_> =
                            egui_plot::PlotPoints::Owned(Vec::from([
                                rect_start,
                                PlotPoint::new(rect_end.x, rect_start.y),
                                rect_end,
                                PlotPoint::new(rect_start.x, rect_end.y),
                                rect_start,
                            ]));

                        plot_ui.line(Line::new(rect_plot_points));
                    }
                }
            });

        region_selection(app, ui, &plot_response);
    });
}

pub fn region_selection(app: &mut PoreDetectionApp, ui: &mut Ui, plot_response: &PlotResponse<()>) {
    if app.region_selector.0.is_none()
        && plot_response.response.drag_started()
        && plot_response
            .response
            .dragged_by(egui::PointerButton::Secondary)
    {
        app.region_selector.0 = plot_response.response.hover_pos();
    }

    if let Some(hover_pos) = plot_response.response.hover_pos() {
        app.region_selector.1 = Some(hover_pos);
    }

    if let (Some(start), Some(end)) = (app.region_selector.0, app.region_selector.1) {
        let rect = epaint::Rect::from_two_pos(start, end);
        let selected_region = epaint::RectShape::stroke(
            rect,
            0.0,
            Stroke::new(2.5, Color32::GREEN),
            egui::StrokeKind::Middle,
        );
        ui.painter().rect_stroke(
            selected_region.rect,
            selected_region.corner_radius,
            selected_region.stroke,
            selected_region.stroke_kind,
        );

        if plot_response.response.drag_stopped() {
            let start = plot_response.transform.value_from_position(start);
            let end = plot_response.transform.value_from_position(end);

            let selected_img = app.images.selected.unwrap_or(0);
            if let Some(img) = &app.images.images[selected_img].image {
                let size = img.dimensions();

                let start = PlotPoint::new(
                    start.x.clamp(0.0, size.0 as f64),
                    start.y.clamp(0.0, size.1 as f64),
                );
                let end = PlotPoint::new(
                    end.x.clamp(0.0, size.0 as f64),
                    end.y.clamp(0.0, size.1 as f64),
                );

                app.images.images[selected_img].region_start = Some(start);
                app.images.images[selected_img].region_end = Some(end);

                // trigger the image analysis
                let (tx, rx) = mpsc::channel();
                app.receiver = Some(rx);
                let selected_img = app.images.selected.unwrap_or(0);
                app.images.images[selected_img].analyze_image(
                    tx,
                    app.threshold,
                    app.minimal_pore_size_low,
                    app.minimal_pore_size_high,
                );
            }

            app.region_selector.0 = None;
            app.region_selector.1 = None;
        }
    }
}
