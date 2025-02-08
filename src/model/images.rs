use super::image_data::ImageData;

#[derive(Default)]
pub struct Images {
    pub images: Vec<ImageData>,
    pub selected: Option<usize>,
}
