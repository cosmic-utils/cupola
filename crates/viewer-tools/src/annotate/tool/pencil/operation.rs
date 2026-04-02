use crate::{
    ToolOperation, RotateDirection,
    renderer::{build_path, stroke_on_image, stroke_on_pixmap},
};
use cosmic::iced::{Color, Point, Rectangle, Size};
use image::DynamicImage;
use tiny_skia::{LineCap, LineJoin, Pixmap};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct PencilOperation {
    pub points: Vec<Point>,
    pub color: Color,
    pub width: f32,
}

impl PencilOperation {
    fn pencil_color(&self) -> Color {
        let mut c = self.color;
        c.a *= 0.85;
        c
    }
}

impl ToolOperation for PencilOperation {
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
            self.pencil_color(),
            self.width / scale,
            LineCap::Butt,
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

        stroke_on_image(image, &path, self.pencil_color(), self.width, LineCap::Butt);
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
