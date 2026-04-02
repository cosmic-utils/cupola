use crate::ToolOperation;
use crate::renderer::{build_path, fill_on_pixmap, stroke_on_pixmap};
use super::CropOperation;

use cosmic::iced::{Color, Point, Size, mouse};
use image::DynamicImage;
use tiny_skia::{LineCap, LineJoin, Pixmap, Rect};

use std::any::Any;

const HANDLE_SIZE: f32 = 12.0;
const HANDLE_HIT_SIZE: f32 = 24.0;
const OVERLAY_COLOR: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.5);
const HANDLE_COLOR: Color = Color::WHITE;
const BORDER_COLOR: Color = Color::WHITE;
const BORDER_WIDTH: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CropDragHandle {
    #[default]
    None,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Bottom,
    Left,
    Right,
    Move,
}

#[derive(Debug, Clone)]
pub struct CropPreview {
    pub region: Option<(f32, f32, f32, f32)>,
    pub is_dragging: bool,
    pub drag_handle: CropDragHandle,
    pub drag_start: Option<(f32, f32)>,
    pub drag_start_region: Option<(f32, f32, f32, f32)>,
}

impl CropPreview {
    pub fn new() -> Self {
        Self {
            region: None,
            is_dragging: false,
            drag_handle: CropDragHandle::None,
            drag_start: None,
            drag_start_region: None,
        }
    }

    pub fn has_selection(&self) -> bool {
        self.region
            .map(|(_, _, w, h)| w > 1.0 && h > 1.0)
            .unwrap_or(false)
    }

    fn hit_test_handle(&self, point: Point, scale: f32) -> CropDragHandle {
        let Some((rx, ry, rw, rh)) = self.region else {
            return CropDragHandle::None;
        };

        let corners = [
            (rx, ry, CropDragHandle::TopLeft),
            (rx + rw, ry, CropDragHandle::TopRight),
            (rx, ry + rh, CropDragHandle::BottomLeft),
            (rx + rw, ry + rh, CropDragHandle::BottomRight),
        ];

        for (hx, hy, handle) in corners {
            if point_in_handle(point, hx, hy, scale) {
                return handle;
            }
        }

        let edges = [
            (rx + rw / 2.0, ry, CropDragHandle::Top),
            (rx + rw / 2.0, ry + rh, CropDragHandle::Bottom),
            (rx, ry + rh / 2.0, CropDragHandle::Left),
            (rx + rw, ry + rh / 2.0, CropDragHandle::Right),
        ];

        for (hx, hy, handle) in edges {
            if point_in_handle(point, hx, hy, scale) {
                return handle;
            }
        }

        // Check if inside selection (for move)
        if point.x >= rx && point.x <= rx + rw && point.y >= ry && point.y <= ry + rh {
            return CropDragHandle::Move;
        }

        CropDragHandle::None
    }

    fn update_drag(&mut self, x: f32, y: f32, img_width: f32, img_height: f32) {
        if !self.is_dragging {
            return;
        }

        match self.drag_handle {
            CropDragHandle::None => {
                if let Some((start_x, start_y)) = self.drag_start {
                    let min_x = start_x.min(x).max(0.0);
                    let min_y = start_y.min(y).max(0.0);
                    let max_x = start_x.max(x).min(img_width);
                    let max_y = start_y.max(y).min(img_height);

                    self.region = Some((min_x, min_y, max_x - min_x, max_y - min_y));
                }
            }
            CropDragHandle::Move => {
                if let (Some((start_x, start_y)), Some((rx, ry, rw, rh))) =
                    (self.drag_start, self.drag_start_region)
                {
                    let dx = x - start_x;
                    let dy = y - start_y;
                    let new_x = (rx + dx).max(0.0).min(img_width - rw);
                    let new_y = (ry + dy).max(0.0).min(img_height - rh);
                    self.region = Some((new_x, new_y, rw, rh));
                }
            }
            _ => {
                if let Some((rx, ry, rw, rh)) = self.drag_start_region {
                    let resized =
                        resize_region(self.drag_handle, rx, ry, rw, rh, x, y, img_width, img_height);
                    self.region = Some(resized);
                }
            }
        }
    }

