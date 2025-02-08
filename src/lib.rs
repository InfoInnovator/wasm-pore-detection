#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use model::detection_app::PoreDetectionApp;

pub mod model;
pub mod view;
