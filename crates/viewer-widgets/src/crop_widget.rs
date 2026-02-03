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
}

impl<'a, Message> CropWidget<'a, Message> {
    pub fn new(handle: Handle, img_width: u32, img_height: u32, selection: &CropSelection) -> Self {
        Self {
            handle,
            img_width,
            img_height,
            selection: selection.clone(),
        }
    }

    fn calculate_image_rect(&self, bounds: Rectangle) -> (Rectangle, f32) {
        let scale_x = bounds.width / self.img_width as f32;
        let scale_y = bounds.height / self.img_height as f32;
        let scale = scale_x.min(scale_y).min(1.0); // Don't upscale

        let img_w = self.img_width as f32 * scale;
        let img_h = self.img_height as f32 * scale;

        let img_x = bounds.x + (bounds.width - img_w) / 2.0;
        let img_y = bounds.y + (bounds.height - img_h) / 2.0;

        (Rectangle::new(Point::new(img_x, img_y), Size::new(img_w, img_h)), scale)
    }

    /// Convert screen coordinates to image coordinates
    fn screen_to_image(&self, img_rect: &Rectangle, scale: f32, point: Point) -> (f32, f32) {
        let x = ((point.x - img_rect.x) / scale)
            .max(0.0)
            .min(self.img_width as f32);
        let y = ((point.y - img_rect.y) / scale)
            .max(0.0)
            .min(self.img_height as f32);
        (x, y)
    }

    /// Convert image coordinates to screen coordinates
    fn image_to_screen(&self, img_rect: &Rectangle, scale: f32, img_x: f32, img_y: f32) -> Point {
        Point::new(img_rect.x + img_x * scale, img_rect.y + img_y * scale)
    }

    /// Check which handle (if any) is at the given screen position
    fn hit_test_handle(&self, img_rect: &Rectangle, scale: f32, point: Point) -> DragHandle {
        let Some((rx, ry, rw, rh)) = self.selection.region else {
            return DragHandle::None;
        };

        // Corner handles (check first - higher priority)
        let corners = [
            (self.image_to_screen(img_rect, scale, rx, ry), DragHandle::TopLeft),
            (self.image_to_screen(img_rect, scale, rx + rw, ry), DragHandle::TopRight),
            (self.image_to_screen(img_rect, scale, rx, ry + rh), DragHandle::BottomLeft),
            (self.image_to_screen(img_rect, scale, rx + rw, ry + rh), DragHandle::BottomRight),
        ];

        for (pos, handle) in corners {
            if self.point_in_handle(point, pos) {
                return handle;
            }
        }

        // Edge handles
        let edges = [
            (self.image_to_screen(img_rect, scale, rx + rw / 2.0, ry), DragHandle::Top),
            (self.image_to_screen(img_rect, scale, rx + rw / 2.0, ry + rh), DragHandle::Bottom),
            (self.image_to_screen(img_rect, scale, rx, ry + rh / 2.0), DragHandle::Left),
            (self.image_to_screen(img_rect, scale, rx + rw, ry + rh / 2.0), DragHandle::Right),
        ];

        for (pos, handle) in edges {
            if self.point_in_handle(point, pos) {
                return handle;
            }
        }

        // Check if inside selection (for move)
        let top_left = self.image_to_screen(img_rect, scale, rx, ry);
        let bottom_right = self.image_to_screen(img_rect, scale, rx + rw, ry + rh);
        let selection_rect = Rectangle::new(
            top_left,
            Size::new(bottom_right.x - top_left.x, bottom_right.y - top_left.y),
        );

        if selection_rect.contains(point) {
            return DragHandle::Move;
        }

        DragHandle::None
    }

    fn point_in_handle(&self, point: Point, handle_center: Point) -> bool {
        let half = HANDLE_HIT_SIZE / 2.0;
        point.x >= handle_center.x - half
            && point.x <= handle_center.x + half
            && point.y >= handle_center.y - half
            && point.y <= handle_center.y + half
    }

    fn cursor_for_handle(&self, handle: DragHandle) -> mouse::Interaction {
        match handle {
            DragHandle::None => mouse::Interaction::Crosshair,
            DragHandle::TopLeft | DragHandle::BottomRight => mouse::Interaction::ResizingDiagonallyDown,
            DragHandle::TopRight | DragHandle::BottomLeft => mouse::Interaction::ResizingDiagonallyUp,
            DragHandle::Top | DragHandle::Bottom => mouse::Interaction::ResizingVertically,
            DragHandle::Left | DragHandle::Right => mouse::Interaction::ResizingHorizontally,
            DragHandle::Move => mouse::Interaction::Grabbing,
        }
    }