    fn render_overlay(&self, pixmap: &mut Pixmap, image_size: Size) {
        let img_w = image_size.width;
        let img_h = image_size.height;

        if let Some((rx, ry, rw, rh)) = self.region
            && rw > 0.0 && rh > 0.0
        {
            // Top strip
            if ry > 0.0 {
                fill_rect(pixmap, 0.0, 0.0, img_w, ry, OVERLAY_COLOR);
            }

            // Bottom strip
            let bottom = ry + rh;
            if bottom < img_h {
                fill_rect(pixmap, 0.0, bottom, img_w, img_h - bottom, OVERLAY_COLOR);
            }

            // Left strip
            if rx > 0.0 {
                fill_rect(pixmap, 0.0, ry, rx, rh, OVERLAY_COLOR);
            }

            // Right strip
            let right = rx + rw;
            if right < img_w {
                fill_rect(pixmap, right, ry, img_w - right, rh, OVERLAY_COLOR);
            }

            return;
        }

        // No valid selection -- full overlay
        fill_rect(pixmap, 0.0, 0.0, img_w, img_h, OVERLAY_COLOR);
    }

    fn render_border(&self, pixmap: &mut Pixmap, scale: f32) {
        let Some((rx, ry, rw, rh)) = self.region else {
            return;
        };

        if rw <= 0.0 || rh <= 0.0 {
            return;
        }

        let Some(path) = build_path(|pb| {
            if let Some(r) = Rect::from_xywh(rx, ry, rw, rh) {
                pb.push_rect(r);
            }
        }) else {
            return;
        };

        stroke_on_pixmap(
            pixmap,
            &path,
            BORDER_COLOR,
            BORDER_WIDTH / scale,
            LineCap::Square,
            LineJoin::Miter,
        );
    }

    fn render_handles(&self, pixmap: &mut Pixmap, scale: f32) {
        let Some((rx, ry, rw, rh)) = self.region else {
            return;
        };

        if rw <= 0.0 || rh <= 0.0 {
            return;
        }

        let half = HANDLE_SIZE / (2.0 * scale);
        let size = HANDLE_SIZE / scale;

        let positions = [
            (rx, ry),
            (rx + rw, ry),
            (rx, ry + rh),
            (rx + rw, ry + rh),
            (rx + rw / 2.0, ry),
            (rx + rw / 2.0, ry + rh),
            (rx, ry + rh / 2.0),
            (rx + rw, ry + rh / 2.0),
        ];

        for (hx, hy) in positions {
            let Some(path) = build_path(|pb| {
                if let Some(r) = Rect::from_xywh(hx - half, hy - half, size, size) {
                    pb.push_rect(r);
                }
            }) else {
                continue;
            };
            fill_on_pixmap(pixmap, &path, HANDLE_COLOR);
        }
    }
}

impl Default for CropPreview {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolOperation for CropPreview {
    fn render(&self, pixmap: &mut Pixmap, image_size: Size, scale: f32) {
        self.render_overlay(pixmap, image_size);
        self.render_border(pixmap, scale);
        self.render_handles(pixmap, scale);
    }

    fn apply(&self, _image: &mut DynamicImage) {}

