use super::image_data::ImageData;

#[derive(Default)]
pub struct Images {
    pub images: Vec<ImageData>,
    pub selected: Option<usize>,
}

impl Images {
    pub fn export(&self) -> String {
        self.images
            .iter()
            .map(|img_data| {
                if let (Some(path), Some(density)) = (&img_data.path, img_data.density) {
                    let region = img_data.region_start.is_some() && img_data.region_end.is_some();

                    format!(
                        "{},region:{},{:.5}%\n",
                        path.file_name().unwrap().to_str().unwrap(),
                        region,
                        density,
                    )
                } else {
                    String::new()
                }
            })
            .collect()
    }
}