    /// Draw the dark overlay regions around the selection
    fn draw_overlay(&self, renderer: &mut Renderer, img_rect: Rectangle, scale: f32) {
        if let Some((rx, ry, rw, rh)) = self.selection.region {
            if rw > 0.0 && rh > 0.0 {
                // Selection rectangle in screen coords
                let sel_x = img_rect.x + rx * scale;
                let sel_y = img_rect.y + ry * scale;
                let sel_w = rw * scale;
                let sel_h = rh * scale;

                // Top region (full width, above selection)
                if sel_y > img_rect.y {
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle::new(
                                img_rect.position(),
                                Size::new(img_rect.width, sel_y - img_rect.y),
                            ),
                            ..Quad::default()
                        },
                        OVERLAY_COLOR,
                    );
                }

                // Bottom region (full width, below selection)
                let sel_bottom = sel_y + sel_h;
                let img_bottom = img_rect.y + img_rect.height;
                if sel_bottom < img_bottom {
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle::new(
                                Point::new(img_rect.x, sel_bottom),
                                Size::new(img_rect.width, img_bottom - sel_bottom),
                            ),
                            ..Quad::default()
                        },
                        OVERLAY_COLOR,
                    );
                }

                // Left region (between top and bottom overlays)
                if sel_x > img_rect.x {
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle::new(
                                Point::new(img_rect.x, sel_y),
                                Size::new(sel_x - img_rect.x, sel_h),
                            ),
                            ..Quad::default()
                        },
                        OVERLAY_COLOR,
                    );
                }

                // Right region (between top and bottom overlays)
                let sel_right = sel_x + sel_w;
                let img_right = img_rect.x + img_rect.width;
                if sel_right < img_right {
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle::new(
                                Point::new(sel_right, sel_y),
                                Size::new(img_right - sel_right, sel_h),
                            ),
                            ..Quad::default()
                        },
                        OVERLAY_COLOR,
                    );
                }

                return;
            }
        }

        // No valid selection - draw full overlay
        renderer.fill_quad(
            Quad {
                bounds: img_rect,
                ..Quad::default()
            },
            OVERLAY_COLOR,
        );
    }

    /// Draw the selection border
    fn draw_border(&self, renderer: &mut Renderer, img_rect: Rectangle, scale: f32) {
        let Some((rx, ry, rw, rh)) = self.selection.region else {
            return;
        };

        if rw <= 0.0 || rh <= 0.0 {
            return;
        }

        let sel_x = img_rect.x + rx * scale;
        let sel_y = img_rect.y + ry * scale;
        let sel_w = rw * scale;
        let sel_h = rh * scale;

        // Top border
        renderer.fill_quad(
            Quad {
                bounds: Rectangle::new(
                    Point::new(sel_x, sel_y),
                    Size::new(sel_w, BORDER_WIDTH),
                ),
                ..Quad::default()
            },
            BORDER_COLOR,
        );

        // Bottom border
        renderer.fill_quad(
            Quad {
                bounds: Rectangle::new(
                    Point::new(sel_x, sel_y + sel_h - BORDER_WIDTH),
                    Size::new(sel_w, BORDER_WIDTH),
                ),
                ..Quad::default()
            },
            BORDER_COLOR,
        );

        // Left border
        renderer.fill_quad(
            Quad {
                bounds: Rectangle::new(
                    Point::new(sel_x, sel_y),
                    Size::new(BORDER_WIDTH, sel_h),
                ),
                ..Quad::default()
            },
            BORDER_COLOR,
        );

        // Right border
        renderer.fill_quad(
            Quad {
                bounds: Rectangle::new(
                    Point::new(sel_x + sel_w - BORDER_WIDTH, sel_y),
                    Size::new(BORDER_WIDTH, sel_h),
                ),
                ..Quad::default()
            },
            BORDER_COLOR,
        );
    }

    /// Draw the resize handles
    fn draw_handles(&self, renderer: &mut Renderer, img_rect: Rectangle, scale: f32) {
        let Some((rx, ry, rw, rh)) = self.selection.region else {
            return;
        };

        if rw <= 0.0 || rh <= 0.0 {
            return;
        }

        let sel_x = img_rect.x + rx * scale;
        let sel_y = img_rect.y + ry * scale;
        let sel_w = rw * scale;
        let sel_h = rh * scale;

        let handle_half = HANDLE_SIZE / 2.0;
        let handles = [
            (sel_x, sel_y),                       // TopLeft
            (sel_x + sel_w, sel_y),               // TopRight
            (sel_x, sel_y + sel_h),               // BottomLeft
            (sel_x + sel_w, sel_y + sel_h),       // BottomRight
            (sel_x + sel_w / 2.0, sel_y),         // Top
            (sel_x + sel_w / 2.0, sel_y + sel_h), // Bottom
            (sel_x, sel_y + sel_h / 2.0),         // Left
            (sel_x + sel_w, sel_y + sel_h / 2.0), // Right
        ];

        for (hx, hy) in handles {
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle::new(
                        Point::new(hx - handle_half, hy - handle_half),
                        Size::new(HANDLE_SIZE, HANDLE_SIZE),
                    ),
                    ..Quad::default()
                },
                HANDLE_COLOR,
            );
        }
    }
}

