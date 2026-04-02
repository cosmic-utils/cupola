pub mod annotate;
pub mod renderer;

use cosmic::iced::{Point, Rectangle, Size, mouse};
use image::DynamicImage;
use tiny_skia::Pixmap;
use std::{any::Any, fmt::Debug};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotateDirection {
    Left,
    Right,
}

/// A tool operation that can be rendered as an overlay and applied to an image.
///
/// Committed operations live in the undo/redo stack.
/// Active tool previews implement this trait for rendering but are never
/// committed to the stack.
pub trait ToolOperation: Debug {
    /// Render the operation onto a tiny_skia Pixmap for display overlay.
    fn render(&self, pixmap: &mut Pixmap, image_size: Size, scale: f32);

    /// Apply the operation destructively to the image pixels.
    fn apply(&self, image: &mut DynamicImage);

    /// Produce the committed operation from this preview, if applicable.
    fn commit(&self) -> Option<Box<dyn ToolOperation>>;

    /// Downcast support for tool-specific config.
    fn as_any(&self) -> &dyn Any;

    /// Mutable downcast support for tool-specific config.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Called on left mouse press.
    fn on_press(&mut self, point: Point, image_size: Size) -> mouse::Interaction {
        let _ = (point, image_size);
        mouse::Interaction::default()
    }

    /// Called on mouse drag while pressed.
    fn on_drag(&mut self, point: Point, image_size: Size) {
        let _ = (point, image_size);
    }

    /// Called on mouse release.
    fn on_release(&mut self, point: Point, image_size: Size) {
        let _ = (point, image_size);
    }

    /// Returns the cursor to show when hovering at this point.
    fn cursor_at(&self, point: Point) -> mouse::Interaction {
        let _ = point;
        mouse::Interaction::default()
    }

    /// Called when the viewport zoom level changes.
    fn on_zoom_changed(&mut self, old_zoom: f32, new_zoom: f32, image_size: Size) {
        let _ = (old_zoom, new_zoom, image_size);
    }

    /// Transform this operation's coordinates for a rotation.
    fn transform_rotate(&mut self, _direction: RotateDirection, _image_size: Size) {}

    /// Transform for horizontal flip.
    fn transform_flip_horizontal(&mut self, _image_size: Size) {}

    /// Transform for vertical flip.
    fn transform_flip_vertical(&mut self, _image_size: Size) {}

    /// Transform this operation's coordinates for a crop.
    fn transform_crop(&mut self, _region: Rectangle) {}

    /// Returns the bounding box of the operation, if it supports selection.
    fn bounds(&self) -> Option<Rectangle> {
        None
    }

    /// Whether this operation supports hit-testing and dragging.
    fn movable(&self) -> bool {
        false
    }

    /// Hit-test a point against this operation.
    fn hit_test(&self, point: Point) -> bool {
        self.bounds().is_some_and(|b| b.contains(point))
    }

    /// Move this operation by a delta.
    fn translate(&mut self, _dx: f32, _dy: f32) {}
}
