use cosmic::{
    Element, Renderer,
    iced::{
        Alignment, Background, Border, Color, Event, Length, Point, Radians, Rectangle, Size,
        advanced::{
            Clipboard, Layout, Shell,
            image::Renderer as ImageRenderer,
            layout,
            renderer::Style as RendererStyle,
            widget::Tree,
            Widget,
        },
        event::Status,
        mouse,
        widget::image::{FilterMethod, Handle},
    },
    theme,
    widget::{
        Space, button, column, container, row, slider, text, text_input,
    },
};

const GRADIENT_WIDTH: u32 = 240;
const GRADIENT_HEIGHT: u32 = 160;
const PICKER_WIDTH: f32 = 280.0;

// ── Color math ──

/// Convert HSV to RGB. h: 0-360, s: 0-1, v: 0-1
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h as u32 {
        0..60 => (c, x, 0.0),
        60..120 => (x, c, 0.0),
        120..180 => (0.0, c, x),
        180..240 => (0.0, x, c),
        240..300 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

pub fn hsv_to_color(h: f32, s: f32, v: f32, a: f32) -> Color {
    let (r, g, b) = hsv_to_rgb(h, s, v);
    Color::from_rgba8(r, g, b, a)
}

pub fn color_to_hex(c: Color) -> String {
    let r = (c.r * 255.0) as u8;
    let g = (c.g * 255.0) as u8;
    let b = (c.b * 255.0) as u8;
    format!("{r:02x}{g:02x}{b:02x}")
}

pub fn hex_to_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(Color::from_rgb8(r, g, b))
}

/// Render the HSV gradient to RGBA bytes at the given brightness.
fn render_gradient(brightness: f32) -> Vec<u8> {
    let w = GRADIENT_WIDTH as usize;
    let h = GRADIENT_HEIGHT as usize;
    let mut data = vec![0u8; w * h * 4];

    for y in 0..h {
        let sat = 1.0 - (y as f32 / (h - 1) as f32);
        for x in 0..w {
            let hue = (x as f32 / (w - 1) as f32) * 360.0;
            let (r, g, b) = hsv_to_rgb(hue, sat, brightness);
            let idx = (y * w + x) * 4;
            data[idx] = r;
            data[idx + 1] = g;
            data[idx + 2] = b;
            data[idx + 3] = 255;
        }
    }

    data
}

// ── HsvGradient custom widget ──

/// Persisted drag state for the gradient widget.
#[derive(Default)]
struct GradientState {
    dragging: bool,
}

/// Render gradient with anti-aliased crosshair via tiny_skia.
fn render_gradient_with_crosshair(brightness: f32, hue: f32, sat: f32) -> Vec<u8> {
    use tiny_skia::{
        Paint, PathBuilder, Pixmap, PremultipliedColorU8, Stroke, Transform as SkTransform,
    };

    let w = GRADIENT_WIDTH;
    let h = GRADIENT_HEIGHT;

    let mut pixmap = Pixmap::new(w, h).expect("valid gradient dimensions");

    // Draw gradient pixels
    let grad = render_gradient(brightness);
    for y in 0..h as usize {
        for x in 0..w as usize {
            let idx = (y * w as usize + x) * 4;
            let pixel = PremultipliedColorU8::from_rgba(
                grad[idx], grad[idx + 1], grad[idx + 2], grad[idx + 3],
            )
            .expect("valid color");
            pixmap.pixels_mut()[y * w as usize + x] = pixel;
        }
    }

    // Crosshair position
    let cx = (hue / 360.0) * (w - 1) as f32;
    let cy = (1.0 - sat) * (h - 1) as f32;

    // Outer ring (dark, anti-aliased)
    if let Some(path) = {
        let mut pb = PathBuilder::new();
        pb.push_circle(cx, cy, 6.0);
        pb.finish()
    } {
        let mut paint = Paint::default();
        paint.set_color_rgba8(0, 0, 0, 180);
        paint.anti_alias = true;
        let stroke = Stroke { width: 2.0, ..Stroke::default() };
        pixmap.stroke_path(&path, &paint, &stroke, SkTransform::identity(), None);
    }

    // Inner ring (white, anti-aliased)
    if let Some(path) = {
        let mut pb = PathBuilder::new();
        pb.push_circle(cx, cy, 4.0);
        pb.finish()
    } {
        let mut paint = Paint::default();
        paint.set_color_rgba8(255, 255, 255, 255);
        paint.anti_alias = true;
        let stroke = Stroke { width: 1.5, ..Stroke::default() };
        pixmap.stroke_path(&path, &paint, &stroke, SkTransform::identity(), None);
    }

    // Convert premultiplied back to straight RGBA
    let pixels = pixmap.pixels();
    let mut out = vec![0u8; (w * h * 4) as usize];
    for (i, px) in pixels.iter().enumerate() {
        let a = px.alpha() as f32 / 255.0;
        let idx = i * 4;
        if a > 0.0 {
            out[idx] = (px.red() as f32 / a).min(255.0) as u8;
            out[idx + 1] = (px.green() as f32 / a).min(255.0) as u8;
            out[idx + 2] = (px.blue() as f32 / a).min(255.0) as u8;
            out[idx + 3] = px.alpha();
        }
    }

    out
}

