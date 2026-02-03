//! A self-contained crop widget that renders the image and crop UI together.
//! This ensures all coordinates are consistent since everything is handled internally.

use viewer_types::{CropSelection, DragHandle};
use cosmic::{
    Element, Renderer,
    iced::{
        Color, Length, Point, Rectangle, Size,
        advanced::{
            Clipboard, Layout, Shell, Widget,
            image::Renderer as ImageRenderer,
            layout::{Limits, Node},
            renderer::{Quad, Renderer as QuadRenderer},
            widget::Tree,
        },
        event::{Event, Status},
        mouse::{self, Button, Cursor},
    },
    widget::image::Handle,
};

const HANDLE_SIZE: f32 = 12.0;
const HANDLE_HIT_SIZE: f32 = 24.0;
const OVERLAY_COLOR: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.5);
const HANDLE_COLOR: Color = Color::WHITE;
const BORDER_COLOR: Color = Color::WHITE;
const BORDER_WIDTH: f32 = 2.0;

/// A self-contained widget that renders an image with crop selection UI.
/// Handles image rendering, overlay, selection border, resize handles, and all mouse events.
pub struct CropWidget<'a, Message> {
    /// The image handle to render
    handle: Handle,
    img_width: u32,
    img_height: u32,
    selection: CropSelection,
    on_drag_start: Option<Box<dyn Fn(f32, f32, DragHandle) -> Message + 'a>>,
    on_drag_move: Option<Box<dyn Fn(f32, f32) -> Message + 'a>>,
    on_drag_end: Option<Message>,
}

impl<'a, Message> CropWidget<'a, Message> {
    pub fn new(handle: Handle, img_width: u32, img_height: u32, selection: &CropSelection) -> Self {
        Self {
            handle,
            img_width,
            img_height,
            selection: selection.clone(),
            on_drag_start: None,
            on_drag_move: None,
            on_drag_end: None,
        }
    }

    /// Set the callback for when dragging starts
    pub fn on_drag_start<F>(mut self, f: F) -> Self
    where
        F: Fn(f32, f32, DragHandle) -> Message + 'a,
    {
        self.on_drag_start = Some(Box::new(f));
        self
    }

    /// Set the callback for when drag moves
    pub fn on_drag_move<F>(mut self, f: F) -> Self
    where
        F: Fn(f32, f32) -> Message + 'a,
    {
        self.on_drag_move = Some(Box::new(f));
        self
    }

    /// Set the message for when dragging ends
    pub fn on_drag_end(mut self, message: Message) -> Self {
        self.on_drag_end = Some(message);
        self
    }
