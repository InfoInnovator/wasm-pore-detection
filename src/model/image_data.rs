use std::{path::PathBuf, thread::JoinHandle};

use egui::TextureHandle;
use egui_plot::PlotPoint;
use image::{DynamicImage, Luma};
use imageproc::definitions::{HasBlack, HasWhite};

#[derive(Clone)]
pub struct ImageData {
    pub path: Option<PathBuf>,
    pub image: Option<DynamicImage>,
    pub image_handle: Option<TextureHandle>,
    pub density: Option<f64>,
    pub green_pixels: Option<Vec<PlotPoint>>,
    pub region_start: Option<PlotPoint>,
    pub region_end: Option<PlotPoint>,
    pub threshold: i16,
    pub minimal_pore_size_low: f32,
    pub minimal_pore_size_high: f32,
    pub included_min_feature_size: f32,
}

impl Default for ImageData {
    fn default() -> Self {
        Self {
            path: Default::default(),
            image: Default::default(),
            image_handle: Default::default(),
            density: Default::default(),
            green_pixels: Default::default(),
            region_start: Default::default(),
            region_end: Default::default(),
            threshold: Default::default(),
            minimal_pore_size_low: 0.0,
            minimal_pore_size_high: i32::MAX as f32,
            included_min_feature_size: 0.0,
        }
    }
}

impl ImageData {
    pub fn analyze_image(
        &mut self,
    ) -> JoinHandle<(
        std::vec::Vec<egui_plot::PlotPoint>,
        std::vec::Vec<egui_plot::PlotPoint>,
        f64,
    )> {
        let image = self.image.clone().unwrap();
        let region_start = self.region_start;
        let region_end = self.region_end;
        let threshold = self.threshold;
        let minimal_pore_size_low = self.minimal_pore_size_low;
        let minimal_pore_size_high = self.minimal_pore_size_high;
        let included_min_feature_size = self.included_min_feature_size;

        let handle = std::thread::spawn(move || {
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
            let num_labels = labels.iter().max().unwrap_or(&0);
            let mut labels_to_size = vec![0; *num_labels as usize + 1];
            labels.enumerate_pixels().for_each(|(_, _, p)| {
                labels_to_size[p[0] as usize] += 1;
            });

            // find connected groups of black pixels
            let black_labels = imageproc::region_labelling::connected_components(
                &grayscale_thresh,
                imageproc::region_labelling::Connectivity::Eight,
                Luma::black(),
            );

            // count the pixels for each group/label
            let num_labels_black = black_labels.iter().max().unwrap_or(&0);
            let mut black_labels_to_size = vec![0; *num_labels_black as usize + 1];
            black_labels.enumerate_pixels().for_each(|(_, _, p)| {
                black_labels_to_size[p[0] as usize] += 1;
            });

            // draw a green pixel for each black pixel that is part of a group with a size greater than the users minimal pore size
            let mut green_pixels = Vec::new();
            let mut white_pixels = Vec::new();
            if let (Some(region_start), Some(region_end)) = (region_start, region_end) {
                labels.enumerate_pixels().for_each(|(x, y, p)| {
                    let y_start = image.height() - region_start.y as u32;
                    let y_end = image.height() - region_end.y as u32;

                    if grayscale_thresh.get_pixel(x, y) == &Luma::black()
                        && labels_to_size[p[0] as usize] > minimal_pore_size_low as i32
                        && labels_to_size[p[0] as usize] < minimal_pore_size_high as i32
                        && x >= region_start.x as u32
                        && x <= region_end.x as u32
                        && y >= y_start
                        && y <= y_end
                    {
                        green_pixels.push(PlotPoint::new(x, y));
                    }

                    if grayscale_thresh.get_pixel(x, y) == &Luma::white()
                        && x >= region_start.x as u32
                        && x <= region_end.x as u32
                        && y >= y_start
                        && y <= y_end
                    {
                        white_pixels.push(PlotPoint::new(x, y));
                    }
                });

                if included_min_feature_size > 0.0 {
                    black_labels.enumerate_pixels().for_each(|(x, y, p)| {
                        let y_start = image.height() - region_start.y as u32;
                        let y_end = image.height() - region_end.y as u32;

                        if black_labels_to_size[p[0] as usize] < included_min_feature_size as i32
                            && x >= region_start.x as u32
                            && x <= region_end.x as u32
                            && y >= y_start
                            && y <= y_end
                        {
                            green_pixels.push(PlotPoint::new(x, y));

                            if white_pixels.contains(&PlotPoint::new(x, y)) {
                                white_pixels.retain(|p| p != &PlotPoint::new(x, y));
                            }
                        }
                    });
                }
            } else {
                labels.enumerate_pixels().for_each(|(x, y, p)| {
                    if grayscale_thresh.get_pixel(x, y) == &Luma::black()
                        && labels_to_size[p[0] as usize] > minimal_pore_size_low as i32
                        && labels_to_size[p[0] as usize] < minimal_pore_size_high as i32
                    {
                        green_pixels.push(PlotPoint::new(x, y));
                    }

                    if grayscale_thresh.get_pixel(x, y) == &Luma::white() {
                        white_pixels.push(PlotPoint::new(x, y));
                    }
                });

                if included_min_feature_size > 0.0 {
                    black_labels.enumerate_pixels().for_each(|(x, y, p)| {
                        if black_labels_to_size[p[0] as usize] < included_min_feature_size as i32 {
                            green_pixels.push(PlotPoint::new(x, y));

                            if white_pixels.contains(&PlotPoint::new(x, y)) {
                                white_pixels.retain(|p| p != &PlotPoint::new(x, y));
                            }
                        }
                    });
                }
            }
            log::info!("pushed black pixels: {:?}", green_pixels.len());

            // calculate the density for the whole image
            let density = (1.0 - (green_pixels.len() as f64 / white_pixels.len() as f64)) * 100.0;

            (green_pixels, white_pixels, density)
        });

        handle
    }
}