/// Custom widget: HSV gradient square with crosshair and mouse picking.
pub struct HsvGradient<Message> {
    width: f32,
    height: f32,
    hue: f32,
    sat: f32,
    brightness: f32,
    on_pick: Box<dyn Fn(f32, f32) -> Message>,
}

impl<Message> HsvGradient<Message> {
    pub fn new(
        hue: f32,
        sat: f32,
        brightness: f32,
        on_pick: impl Fn(f32, f32) -> Message + 'static,
    ) -> Self {
        Self {
            width: GRADIENT_WIDTH as f32,
            height: GRADIENT_HEIGHT as f32,
            hue,
            sat,
            brightness,
            on_pick: Box::new(on_pick),
        }
    }

    fn point_to_hs(&self, pos: Point, bounds: Rectangle) -> (f32, f32) {
        let x = ((pos.x - bounds.x) / bounds.width).clamp(0.0, 1.0);
        let y = ((pos.y - bounds.y) / bounds.height).clamp(0.0, 1.0);
        (x * 360.0, 1.0 - y)
    }
}

impl<Message: Clone + 'static> Widget<Message, cosmic::Theme, Renderer> for HsvGradient<Message> {
    fn tag(&self) -> cosmic::iced::advanced::widget::tree::Tag {
        cosmic::iced::advanced::widget::tree::Tag::of::<GradientState>()
    }

    fn state(&self) -> cosmic::iced::advanced::widget::tree::State {
        cosmic::iced::advanced::widget::tree::State::new(GradientState::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(self.width), Length::Fixed(self.height))
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(self.width, self.height))
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &cosmic::Theme,
        _style: &RendererStyle,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        // Gradient with crosshair baked in
        let data = render_gradient_with_crosshair(self.brightness, self.hue, self.sat);
        let handle = Handle::from_rgba(GRADIENT_WIDTH, GRADIENT_HEIGHT, data);

        ImageRenderer::draw_image(
            renderer,
            handle,
            FilterMethod::Nearest,
            bounds,
            Radians(0.0),
            1.0,
            [0.0; 4],
        );
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> Status {
        let bounds = layout.bounds();
        let grad_state = state.state.downcast_mut::<GradientState>();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position() {
                    if bounds.contains(pos) {
                        grad_state.dragging = true;
                        let (h, s) = self.point_to_hs(pos, bounds);
                        shell.publish((self.on_pick)(h, s));
                        return Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if grad_state.dragging {
                    if let Some(pos) = cursor.position() {
                        let (h, s) = self.point_to_hs(pos, bounds);
                        shell.publish((self.on_pick)(h, s));
                        return Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if grad_state.dragging {
                    grad_state.dragging = false;
                    return Status::Captured;
                }
            }
            _ => {}
        }

        Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if let Some(pos) = cursor.position() {
            if layout.bounds().contains(pos) {
                return mouse::Interaction::Crosshair;
            }
        }
        mouse::Interaction::default()
    }
}

impl<'a, Message: Clone + 'static> From<HsvGradient<Message>> for Element<'a, Message> {
    fn from(gradient: HsvGradient<Message>) -> Self {
        Self::new(gradient)
    }
}