    fn commit(&self) -> Option<Box<dyn ToolOperation>> {
        let (rx, ry, rw, rh) = self.region?;

        if rw <= 1.0 || rh <= 1.0 {
            return None;
        }

        Some(Box::new(CropOperation::new(rx, ry, rw, rh)))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn on_press(&mut self, point: Point, image_size: Size) -> mouse::Interaction {
        let _ = image_size;
        let scale = 1.0;
        let handle = self.hit_test_handle(point, scale);

        if handle != CropDragHandle::None {
            self.is_dragging = true;
            self.drag_handle = handle;
            self.drag_start = Some((point.x, point.y));
            self.drag_start_region = self.region;
        } else {
            // Start new selection
            self.region = Some((point.x, point.y, 0.0, 0.0));
            self.is_dragging = true;
            self.drag_handle = CropDragHandle::None;
            self.drag_start = Some((point.x, point.y));
            self.drag_start_region = None;
        }

        cursor_for_handle(self.drag_handle)
    }

    fn on_drag(&mut self, point: Point, image_size: Size) {
        self.update_drag(point.x, point.y, image_size.width, image_size.height);
    }

    fn on_release(&mut self, _point: Point, _image_size: Size) {
        self.is_dragging = false;
        self.drag_start = None;
        self.drag_start_region = None;
    }

    fn cursor_at(&self, point: Point) -> mouse::Interaction {
        if self.is_dragging {
            return cursor_for_handle(self.drag_handle);
        }

        let handle = self.hit_test_handle(point, 1.0);
        cursor_for_handle(handle)
    }
}

fn cursor_for_handle(handle: CropDragHandle) -> mouse::Interaction {
    match handle {
        CropDragHandle::None => mouse::Interaction::Crosshair,
        CropDragHandle::TopLeft | CropDragHandle::BottomRight => {
            mouse::Interaction::ResizingDiagonallyDown
        }
        CropDragHandle::TopRight | CropDragHandle::BottomLeft => {
            mouse::Interaction::ResizingDiagonallyUp
        }
        CropDragHandle::Top | CropDragHandle::Bottom => mouse::Interaction::ResizingVertically,
        CropDragHandle::Left | CropDragHandle::Right => mouse::Interaction::ResizingHorizontally,
        CropDragHandle::Move => mouse::Interaction::Grabbing,
    }
}

fn point_in_handle(point: Point, hx: f32, hy: f32, scale: f32) -> bool {
    let half = HANDLE_HIT_SIZE / (2.0 * scale);
    point.x >= hx - half
        && point.x <= hx + half
        && point.y >= hy - half
        && point.y <= hy + half
}

fn fill_rect(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, color: Color) {
    let Some(path) = build_path(|pb| {
        if let Some(r) = Rect::from_xywh(x, y, w, h) {
            pb.push_rect(r);
        }
    }) else {
        return;
    };
    fill_on_pixmap(pixmap, &path, color);
}

#[allow(clippy::too_many_arguments)]
fn resize_region(
    handle: CropDragHandle,
    rx: f32,
    ry: f32,
    rw: f32,
    rh: f32,
    x: f32,
    y: f32,
    img_width: f32,
    img_height: f32,
) -> (f32, f32, f32, f32) {
    let right = rx + rw;
    let bottom = ry + rh;
    let x = x.max(0.0).min(img_width);
    let y = y.max(0.0).min(img_height);

    match handle {
        CropDragHandle::TopLeft => {
            let new_x = x.min(right - 10.0);
            let new_y = y.min(bottom - 10.0);
            (new_x, new_y, right - new_x, bottom - new_y)
        }
        CropDragHandle::TopRight => {
            let new_right = x.max(rx + 10.0);
            let new_y = y.min(bottom - 10.0);
            (rx, new_y, new_right - rx, bottom - new_y)
        }
        CropDragHandle::BottomLeft => {
            let new_x = x.min(right - 10.0);
            let new_bottom = y.max(ry + 10.0);
            (new_x, ry, right - new_x, new_bottom - ry)
        }
        CropDragHandle::BottomRight => {
            let new_right = x.max(rx + 10.0);
            let new_bottom = y.max(ry + 10.0);
            (rx, ry, new_right - rx, new_bottom - ry)
        }
        CropDragHandle::Top => {
            let new_y = y.min(bottom - 10.0);
            (rx, new_y, rw, bottom - new_y)
        }
        CropDragHandle::Bottom => {
            let new_bottom = y.max(ry + 10.0);
            (rx, ry, rw, new_bottom - ry)
        }
        CropDragHandle::Left => {
            let new_x = x.min(right - 10.0);
            (new_x, ry, right - new_x, rh)
        }
        CropDragHandle::Right => {
            let new_right = x.max(rx + 10.0);
            (rx, ry, new_right - rx, rh)
        }
        _ => (rx, ry, rw, rh),
    }
}
