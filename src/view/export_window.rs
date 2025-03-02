use core::fmt;

use egui::ComboBox;
use egui_extras::{Column, TableBuilder};

use crate::PoreDetectionApp;

pub fn display_export_window(ctx: &egui::Context, app: &mut PoreDetectionApp) {
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

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Choose number format:");
                ComboBox::from_id_salt("Number format")
                    .selected_text(format!("{}", app.export_decimal_format))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut app.export_decimal_format,
                            ExportDecimalFormat::Dot,
                            "Dot (.)",
                        );
                        ui.selectable_value(
                            &mut app.export_decimal_format,
                            ExportDecimalFormat::Comma,
                            "Comma (,)",
                        );
                    });

                if ui.button("Export Excel").clicked() {
                    app.images.export();
                }
            })
        });
}

#[derive(Default, PartialEq)]
pub enum ExportDecimalFormat {
    #[default]
    Dot,
    Comma,
}

impl fmt::Display for ExportDecimalFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportDecimalFormat::Dot => write!(f, "Dot (.)"),
            ExportDecimalFormat::Comma => write!(f, "Comma (,)"),
        }
    }
}
