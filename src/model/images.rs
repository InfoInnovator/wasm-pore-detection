use egui_plot::PlotPoint;
use rust_xlsxwriter::{workbook::Workbook, Table, TableColumn};

use super::image_data::ImageData;

#[derive(Default)]
pub struct Images {
    pub images: Vec<ImageData>,
    pub selected: Option<usize>,
    pub prev_selected: Option<usize>,
}

impl Images {
    pub fn export(&self) {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        for (i, image) in self.images.iter().enumerate() {
            let filename = image
                .path
                .as_ref()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();
            let density = image.density.unwrap_or(0.0);
            let threshold = image.threshold;
            let start = image.region_start.unwrap_or(PlotPoint::new(0.0, 0.0));
            let end = image.region_end.unwrap_or(PlotPoint::new(0.0, 0.0));

            let row = [
                filename,
                &format!("{:.5}", density),
                &threshold.to_string(),
                &image.minimal_pore_size_low.to_string(),
                &image.minimal_pore_size_high.to_string(),
                &format!(
                    "({:.2}, {:.2}) - ({:.2}, {:.2})",
                    start.x, start.y, end.x, end.y
                ),
                image.path.as_ref().unwrap().to_str().unwrap(),
            ];

            worksheet
                .write_row((i + 1).try_into().unwrap(), 0, row)
                .unwrap();
        }

        let columns = vec![
            TableColumn::new().set_header("Filename"),
            TableColumn::new().set_header("Density"),
            TableColumn::new().set_header("Threshold"),
            TableColumn::new().set_header("Lower Pore Size"),
            TableColumn::new().set_header("Upper Pore Size"),
            TableColumn::new().set_header("Selected Region"),
            TableColumn::new().set_header("File Path"),
        ];

        let table = Table::new()
            .set_columns(&columns)
            .set_total_row(true)
            .set_banded_rows(true);
        worksheet
            .add_table(0, 0, self.images.len().try_into().unwrap(), 6, &table)
            .unwrap();

        worksheet.autofit();
        workbook.save("demo.xlsx").unwrap();
    }
}
