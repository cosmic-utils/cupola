use std::path::PathBuf;

use viewer_types::{CropRegion, CropSelection};

/// A transformation to an image
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transform {
    Rotate90,
    Rotate180,
    FlipHorizontal,
    FlipVertical,
}

#[derive(Debug, Clone)]
pub struct EditState {
    pub original_path: Option<PathBuf>,
    pub transforms: Vec<Transform>,
    pub crop: Option<CropRegion>,
    pub is_modified: bool,
    pub is_cropping: bool,
    pub crop_selection: CropSelection,
}

impl Default for EditState {
    fn default() -> Self {
        Self::new()
    }
}

impl EditState {
    pub fn new() -> Self {
        Self {
            original_path: None,
            transforms: Vec::new(),
            crop: None,
            is_modified: false,
            is_cropping: false,
            crop_selection: CropSelection::new(),
        }
    }

    pub fn start_editing(&mut self, path: PathBuf) {
        self.original_path = Some(path);
        self.transforms.clear();
        self.crop = None;
        self.is_modified = false;
        self.is_cropping = false;
        self.crop_selection.reset();
    }

    pub fn apply_transform(&mut self, transform: Transform) {
        self.transforms.push(transform);
        self.is_modified = true;
    }

    pub fn set_crop(&mut self, region: CropRegion) {
        self.crop = Some(region);
        self.is_modified = true;
    }

    pub fn clear_edits(&mut self) {
        self.transforms.clear();
        self.crop = None;
        self.is_modified = false;
        self.is_cropping = false;
        self.crop_selection.reset();
    }

    pub fn reset(&mut self) {
        self.original_path = None;
        self.clear_edits();
    }

    pub fn is_editing(&self) -> bool {
        self.original_path.is_some()
    }

    pub fn start_crop(&mut self) {
        self.is_cropping = true;
        self.crop_selection.reset();
    }

    pub fn cancel_crop(&mut self) {
        self.is_cropping = false;
        self.crop = None;
        self.crop_selection.reset();
    }

    pub fn apply_crop(&mut self) {
        self.is_cropping = false;
        if self.crop.is_some() {
            self.is_modified = true;
        }
    }

    pub fn undo(&mut self) -> bool {
        if self.transforms.pop().is_some() {
            self.is_modified = !self.transforms.is_empty() || self.crop.is_some();
            true
        } else {
            false
        }
    }
}
