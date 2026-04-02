use crate::{RotateDirection, ToolOperation};
use crate::renderer::{build_path, fill_on_pixmap};

use cosmic::iced::{Color, Point, Rectangle, Size};
use image::DynamicImage;
use tiny_skia::Pixmap;

use std::any::Any;

const OVERLAY_COLOR: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.5);

#[derive(Debug, Clone)]
pub struct CropOperation {
    pub region: Rectangle,
}

impl CropOperation {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            region: Rectangle::new(Point::new(x, y), Size::new(width, height)),
        }
    }
}

impl ToolOperation for CropOperation {
    fn render(&self, pixmap: &mut Pixmap, image_size: Size, _scale: f32) {
        let img_w = image_size.width;
        let img_h = image_size.height;
        let r = &self.region;

        // Top strip
        if r.y > 0.0
            && let Some(path) = build_rect_path(0.0, 0.0, img_w, r.y)
        {
            fill_on_pixmap(pixmap, &path, OVERLAY_COLOR);
        }

        // Bottom strip
        let bottom = r.y + r.height;
        if bottom < img_h
            && let Some(path) = build_rect_path(0.0, bottom, img_w, img_h - bottom)
        {
            fill_on_pixmap(pixmap, &path, OVERLAY_COLOR);
        }

        // Left strip (between top and bottom)
        if r.x > 0.0
            && let Some(path) = build_rect_path(0.0, r.y, r.x, r.height)
        {
            fill_on_pixmap(pixmap, &path, OVERLAY_COLOR);
        }

        // Right strip (between top and bottom)
        let right = r.x + r.width;
        if right < img_w
            && let Some(path) = build_rect_path(right, r.y, img_w - right, r.height)
        {
            fill_on_pixmap(pixmap, &path, OVERLAY_COLOR);
        }
    }

    fn apply(&self, image: &mut DynamicImage) {
        let r = &self.region;
        let x = (r.x as u32).min(image.width().saturating_sub(1));
        let y = (r.y as u32).min(image.height().saturating_sub(1));
        let w = (r.width as u32).min(image.width().saturating_sub(x));
        let h = (r.height as u32).min(image.height().saturating_sub(y));

        if w > 0 && h > 0 {
            *image = image.crop_imm(x, y, w, h);
        }
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
        let (w, h) = (image_size.width, image_size.height);
        let r = &self.region;
        let (rx, ry, rw, rh) = (r.x, r.y, r.width, r.height);

        self.region = match direction {
            RotateDirection::Left => {
                Rectangle::new(Point::new(ry, w - rx - rw), Size::new(rh, rw))
            }
            RotateDirection::Right => {
                Rectangle::new(Point::new(h - ry - rh, rx), Size::new(rh, rw))
            }
        };
    }

    fn transform_flip_horizontal(&mut self, image_size: Size) {
        self.region.x = image_size.width - self.region.x - self.region.width;
    }

    fn transform_flip_vertical(&mut self, image_size: Size) {
        self.region.y = image_size.height - self.region.y - self.region.height;
    }

    fn transform_crop(&mut self, region: Rectangle) {
        self.region.x -= region.x;
        self.region.y -= region.y;
    }
}

fn build_rect_path(x: f32, y: f32, w: f32, h: f32) -> Option<tiny_skia::Path> {
    build_path(|pb| {
        if let Some(r) = tiny_skia::Rect::from_xywh(x, y, w, h) {
            pb.push_rect(r);
        }
    })
}