impl<'a, Message: Clone> Widget<Message, cosmic::Theme, Renderer> for CropWidget<'a, Message> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, limits: &Limits) -> Node {
        Node::new(limits.max())
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &cosmic::Theme,
        _style: &cosmic::iced::advanced::renderer::Style,
        layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let (img_rect, scale) = self.calculate_image_rect(bounds);

        // Draw the image
        renderer.draw_image(
            self.handle.clone(),
            cosmic::iced::widget::image::FilterMethod::Linear,
            img_rect,
            cosmic::iced::Radians(0.0),
            1.0,
            [0.0; 4],
        );

        renderer.with_layer(img_rect, |renderer| {
            self.draw_overlay(renderer, img_rect, scale);
            self.draw_border(renderer, img_rect, scale);
            self.draw_handles(renderer, img_rect, scale);
        });
    }

    fn on_event(
        &mut self,
        _tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> Status {
        let bounds = layout.bounds();
        let (img_rect, scale) = self.calculate_image_rect(bounds);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(Button::Left)) => {
                if let Some(pos) = cursor.position() {
                    // Only handle clicks within the image area
                    if !img_rect.contains(pos) {
                        return Status::Ignored;
                    }

                    // Check for handle hits first
                    let handle = self.hit_test_handle(&img_rect, scale, pos);

                    if handle != DragHandle::None {
                        // Clicking on a handle - start resize/move
                        let (img_x, img_y) = self.screen_to_image(&img_rect, scale, pos);
                        shell.publish(Message::Edit(EditMessage::CropDragStart {
                            x: img_x,
                            y: img_y,
                            handle,
                        }));
                        return Status::Captured;
                    }

                    // Clicking on image (not on handle) - start new selection
                    let (img_x, img_y) = self.screen_to_image(&img_rect, scale, pos);
                    shell.publish(Message::Edit(EditMessage::CropDragStart {
                        x: img_x,
                        y: img_y,
                        handle: DragHandle::None,
                    }));
                    return Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if self.selection.is_dragging {
                    if let Some(pos) = cursor.position() {
                        let (img_x, img_y) = self.screen_to_image(&img_rect, scale, pos);
                        shell.publish(Message::Edit(EditMessage::CropDragMove {
                            x: img_x,
                            y: img_y,
                        }));
                        return Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(Button::Left)) => {
                if self.selection.is_dragging {
                    shell.publish(Message::Edit(EditMessage::CropDragEnd));
                    return Status::Captured;
                }
            }
            _ => {}
        }

        Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        let (img_rect, scale) = self.calculate_image_rect(bounds);

        if self.selection.is_dragging {
            return self.cursor_for_handle(self.selection.drag_handle);
        }

        if let Some(pos) = cursor.position() {
            if img_rect.contains(pos) {
                let handle = self.hit_test_handle(&img_rect, scale, pos);
                if handle != DragHandle::None {
                    return self.cursor_for_handle(handle);
                }
                return mouse::Interaction::Crosshair;
            }
        }

        mouse::Interaction::default()
    }
}

impl<'a> From<CropWidget> for Element<'a, Message> {
    fn from(widget: CropWidget) -> Self {
        Self::new(widget)
    }
}

/// Helper function to create a CropWidget
pub fn crop_widget(handle: Handle, img_width: u32, img_height: u32, selection: &CropSelection) -> CropWidget {
    CropWidget::new(handle, img_width, img_height, selection)
}
