use egui::DragValue;
use egui_double_slider::DoubleSlider;
use egui_extras::{Column, TableBuilder};
use rfd::FileDialog;

use crate::{model::image_data::ImageData, PoreDetectionApp};

pub fn display_sidepanel(ctx: &egui::Context, app: &mut PoreDetectionApp) {
    egui::SidePanel::new(egui::panel::Side::Left, "sidebar")
        .resizable(false)
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
                            ui.style_mut().spacing.slider_width = 250.0;

                            let mut response =
                                ui.add(egui::Slider::new(&mut app.threshold, 0..=255).step_by(1.0));

                            // use mouse wheel to change slider
                            if response.hovered() {
                                let scroll = ui.input(|i| i.smooth_scroll_delta);
                                if scroll.y > 10.0 || scroll.y < -10.0 {
                                    app.threshold = (app.threshold as f32 + scroll.y.signum())
                                        .clamp(0.0, 255.0)
                                        as i16;
                                    response.mark_changed();
                                }
                            }

                            if response.changed() {
                                // [TODO] use channels differently bc changing the value will create a new channel
                                //        and the old receiver will be dropped, so the thread is sending on a closed
                                //        channel
                                app.reload_image(ctx, app.images.selected);
                            }
                        });
                    });
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Minimal Pore Size");
                        });
                        row.col(|ui| {
                            ui.style_mut().spacing.slider_width = 250.0;

                            let response = ui.horizontal(|ui| {
                                let low_drag_response = ui.add(
                                    DragValue::new(&mut app.minimal_pore_size_low)
                                        .speed(0.1)
                                        .fixed_decimals(0),
                                );

                                let slider_response = ui.add(
                                    DoubleSlider::new(
                                        &mut app.minimal_pore_size_low,
                                        &mut app.minimal_pore_size_high,
                                        0.0..=1000.0,
                                    )
                                    .width(250.0)
                                    .separation_distance(1.0),
                                );

                                let high_drag_response = ui.add(
                                    DragValue::new(&mut app.minimal_pore_size_high)
                                        .speed(0.1)
                                        .fixed_decimals(0),
                                );

                                (low_drag_response, slider_response, high_drag_response)
                            });

                            if response.inner.0.changed()
                                || response.inner.1.changed()
                                || response.inner.2.changed()
                            {
                                app.reload_image(ctx, app.images.selected);

                                log::info!(
                                    "min pore size bounds: {} - {}",
                                    app.minimal_pore_size_low,
                                    app.minimal_pore_size_high
                                );
                            }
                        });
                    });
                });

            if ui.button("Reset Region").clicked() {
                log::info!("Reset Region");
                let selected_img = app.images.selected.unwrap_or(0);
                app.images.images[selected_img].region_start = None;
                app.images.images[selected_img].region_end = None;

                app.reload_image(ctx, app.images.selected);
            }

            if ui.button("Download Results").clicked() {
                log::info!("Download Results");
                app.export_window_open = true;
            }
            egui::Window::new("Export Results")
                .open(&mut app.export_window_open)
                .show(ctx, |ui| {
                    let result_table = TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .column(Column::initial(150.0).clip(true))
                        .column(Column::initial(100.0))
                        .column(Column::initial(100.0))
                        .column(Column::initial(150.0))
                        .column(Column::initial(150.0))
                        .column(Column::initial(150.0))
                        .column(Column::initial(150.0).clip(true))
                        .header(30.0, |mut header| {
                            header.col(|ui| {
                                ui.heading("Filename");
                            });
                            header.col(|ui| {
                                ui.heading("Density");
                            });
                            header.col(|ui| {
                                ui.heading("Threshold");
                            });
                            header.col(|ui| {
                                ui.heading("Lower Pore Size");
                            });
                            header.col(|ui| {
                                ui.heading("Upper Pore Size");
                            });
                            header.col(|ui| {
                                ui.heading("Selected Region");
                            });
                            header.col(|ui| {
                                ui.heading("File Path");
                            });
                        });

                    result_table.body(|body| {
                        body.rows(25.0, app.images.images.len(), |mut row| {
                            let current_image = &app.images.images[row.index()];

                            row.col(|ui| {
                                if let Some(path) = &current_image.path {
                                    ui.label(path.file_name().unwrap().to_str().unwrap());
                                } else {
                                    ui.label("No File");
                                }
                            });
                            row.col(|ui| {
                                if let Some(density) = current_image.density {
                                    ui.label(format!("{:.5}%", density));
                                } else {
                                    ui.label("No Density");
                                }
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", app.threshold));
                            });
                            row.col(|ui| {
                                ui.label(format!("{:.0}", app.minimal_pore_size_low));
                            });
                            row.col(|ui| {
                                ui.label(format!("{:.0}", app.minimal_pore_size_high));
                            });
                            row.col(|ui| {
                                if let (Some(start), Some(end)) =
                                    (current_image.region_start, current_image.region_end)
                                {
                                    ui.label(format!(
                                        "({:.2}, {:.2}) - ({:.2}, {:.2})",
                                        start.x, start.y, end.x, end.y
                                    ));
                                } else {
                                    ui.label("No Region");
                                }
                            });
                            row.col(|ui| {
                                if let Some(path) = &current_image.path {
                                    ui.label(path.to_str().unwrap());
                                } else {
                                    ui.label("No File");
                                }
                            });
                        });
                    });
                });

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
                                    app.reload_image(ctx, Some(i));
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

                            app.images.selected = Some(0);
                            app.image_paths = Some(paths.clone());

                            app.reload_image(ctx, None);
                        }
                    }
                });
            }
        });
}
