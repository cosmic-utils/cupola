mod color;
mod common;
mod popouts;
mod properties;
mod strip;

pub use properties::properties_bar;
pub use strip::tool_strip;

use cosmic::iced::{Color, alignment::Horizontal};
use viewer_tools::annotate::{AnnotateColor, AnnotateTool, CropRatio, PenMode, TransformSubTool};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Breakpoint {
    Desktop,
    Comfortable,
    Compact,
}

impl Breakpoint {
    pub fn from_width(width: f32) -> Self {
        if width >= 1024.0 {
            Self::Desktop
        } else if width >= 600.0 {
            Self::Comfortable
        } else {
            Self::Compact
        }
    }
}

#[derive(Clone)]
pub struct AnnotationProps {
    pub tool: AnnotateTool,
    pub color: AnnotateColor,
    pub stroke_width: f32,
    pub pen_mode: PenMode,
    pub opacity: f32,
    pub fill_mode: bool,
    pub fill_color: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub font_size: f32,
    pub font_family: &'static str,
    pub alignment: Horizontal,
    pub crop_ratio: CropRatio,
    pub can_undo: bool,
    pub can_redo: bool,
    pub is_drawing: bool,
    pub active_shape: AnnotateTool,
    pub active_transform: TransformSubTool,
    pub shape_popout_open: bool,
    pub transform_popout_open: bool,
    pub color_picker_open: bool,
    pub picker_hue: f32,
    pub picker_sat: f32,
    pub picker_bright: f32,
    pub picker_alpha: f32,
    pub picker_hex: String,
    pub recent_colors: Vec<Color>,
    pub breakpoint: Breakpoint,
}
