use crate::ToolOperation;
use cosmic::iced::{Color, Point, Size, mouse};
use image::DynamicImage;
use tiny_skia::Pixmap;
use std::any::Any;

use super::{ShapeKind, ShapeOperation, render_shape};

#[derive(Debug, Clone)]
pub struct ShapePreview {
    pub kind: ShapeKind,
    pub start: Option<Point>,
    pub end: Option<Point>,
    pub color: Color,
    pub width: f32,
}

impl ShapePreview {
    pub fn new(kind: ShapeKind, color: Color, width: f32) -> Self {
        Self {
            kind,
            start: None,
            end: None,
            color,
            width,
        }
    }
}

impl ToolOperation for ShapePreview {
    fn render(&self, pixmap: &mut Pixmap, _image_size: Size, scale: f32) {
        if let (Some(start), Some(end)) = (self.start, self.end) {
            render_shape(self.kind, start, end, self.color, self.width, pixmap, scale);
        }
    }

    fn apply(&self, _image: &mut DynamicImage) {}

    fn commit(&self) -> Option<Box<dyn ToolOperation>> {
        let (start, end) = match (self.start, self.end) {
            (Some(s), Some(e)) if (s.x - e.x).abs() > 1.0 || (s.y - e.y).abs() > 1.0 => (s, e),
            _ => return None,
        };

        Some(Box::new(ShapeOperation::new(
            self.kind, start, end, self.color, self.width,
        )))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn on_press(&mut self, point: Point, _image_size: Size) -> mouse::Interaction {
        self.start = Some(point);
        self.end = None;
        mouse::Interaction::Crosshair
    }

    fn on_drag(&mut self, point: Point, _image_size: Size) {
        self.end = Some(point);
    }
}
