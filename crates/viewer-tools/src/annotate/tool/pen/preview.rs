use crate::{
    ToolOperation,
    renderer::{build_path, stroke_on_pixmap},
};
use cosmic::iced::{Color, Point, Size, mouse};
use image::DynamicImage;
use tiny_skia::{LineCap, LineJoin, Pixmap};
use std::any::Any;

use super::PenOperation;

#[derive(Debug, Clone)]
pub struct PenPreview {
    pub points: Vec<Point>,
    pub color: Color,
    pub width: f32,
}

impl PenPreview {
    pub fn new(color: Color, width: f32) -> Self {
        Self {
            points: Vec::new(),
            color,
            width,
        }
    }
}

impl ToolOperation for PenPreview {
    fn render(&self, pixmap: &mut Pixmap, _image_size: Size, scale: f32) {
        if self.points.len() < 2 {
            return;
        }

        let Some(path) = build_path(|pb| {
            pb.move_to(self.points[0].x, self.points[0].y);
            for point in &self.points[1..] {
                pb.line_to(point.x, point.y);
            }
        }) else {
            return;
        };

        stroke_on_pixmap(
            pixmap,
            &path,
            self.color,
            self.width / scale,
            LineCap::Round,
            LineJoin::Round,
        );
    }

    fn apply(&self, _image: &mut DynamicImage) {}

    fn commit(&self) -> Option<Box<dyn ToolOperation>> {
        if self.points.len() >= 2 {
            Some(Box::new(PenOperation {
                points: self.points.clone(),
                color: self.color,
                width: self.width,
            }))
        } else {
            None
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn on_press(&mut self, point: Point, _image_size: Size) -> mouse::Interaction {
        self.points.clear();
        self.points.push(point);
        mouse::Interaction::Crosshair
    }

    fn on_drag(&mut self, point: Point, _image_size: Size) {
        self.points.push(point);
    }

    fn on_release(&mut self, _point: Point, _image_size: Size) {}
}
