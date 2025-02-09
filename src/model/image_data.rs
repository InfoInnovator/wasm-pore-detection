use std::{collections::HashMap, path::PathBuf, sync::mpsc};

use egui_plot::PlotPoint;
use image::{DynamicImage, Luma};
use imageproc::definitions::{HasBlack, HasWhite};

#[derive(Default)]
pub struct ImageData {
    pub path: Option<PathBuf>,
    pub image: Option<DynamicImage>,
    pub density: Option<f64>,
    pub black_pixels: Option<Vec<PlotPoint>>,
    pub region_start: Option<PlotPoint>,
    pub region_end: Option<PlotPoint>,
}

impl ImageData {
    pub fn analyze_image(
        &mut self,
        tx: mpsc::Sender<(Vec<PlotPoint>, f64)>,
        threshold: i16,
        minimal_pore_size: i16,
    ) {
        let image = self.image.clone().unwrap();
        let region_start = self.region_start;
        let region_end = self.region_end;

        std::thread::spawn(move || {
            let grayscale = image.grayscale().to_luma8();
            let grayscale_thresh = imageproc::contrast::threshold(
                &grayscale,
                threshold.try_into().unwrap(),
                imageproc::contrast::ThresholdType::Binary,
            );

            // find connected groups of white pixels
            let labels = imageproc::region_labelling::connected_components(
                &grayscale_thresh,
                imageproc::region_labelling::Connectivity::Eight,
                Luma::white(),
            );

            // count the pixels for each group/label
            let mut labels_to_size: HashMap<u32, i32> = HashMap::new();
            labels.enumerate_pixels().for_each(|(_, _, p)| {
                let count = labels_to_size.entry(p[0]).or_insert(0);
                *count += 1;
            });

            // draw a green pixel for each black pixel that is part of a group with a size greater than the users minimal pore size
            let mut black_pixels = Vec::new();
            if let (Some(region_start), Some(region_end)) = (region_start, region_end) {
                labels.enumerate_pixels().for_each(|(x, y, p)| {
                    let y_start = image.height() - region_start.y as u32;
                    let y_end = image.height() - region_end.y as u32;

                    if grayscale_thresh.get_pixel(x, y) == &Luma::black()
                        && labels_to_size[&p[0]] > minimal_pore_size.into()
                        && x >= region_start.x as u32
                        && x <= region_end.x as u32
                        && y >= y_start
                        && y <= y_end
                    {
                        black_pixels.push(PlotPoint::new(x, y));
                    }
                });
            } else {
                labels.enumerate_pixels().for_each(|(x, y, p)| {
                    if grayscale_thresh.get_pixel(x, y) == &Luma::black()
                        && labels_to_size[&p[0]] > minimal_pore_size.into()
                    {
                        black_pixels.push(PlotPoint::new(x, y));
                    }
                });
            }
            log::info!("pushed black pixels: {:?}", black_pixels.len());

            // calculate the density for the whole image
            let density;
            if let (Some(start), Some(end)) = (region_start, region_end) {
                density = (1.0
                    - (black_pixels.len() as f64
                        / ((f64::abs(end.x - start.x)) * f64::abs(end.y - start.y))))
                    * 100.0;
            } else {
                density = (1.0
                    - (black_pixels.len() as f64
                        / (grayscale.width() * grayscale.height()) as f64))
                    * 100.0;
            }

            // send the black pixels and the density to the main thread
            if let Err(err) = tx.send((black_pixels.clone(), density)) {
                log::error!("Error sending data to another thread: {}", err);
            }
        });
    }
}
