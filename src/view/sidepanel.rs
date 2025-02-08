use std::sync::mpsc;

use egui::Color32;
use egui_extras::{Column, TableBuilder};
use rfd::FileDialog;

use crate::{
    model::{self, image_data::ImageData},
    PoreDetectionApp,
};

pub fn display_sidepanel(ctx: &egui::Context, app: &mut PoreDetectionApp) {
    egui::SidePanel::new(egui::panel::Side::Left, "sidebar")
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Options");

            TableBuilder::new(ui)
                .id_salt("options_table")
                .column(Column::auto())
                .column(Column::auto())
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

                            let response =
                                ui.add(egui::Slider::new(&mut app.threshold, 0..=255).step_by(1.0));

                            if response.changed() {
                                // [TODO] use channels differently bc changing the value will create a new channel
                                //        and the old receiver will be dropped, so the thread is sending on a closed
                                //        channel
                                let (tx, rx) = mpsc::channel();
                                app.receiver = Some(rx);

                                let selected_img = app.images.selected.unwrap_or(0);
                                // if let Some(image) = &app.images.images[selected_img] {
                                //     // analyze_image(
                                //     //     tx,
                                //     //     image.clone(),
                                //     //     app.threshold,
                                //     //     app.minimal_pore_size,
                                //     // );
                                // } else {
                                //     log::warn!("No image selected");
                                // }
                                app.images.images[selected_img].analyze_image(
                                    tx,
                                    app.threshold,
                                    app.minimal_pore_size,
                                );
                            }
                        });
                    });
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Minimal Pore Size");
                        });
                        row.col(|ui| {
                            ui.style_mut().spacing.slider_width = 250.0;
                            let response =
                                ui.add(egui::Slider::new(&mut app.minimal_pore_size, 0..=75));

                            if response.changed() {
                                let (tx, rx) = mpsc::channel();
                                app.receiver = Some(rx);

                                // if let Some(image) =
                                //     &self.images[self.selected_image.unwrap_or(0)].image
                                // {
                                //     self.analyze_image(image.clone(), tx);
                                // }
                                let selected_img = app.images.selected.unwrap_or(0);
                                app.images.images[selected_img].analyze_image(
                                    tx,
                                    app.threshold,
                                    app.minimal_pore_size,
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
            }

            if ui.button("Apply to Batch").clicked() {
                log::info!("Apply to Batch");
            }

            if ui.button("Download Results").clicked() {
                log::info!("Download Results");
            }

            if let Some(selected_img) = app.images.selected {
                if let Some(density) = app.images.images[selected_img].density {
                    ui.heading(format!("Density: {:.5}%", density));
                } else {
                    ui.heading("Density: -".to_string());
                }
            }

            ui.separator();

            ui.heading("Image List");

            // [TODO] make async so the ui is not blocked
            if let Some(folder_path) = &app.image_paths {
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
                                row.col(|ui| {
                                    ui.image(format!("file://{}", path.to_str().unwrap()));
                                });
                                row.col(|ui| {
                                    let resp = ui.heading(path_str);

                                    ui.label(format!(
                                        "Density: {:.5}%",
                                        app.images.images[i].density.unwrap_or(0.0),
                                    ));

                                    if resp.clicked() {
                                        app.images.selected = Some(i);
                                        let image = image::open(path).unwrap();
                                        app.image_to_display = Some(
                                            model::detection_app::PoreDetectionApp::load_texture(
                                                ctx, &image,
                                            ),
                                        );
                                        app.images.images[i].image = Some(image.clone());

                                        log::info!("Selected Image: {:?}", app.images.selected);
                                    }
                                });
                            });
                        }
                    });
            } else {
                ui.vertical_centered(|ui| {
                    if ui.button("Open Files").clicked() {
                        let path = FileDialog::new().pick_files();
                        if let Some(paths) = path {
                            for path in &paths {
                                app.images.images.push(ImageData {
                                    path: Some(path.to_path_buf()),
                                    ..Default::default()
                                });
                            }

                            app.images.selected = Some(0);
                            app.image_paths = Some(paths.clone());

                            let image = image::open(&paths[0]).unwrap();
                            app.images.images[0].image = Some(image.clone());
                            app.image_to_display =
                                Some(model::detection_app::PoreDetectionApp::load_texture(
                                    ctx,
                                    &image.clone(),
                                ));
                        }
                    }
                });
            }
        });
}
