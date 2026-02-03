pub mod types;

pub use types::{CropRegion, CropSelection, DragHandle};

/// ThumbnailMetadata for viewer
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThumbnailMetadata {
    pub source_path: std::path::PathBuf,
    pub thumbnail_path: std::path::PathBuf,
    pub original_width: u32,
    pub original_height: u32,
    pub aspect_ratio: f64,
    pub screen_reader_label: String,
    pub accessibility_alt_text: String,
}

impl ThumbnailMetadata {
    pub fn new(
        source_path: std::path::PathBuf,
        thumbnail_path: std::path::PathBuf,
        original_width: u32,
        original_height: u32,
        aspect_ratio: f64,
        screen_reader_label: String,
        accessibility_alt_text: String,
    ) -> Self {
        Self {
            source_path,
            thumbnail_path,
            original_width,
            original_height,
            aspect_ratio,
            screen_reader_label,
            accessibility_alt_text,
        }
    }
}

/// Transform enum for image transformations
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Transform {
    Rotate90,
    Rotate180,
    FlipHorizontal,
    FlipVertical,
}

impl Transform {
    pub const NONE: Option<Transform> = None;
}
