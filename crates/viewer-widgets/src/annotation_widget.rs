use cosmic::{
    Element, Renderer,
    iced::{
        Length, Point, Rectangle, Size, Vector,
        advanced::{
            Clipboard, Layout, Shell, Widget,
            image::Renderer as ImageRenderer,
            layout::{Limits, Node},
            widget::Tree,
        },
        event::{Event, Status},
        mouse::{self, Button, Cursor},
    },
    widget::image::Handle,
};

use std::cell::Cell;

pub struct AnnotationWidget<'a, Message> {
    handle: Handle,
    img_width: u32,
    img_height: u32,
    committed_overlay: Option<Handle>,
    preview_overlay: Option<Handle>,
    zoom: f32,
    pan: Vector,
    on_tool_start: Option<Box<dyn Fn(Point) -> Message + 'a>>,
    on_tool_drag: Option<Box<dyn Fn(Point) -> Message + 'a>>,
    on_tool_end: Option<Box<dyn Fn() -> Message + 'a>>,
    is_dragging: Cell<bool>,
}

impl<'a, Message> AnnotationWidget<'a, Message> {
    pub fn new(handle: Handle, img_width: u32, img_height: u32) -> Self {
        Self {
            handle,
            img_width,
            img_height,
            committed_overlay: None,
            preview_overlay: None,
            zoom: 1.0,
            pan: Vector::ZERO,
            on_tool_start: None,
            on_tool_drag: None,
            on_tool_end: None,
            is_dragging: Cell::new(false),
        }
    }

    pub fn committed_overlay(mut self, handle: Handle) -> Self {
        self.committed_overlay = Some(handle);
        self
    }

    pub fn preview_overlay(mut self, handle: Handle) -> Self {
        self.preview_overlay = Some(handle);
        self
    }

    pub fn zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom;
        self
    }

    pub fn pan(mut self, pan: Vector) -> Self {
        self.pan = pan;
        self
    }

    pub fn on_tool_start(mut self, f: impl Fn(Point) -> Message + 'a) -> Self {
        self.on_tool_start = Some(Box::new(f));
        self
    }

    pub fn on_tool_drag(mut self, f: impl Fn(Point) -> Message + 'a) -> Self {
        self.on_tool_drag = Some(Box::new(f));
        self
    }

    pub fn on_tool_end(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_tool_end = Some(Box::new(f));
        self
    }

    fn image_rect(&self, bounds: Rectangle) -> Rectangle {
        let fit_scale = (bounds.width / self.img_width as f32)
            .min(bounds.height / self.img_height as f32);
        let effective_scale = self.zoom * fit_scale;

        let img_w = self.img_width as f32 * effective_scale;
        let img_h = self.img_height as f32 * effective_scale;

        let center_x = bounds.x + bounds.width / 2.0;
        let center_y = bounds.y + bounds.height / 2.0;

        let img_x = center_x - img_w / 2.0 + self.pan.x;
        let img_y = center_y - img_h / 2.0 + self.pan.y;

        Rectangle::new(Point::new(img_x, img_y), Size::new(img_w, img_h))
    }

    fn screen_to_image(&self, bounds: Rectangle, screen_point: Point) -> Point {
        let fit_scale = (bounds.width / self.img_width as f32)
            .min(bounds.height / self.img_height as f32);
        let effective_scale = self.zoom * fit_scale;

        let center_x = bounds.x + bounds.width / 2.0;
        let center_y = bounds.y + bounds.height / 2.0;

        let img_x = (screen_point.x - center_x - self.pan.x) / effective_scale
            + self.img_width as f32 / 2.0;
        let img_y = (screen_point.y - center_y - self.pan.y) / effective_scale
            + self.img_height as f32 / 2.0;

        Point::new(img_x, img_y)
    }
}

impl<'a, Message: Clone> Widget<Message, cosmic::Theme, Renderer> for AnnotationWidget<'a, Message> {
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
        let img_rect = self.image_rect(bounds);

        renderer.draw_image(
            self.handle.clone(),
            cosmic::iced::widget::image::FilterMethod::Linear,
            img_rect,
            cosmic::iced::Radians(0.0),
            1.0,
            [0.0; 4],
        );

        if let Some(ref overlay) = self.committed_overlay {
            renderer.draw_image(
                overlay.clone(),
                cosmic::iced::widget::image::FilterMethod::Linear,
                img_rect,
                cosmic::iced::Radians(0.0),
                1.0,
                [0.0; 4],
            );
        }

        if let Some(ref overlay) = self.preview_overlay {
            renderer.draw_image(
                overlay.clone(),
                cosmic::iced::widget::image::FilterMethod::Linear,
                img_rect,
                cosmic::iced::Radians(0.0),
                1.0,
                [0.0; 4],
            );
        }
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

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(Button::Left)) => {
                if let Some(pos) = cursor.position() {
                    let img_rect = self.image_rect(bounds);
                    if !img_rect.contains(pos) {
                        return Status::Ignored;
                    }

                    self.is_dragging.set(true);
                    let img_pt = self.screen_to_image(bounds, pos);
                    if let Some(ref cb) = self.on_tool_start {
                        shell.publish(cb(img_pt));
                    }
                    return Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if self.is_dragging.get()
                    && let Some(pos) = cursor.position()
                {
                    let img_pt = self.screen_to_image(bounds, pos);
                    if let Some(ref cb) = self.on_tool_drag {
                        shell.publish(cb(img_pt));
                    }
                    return Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(Button::Left)) => {
                if self.is_dragging.get() {
                    self.is_dragging.set(false);
                    if let Some(ref cb) = self.on_tool_end {
                        shell.publish(cb());
                    }
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
        if self.is_dragging.get() {
            return mouse::Interaction::Crosshair;
        }

        if let Some(pos) = cursor.position() {
            let img_rect = self.image_rect(layout.bounds());
            if img_rect.contains(pos) {
                return mouse::Interaction::Crosshair;
            }
        }

        mouse::Interaction::default()
    }
}

impl<'a, Message: Clone + 'a> From<AnnotationWidget<'a, Message>> for Element<'a, Message> {
    fn from(widget: AnnotationWidget<'a, Message>) -> Self {
        Self::new(widget)
    }
}

pub fn annotation_widget<'a, Message>(
    handle: Handle,
    width: u32,
    height: u32,
) -> AnnotationWidget<'a, Message> {
    AnnotationWidget::new(handle, width, height)
}
