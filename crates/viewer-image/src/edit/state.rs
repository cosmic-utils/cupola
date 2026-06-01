use image::DynamicImage;
use std::path::PathBuf;
use viewer_tools::ToolOperation;
use viewer_tools::annotate::AnnotateTool;
use viewer_types::CropRegion;

/// A transformation to an image
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transform {
    Rotate90,
    Rotate180,
    Rotate270,
    FlipHorizontal,
    FlipVertical,
}

#[derive(Debug)]
pub struct EditState {
    pub original_path: Option<PathBuf>,
    pub transforms: Vec<Transform>,
    pub crop: Option<CropRegion>,
    pub is_modified: bool,
    pub operations: Vec<Box<dyn ToolOperation>>,
    pub redo_stack: Vec<Box<dyn ToolOperation>>,
    pub active_preview: Option<Box<dyn ToolOperation>>,
    pub active_tool: Option<AnnotateTool>,
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
            operations: Vec::new(),
            redo_stack: Vec::new(),
            active_preview: None,
            active_tool: None,
        }
    }

    pub fn start_editing(&mut self, path: PathBuf) {
        self.original_path = Some(path);
        self.transforms.clear();
        self.crop = None;
        self.is_modified = false;
        self.operations.clear();
        self.redo_stack.clear();
        self.active_preview = None;
        self.active_tool = None;
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
        self.operations.clear();
        self.redo_stack.clear();
        self.active_preview = None;
        self.active_tool = None;
    }

    pub fn reset(&mut self) {
        self.original_path = None;
        self.clear_edits();
    }

    pub fn is_editing(&self) -> bool {
        self.original_path.is_some()
    }

    pub fn commit_preview(&mut self) {
        if let Some(preview) = self.active_preview.take()
            && let Some(committed) = preview.commit()
        {
            self.operations.push(committed);
            self.redo_stack.clear();
            self.is_modified = true;
        }
    }

    pub fn set_preview(&mut self, preview: Option<Box<dyn ToolOperation>>) {
        self.active_preview = preview;
    }

    pub fn cancel_tool(&mut self) {
        self.active_preview = None;
    }

    pub fn undo(&mut self) -> bool {
        if let Some(op) = self.operations.pop() {
            self.redo_stack.push(op);
            self.is_modified =
                !self.operations.is_empty() || !self.transforms.is_empty() || self.crop.is_some();
            return true;
        }
        if self.transforms.pop().is_some() {
            self.is_modified =
                !self.transforms.is_empty() || !self.operations.is_empty() || self.crop.is_some();
            return true;
        }
        false
    }

    pub fn redo(&mut self) -> bool {
        if let Some(op) = self.redo_stack.pop() {
            self.operations.push(op);
            self.is_modified = true;
            true
        } else {
            false
        }
    }

    pub fn apply_all(&self, image: &mut DynamicImage) {
        for op in &self.operations {
            op.apply(image);
        }
    }

    pub fn has_operations(&self) -> bool {
        !self.operations.is_empty()
    }
}
