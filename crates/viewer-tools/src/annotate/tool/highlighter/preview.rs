use crate::{
    ToolOperation,
    renderer::{build_path, stroke_on_pixmap},
};
use cosmic::iced::{Color, Point, Size, mouse};
use image::DynamicImage;
use tiny_skia::{LineCap, LineJoin, Pixmap};
use std::any::Any;

use super::HighlighterOperation;

const HIGHLIGHT_ALPHA: f32 = 0.35;

#[derive(Debug, Clone)]
pub struct HighlighterPreview {
    pub points: Vec<Point>,
    pub color: Color,
    pub width: f32,
}

impl HighlighterPreview {
    pub fn new(color: Color, width: f32) -> Self {
        Self {
            points: Vec::new(),
            color,
            width,
        }
    }

    fn highlight_color(&self) -> Color {
        Color::from_rgba(
            self.color.r,
            self.color.g,
            self.color.b,
            self.color.a * HIGHLIGHT_ALPHA,
        )
    }
}

impl ToolOperation for HighlighterPreview {
    fn render(&self, pixmap: &mut Pixmap, _image_size: Size, _scale: f32) {
        if self.points.len() < 2 {
            return;
        }

        let Some(path) = build_path(|pb| {
            pb.move_to(self.points[0].x, self.points[0].y);

            if self.points.len() == 2 {
                pb.line_to(self.points[1].x, self.points[1].y);
            } else {
                let mid_x = (self.points[0].x + self.points[1].x) / 2.0;
                let mid_y = (self.points[0].y + self.points[1].y) / 2.0;
                pb.line_to(mid_x, mid_y);

                for idx in 1..self.points.len() - 1 {
                    let ctrl = self.points[idx];
                    let next = self.points[idx + 1];
                    let end_x = (ctrl.x + next.x) / 2.0;
                    let end_y = (ctrl.y + next.y) / 2.0;
                    pb.quad_to(ctrl.x, ctrl.y, end_x, end_y);
                }

                let last = self.points.last().unwrap();
                pb.line_to(last.x, last.y);
            }
        }) else {
            return;
        };

        stroke_on_pixmap(
            pixmap,
            &path,
            self.highlight_color(),
            self.width,
            LineCap::Square,
            LineJoin::Round,
        );
    }

    fn apply(&self, _image: &mut DynamicImage) {}

    fn commit(&self) -> Option<Box<dyn ToolOperation>> {
        if self.points.len() >= 2 {
            Some(Box::new(HighlighterOperation {
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
        if let Some(last) = self.points.last() {
            let dx = point.x - last.x;
            let dy = point.y - last.y;
            let min_dist = self.width * 0.5;

            if dx * dx + dy * dy < min_dist * min_dist {
                return;
            }
        }
        self.points.push(point);
    }

    fn on_release(&mut self, _point: Point, _image_size: Size) {}
}