// ── Color picker assembly ──

#[allow(clippy::too_many_arguments)]
pub fn color_picker<Message: Clone + 'static>(
    hue: f32,
    sat: f32,
    bright: f32,
    alpha: f32,
    hex_input: String,
    recent: Vec<Color>,
    on_hue_sat: impl Fn(f32, f32) -> Message + 'static,
    on_brightness: impl Fn(f32) -> Message + 'static,
    on_alpha: impl Fn(f32) -> Message + 'static,
    on_hex: impl Fn(String) -> Message + 'static,
    on_ok: impl Fn(Color) -> Message + 'static,
    on_recent: impl Fn(Color) -> Message + 'static,
) -> Element<'static, Message> {
    let current = hsv_to_color(hue, sat, bright, alpha);

    // Gradient widget with mouse picking
    let gradient = HsvGradient::new(hue, sat, bright, on_hue_sat);

    // Brightness slider
    let bright_row = row()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(text::body("B").size(11).width(Length::Fixed(14.0)))
        .push(
            slider(0.0..=1.0, bright, on_brightness)
                .step(0.01)
                .width(Length::Fill),
        )
        .push(
            text::body(format!("{:.0}%", bright * 100.0))
                .size(10)
                .width(Length::Fixed(32.0)),
        );

    // Alpha slider
    let alpha_row = row()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(text::body("A").size(11).width(Length::Fixed(14.0)))
        .push(
            slider(0.0..=1.0, alpha, on_alpha)
                .step(0.01)
                .width(Length::Fill),
        )
        .push(
            text::body(format!("{:.0}%", alpha * 100.0))
                .size(10)
                .width(Length::Fixed(32.0)),
        );

    // Hex input + preview swatch + OK
    let hex_row = row()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(text::body("#").size(12))
        .push(
            text_input("ff0000", hex_input)
                .on_input(on_hex)
                .width(Length::Fixed(80.0))
                .size(12),
        )
        .push(
            container(Space::new(20.0, 20.0)).class(theme::Container::custom(move |_| {
                container::Style {
                    background: Some(Background::Color(current)),
                    border: Border {
                        radius: 4.0.into(),
                        width: 1.0,
                        color: Color::from_rgba(1.0, 1.0, 1.0, 0.3),
                    },
                    ..Default::default()
                }
            })),
        )
        .push(button::suggested("OK").on_press(on_ok(current)));

    // Recent colors
    let mut recent_row = row().spacing(4);
    for c in recent {
        recent_row = recent_row.push(
            cosmic::widget::mouse_area(
                container(Space::new(16.0, 16.0)).class(theme::Container::custom(move |_| {
                    container::Style {
                        background: Some(Background::Color(c)),
                        border: Border {
                            radius: 8.0.into(),
                            width: 1.0,
                            color: Color::from_rgba(1.0, 1.0, 1.0, 0.2),
                        },
                        ..Default::default()
                    }
                })),
            )
            .on_press(on_recent(c)),
        );
    }

    // Assemble
    let content = column()
        .spacing(8)
        .padding(12)
        .width(Length::Fixed(PICKER_WIDTH))
        .push(Element::from(gradient))
        .push(bright_row)
        .push(alpha_row)
        .push(hex_row)
        .push(recent_row);

    container(content)
        .class(theme::Container::Dialog)
        .into()
}
