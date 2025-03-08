use egui::{DragValue, Slider};
use egui_double_slider::DoubleSlider;
use egui_extras::{Column, TableBuilder};
use rfd::FileDialog;

use crate::{
    model::{detection_app::load_texture_into_ctx, image_data::ImageData},
    PoreDetectionApp,
};

pub fn display_sidepanel(ctx: &egui::Context, app: &mut PoreDetectionApp) {
    egui::SidePanel::new(egui::panel::Side::Left, "sidebar")
        .resizable(false)
        .max_width(525.0)
        .show(ctx, |ui| {
            ui.heading("Options");

            TableBuilder::new(ui)
                .id_salt("options_table")
                .column(Column::auto())
                .column(Column::auto())
                .striped(true)
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("Name");
                    });
                    header.col(|ui| {
                        ui.heading("Value");
                    });
                })
                .body(|mut body| {
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Threshold");
                        });
                        row.col(|ui| {
                            ui.style_mut().spacing.slider_width = 300.0;

                            if app.images.selected.is_none() {
                                return;
                            }

                            let current_image =
                                &mut app.images.images[app.images.selected.unwrap_or(0)];
                            let mut threshold = current_image.threshold;

                            ui.horizontal(|ui| {
                                let mut response =
                                    ui.add(egui::Slider::new(&mut threshold, 0..=255).step_by(1.0));
                                if ui.button("-").clicked() {
                                    threshold -= 1;
                                    response.mark_changed();
                                } else if ui.button("+").clicked() {
                                    threshold += 1;
                                    response.mark_changed();
                                }

                                // use mouse wheel to change slider
                                if response.hovered() {
                                    let scroll = ui.input(|i| i.smooth_scroll_delta);
                                    if scroll.y > 10.0 || scroll.y < -10.0 {
                                        threshold = (threshold as f32 + scroll.y.signum())
                                            .clamp(0.0, 255.0)
                                            as i16;
                                        response.mark_changed();
                                    }
                                }

                                if response.changed() {
                                    // [TODO] use channels differently bc changing the value will create a new channel
                                    //        and the old receiver will be dropped, so the thread is sending on a closed
                                    //        channel
                                    app.images.images[app.images.selected.unwrap_or(0)].threshold =
                                        threshold;
                                    app.reload_image(app.images.selected);
                                }
                            });
                        });
                    });
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Including minimal feature size");
                        });
                        row.col(|ui| {
                            ui.style_mut().spacing.slider_width = 300.0;

                            if app.images.selected.is_none() {
                                return;
                            }

                            let selected_i = app.images.selected.unwrap_or(0);
                            let current_image = &mut app.images.images[selected_i].clone();

                            ui.horizontal(|ui| {
                                let mut response = ui.add(
                                    Slider::new(
                                        &mut current_image.included_min_feature_size,
                                        0.0..=100.0,
                                    )
                                    .fixed_decimals(0),
                                );
                                if ui.button("-").clicked() {
                                    current_image.included_min_feature_size -= 1.0;
                                    response.mark_changed();
                                } else if ui.button("+").clicked() {
                                    current_image.included_min_feature_size += 1.0;
                                    response.mark_changed();
                                }

                                if response.changed() {
                                    app.images.images[selected_i] = current_image.clone();
                                    app.reload_image(app.images.selected);

                                    log::info!(
                                        "included min feature size: {}",
                                        current_image.included_min_feature_size
                                    );
                                }
                            });
                        });
                    });

                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Minimal Pore Size");
                        });
                        row.col(|ui| {
                            // ui.style_mut().spacing.slider_width = 300.0;

                            if app.images.selected.is_none() {
                                return;
                            }

                            let selected_i = app.images.selected.unwrap_or(0);
                            let current_image = &mut app.images.images[selected_i].clone();

                            let response = ui.horizontal(|ui| {
                                let mut low_drag_response = ui.add(
                                    DragValue::new(&mut current_image.minimal_pore_size_low)
                                        .speed(0.1)
                                        .fixed_decimals(0),
                                );
                                if ui.button("-").clicked() {
                                    current_image.minimal_pore_size_low -= 1.0;
                                    low_drag_response.mark_changed();
                                } else if ui.button("+").clicked() {
                                    current_image.minimal_pore_size_low += 1.0;
                                    low_drag_response.mark_changed();
                                }

                                let slider_response = ui.add(
                                    DoubleSlider::new(
                                        &mut current_image.minimal_pore_size_low,
                                        &mut current_image.minimal_pore_size_high,
                                        0.0..=i32::MAX as f32,
                                    )
                                    .width(170.0)
                                    .separation_distance(1.0),
                                );

                                let mut high_drag_response = ui.add(
                                    DragValue::new(&mut current_image.minimal_pore_size_high)
                                        .speed(0.1)
                                        .fixed_decimals(0),
                                );
                                if ui.button("-").clicked() {
                                    current_image.minimal_pore_size_high -= 1.0;
                                    high_drag_response.mark_changed();
                                } else if ui.button("+").clicked() {
                                    current_image.minimal_pore_size_high += 1.0;
                                    high_drag_response.mark_changed();
                                }

                                (low_drag_response, slider_response, high_drag_response)
                            });

                            if response.inner.0.changed()
                                || response.inner.1.changed()
                                || response.inner.2.changed()
                            {
                                app.images.images[selected_i] = current_image.clone();
                                app.reload_image(app.images.selected);

                                log::info!(
                                    "min pore size bounds: {} - {}",
                                    current_image.minimal_pore_size_low,
                                    current_image.minimal_pore_size_high
                                );
                            }
                        });
                    });
                });

            ui.with_layout(
                egui::Layout::with_cross_justify(
                    egui::Layout::left_to_right(egui::Align::Min),
                    false,
                ),
                |ui| {
                    let button_width = (ui.available_width() / 2.0) - 10.0;

                    if ui
                        .add_sized([button_width, 0.0], egui::Button::new("Reset Region"))
                        .clicked()
                    {
                        log::info!("Reset Region");
                        let selected_img = app.images.selected.unwrap_or(0);
                        app.images.images[selected_img].region_start = None;
                        app.images.images[selected_img].region_end = None;

                        app.reload_image(app.images.selected);
                    }

                    if ui
                        .add_sized([button_width, 0.0], egui::Button::new("Export Results"))
                        .clicked()
                    {
                        log::info!("Export Results");
                        app.export_window_open = true;
                    }
                },
            );

            if let Some(selected_img) = app.images.selected {
                if let Some(density) = app.images.images[selected_img].density {
                    ui.heading(format!("Density: {:.5}%", density));
                } else {
                    ui.heading("Density: -".to_string());
                }
            }

            ui.separator();

            ui.heading("Image List");

            if let Some(folder_path) = &app.image_paths.clone() {
                // create table with image names
                TableBuilder::new(ui)
                    .id_salt("image_list")
                    .column(Column::auto().at_least(200.0))
                    .column(Column::remainder())
                    .striped(true)
                    .sense(egui::Sense::click())
                    .body(|mut body| {
                        for (i, path) in folder_path.iter().enumerate() {
                            let path_str = path.file_name().unwrap().to_str().unwrap().to_string();

                            body.row(150.0, |mut row| {
                                row.set_selected(Some(i) == app.images.selected);

                                row.col(|ui| {
                                    ui.image(format!("file://{}", path.to_str().unwrap()));
                                });
                                row.col(|ui| {
                                    ui.heading(path_str);

                                    ui.label(format!(
                                        "Density: {:.5}%",
                                        app.images.images[i].density.unwrap_or(0.0),
                                    ));
                                });

                                if row.response().clicked() {
                                    app.reload_image(Some(i));
                                    log::info!("Selected Image: {:?}", app.images.selected);
                                }
                            });
                        }
                    });
            } else {
                // [TODO] make async so the ui is not blocked
                ui.vertical_centered(|ui| {
                    if ui.button("Open Files").clicked() {
                        let path = FileDialog::new().pick_files();
                        if let Some(paths) = path {
                            for path in &paths {
                                app.images.images.push(ImageData {
                                    path: Some(path.to_path_buf()),
                                    image: Some(image::open(path).unwrap()),
                                    ..Default::default()
                                });
                            }

                            // load texture handles for every image
                            for image in &mut app.images.images {
                                image.image_handle =
                                    Some(load_texture_into_ctx(ctx, &image.image.clone().unwrap()));
                            }

                            app.images.selected = Some(0);
                            app.image_to_display = app.images.images[0].image_handle.clone();
                            app.image_paths = Some(paths.clone());
                        }
                    }
                });
            }
        });
}
