use std::path::PathBuf;

use calamine::{open_workbook, DataType, Reader, Xlsx};
use egui::Modal;
use egui_plot::PlotPoint;

use crate::{
    model::{
        detection_app::{self, load_texture_into_ctx},
        image_data::ImageData,
    },
    PoreDetectionApp,
};

pub fn display_load_project_modal(ctx: &egui::Context, app: &mut detection_app::PoreDetectionApp) {
    if app.load_project_model_open {
        Modal::new("load_project_modal".into()).show(ctx, |ui| {
            ui.heading("Load project");

            ui.label("This will load a project from a file and erase all current data.");

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    app.load_project_model_open = false;
                }

                ui.add_space(8.0);

                // if ui.button("Load Project").clicked() {
                //     let path = rfd::FileDialog::new()
                //         .add_filter("Excel", &["xlsx"])
                //         .pick_file();
                //     app.load_project_model_open = false;

                //     if let Some(path) = path {
                //         log::info!("Loading project from file: {:?}", path);

                //         let mut workbook: Xlsx<_> =
                //             open_workbook(path.as_path()).expect("Cannot open xlsx file");

                //         if let Ok(range) = workbook.worksheet_range("Sheet1") {
                //             let mut new_app = PoreDetectionApp::default();

                //             for row in range.rows().skip(1) {
                //                 let density = row[1].get_string().unwrap().parse().unwrap();
                //                 let threshold: i16 = row[2].get_string().unwrap().parse().unwrap();
                //                 let included_min_feature_size: f32 =
                //                     row[3].get_string().unwrap().parse().unwrap();
                //                 let minimal_pore_size_low: f32 =
                //                     row[4].get_string().unwrap().parse().unwrap();
                //                 let minimal_pore_size_high: f32 =
                //                     row[5].get_string().unwrap().parse().unwrap();
                //                 let path: PathBuf = row[7].get_string().unwrap().into();
                //                 let region = row[6]
                //                     .get_string()
                //                     .unwrap()
                //                     .split(" - ")
                //                     .collect::<Vec<_>>();

                //                 let region_start_clean =
                //                     region[0].replace("(", "").replace(")", "");
                //                 let region_start_str =
                //                     region_start_clean.split(", ").collect::<Vec<_>>();
                //                 let region_start = PlotPoint::new(
                //                     region_start_str[0].parse::<f32>().unwrap(),
                //                     region_start_str[1].parse::<f32>().unwrap(),
                //                 );

                //                 let region_end_clean = region[1].replace("(", "").replace(")", "");
                //                 let region_end_str =
                //                     region_end_clean.split(", ").collect::<Vec<_>>();
                //                 let region_end = PlotPoint::new(
                //                     region_end_str[0].parse::<f32>().unwrap(),
                //                     region_end_str[1].parse::<f32>().unwrap(),
                //                 );

                //                 let image = image::open(&path).unwrap();
                //                 let texture_handle = load_texture_into_ctx(ctx, &image);

                //                 let new_image_data = ImageData {
                //                     image: Some(image),
                //                     image_handle: Some(texture_handle),
                //                     path: Some(path),
                //                     density: Some(density),
                //                     threshold,
                //                     minimal_pore_size_low,
                //                     minimal_pore_size_high,
                //                     included_min_feature_size,
                //                     region_start: Some(region_start),
                //                     region_end: Some(region_end),
                //                     ..Default::default()
                //                 };

                //                 new_app.images.images.push(new_image_data);
                //             }

                //             log::info!(
                //                 "Loaded project with {} images",
                //                 new_app.images.images.len()
                //             );

                //             new_app.images.selected = Some(0);

                //             *app = new_app;
                //             app.image_to_display = app.images.images[0].image_handle.clone();
                //         }
                //     }
                // }
            });
        });
    }
}
