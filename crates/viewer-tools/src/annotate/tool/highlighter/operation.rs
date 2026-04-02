use crate::{
    ToolOperation, RotateDirection,
    renderer::{build_path, stroke_on_image, stroke_on_pixmap},
};
use cosmic::iced::{Color, Point, Rectangle, Size};
use image::DynamicImage;
use tiny_skia::{LineCap, LineJoin, Pixmap};
use std::any::Any;

const HIGHLIGHT_ALPHA: f32 = 0.35;

#[derive(Debug, Clone)]
pub struct HighlighterOperation {
    pub points: Vec<Point>,
    pub color: Color,
    pub width: f32,
}

impl HighlighterOperation {
    fn highlight_color(&self) -> Color {
        Color::from_rgba(
            self.color.r,
            self.color.g,
            self.color.b,
            self.color.a * HIGHLIGHT_ALPHA,
        )
    }
}

impl ToolOperation for HighlighterOperation {
    fn render(&self, pixmap: &mut Pixmap, _image_size: Size, scale: f32) {
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
            self.width / scale,
            LineCap::Square,
            LineJoin::Round,
        );
    }

    fn apply(&self, image: &mut DynamicImage) {
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

        stroke_on_image(
            image,
            &path,
            self.highlight_color(),
            self.width,
            LineCap::Square,
        );
    }

    fn commit(&self) -> Option<Box<dyn ToolOperation>> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn transform_rotate(&mut self, direction: RotateDirection, image_size: Size) {
        let (width, height) = (image_size.width, image_size.height);
        for point in &mut self.points {
            let (x, y) = (point.x, point.y);

            *point = match direction {
                RotateDirection::Left => Point::new(y, width - x),
                RotateDirection::Right => Point::new(height - y, x),
            };
        }
    }

    fn transform_flip_horizontal(&mut self, image_size: Size) {
        for point in &mut self.points {
            point.x = image_size.width - point.x;
        }
    }

    fn transform_flip_vertical(&mut self, image_size: Size) {
        for point in &mut self.points {
            point.y = image_size.height - point.y;
        }
    }

    fn transform_crop(&mut self, region: Rectangle) {
        for point in &mut self.points {
            point.x -= region.x;
            point.y -= region.y;
        }
    }
}
