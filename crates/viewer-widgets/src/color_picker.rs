use cosmic::{
    Element,
    iced::{Alignment, Color, Length},
    theme,
    widget::{button, column, container, row, slider, text, text_input, tooltip},
};

const PICKER_WIDTH: f32 = 260.0;
const SWATCH_SIZE: f32 = 24.0;
const RECENT_SIZE: f32 = 20.0;

const HUE_PRESETS: [(f32, &str); 12] = [
    (0.0, "Red"),
    (30.0, "Orange"),
    (60.0, "Yellow"),
    (90.0, "Chartreuse"),
    (120.0, "Green"),
    (150.0, "Spring"),
    (180.0, "Cyan"),
    (210.0, "Azure"),
    (240.0, "Blue"),
    (270.0, "Violet"),
    (300.0, "Magenta"),
    (330.0, "Rose"),
];

pub fn hsb_to_color(hue: f32, sat: f32, bright: f32, alpha: f32) -> Color {
    let c = bright * sat;
    let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let m = bright - c;
    let (r, g, b) = match hue as u32 {
        0..60 => (c, x, 0.0),
        60..120 => (x, c, 0.0),
        120..180 => (0.0, c, x),
        180..240 => (0.0, x, c),
        240..300 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    Color::from_rgba(r + m, g + m, b + m, alpha)
}

pub fn color_to_hex(color: Color) -> String {
    let r = (color.r * 255.0) as u8;
    let g = (color.g * 255.0) as u8;
    let b = (color.b * 255.0) as u8;
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

#[allow(clippy::too_many_arguments)]
pub fn color_picker<'a, Message: Clone + 'static>(
    hue: f32,
    sat: f32,
    bright: f32,
    alpha: f32,
    hex_input: &'a str,
    recent: &[Color],
    on_hue_sat: impl Fn(f32, f32) -> Message + 'a,
    on_brightness: impl Fn(f32) -> Message + 'a,
    on_alpha: impl Fn(f32) -> Message + 'a,
    on_hex: impl Fn(String) -> Message + 'a,
    on_ok: impl Fn(Color) -> Message + 'a,
    on_recent: impl Fn(Color) -> Message + 'a,
) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;
    let current = hsb_to_color(hue, sat, bright, alpha);

    // Hue preset buttons (12 around the wheel)
    let hue_row_top = {
        let mut r = row().spacing(2);
        for &(h, name) in &HUE_PRESETS[..6] {
            let preview = hsb_to_color(h, 1.0, 1.0, 1.0);
            let [cr, cg, cb] = [
                (preview.r * 255.0) as u8,
                (preview.g * 255.0) as u8,
                (preview.b * 255.0) as u8,
            ];
            let is_active = (hue - h).abs() < 15.0;
            let msg = on_hue_sat(h, sat);
            let mut btn = button::custom(
                text::body("\u{25A0}")
                    .class(Color::from_rgb8(cr, cg, cb))
                    .size(16),
            )
            .on_press(msg)
            .width(Length::Fixed(SWATCH_SIZE))
            .height(Length::Fixed(SWATCH_SIZE));
            if is_active {
                btn = btn.class(theme::Button::Suggested);
            }
            r = r.push(tooltip::tooltip(btn, text::body(name), tooltip::Position::Top));
        }
        r
    };

    let hue_row_bottom = {
        let mut r = row().spacing(2);
        for &(h, name) in &HUE_PRESETS[6..] {
            let preview = hsb_to_color(h, 1.0, 1.0, 1.0);
            let [cr, cg, cb] = [
                (preview.r * 255.0) as u8,
                (preview.g * 255.0) as u8,
                (preview.b * 255.0) as u8,
            ];
            let is_active = (hue - h).abs() < 15.0;
            let msg = on_hue_sat(h, sat);
            let mut btn = button::custom(
                text::body("\u{25A0}")
                    .class(Color::from_rgb8(cr, cg, cb))
                    .size(16),
            )
            .on_press(msg)
            .width(Length::Fixed(SWATCH_SIZE))
            .height(Length::Fixed(SWATCH_SIZE));
            if is_active {
                btn = btn.class(theme::Button::Suggested);
            }
            r = r.push(tooltip::tooltip(btn, text::body(name), tooltip::Position::Bottom));
        }
        r
    };

    // Saturation slider
    let sat_row = row()
        .spacing(spacing.space_xs)
        .align_y(Alignment::Center)
        .push(text::body("Sat").width(Length::Fixed(50.0)))
        .push(
            slider(0.0..=1.0, sat, move |s| on_hue_sat(hue, s))
                .width(Length::Fill)
                .step(0.01),
        );

    // Brightness slider
    let bright_row = row()
        .spacing(spacing.space_xs)
        .align_y(Alignment::Center)
        .push(text::body("Bright").width(Length::Fixed(50.0)))
        .push(
            slider(0.0..=1.0, bright, on_brightness)
                .width(Length::Fill)
                .step(0.01),
        );

    // Alpha slider
    let alpha_row = row()
        .spacing(spacing.space_xs)
        .align_y(Alignment::Center)
        .push(text::body("Alpha").width(Length::Fixed(50.0)))
        .push(
            slider(0.0..=1.0, alpha, on_alpha)
                .width(Length::Fill)
                .step(0.01),
        );

    // Hex input + preview swatch + OK button
    let [sr, sg, sb] = [
        (current.r * 255.0) as u8,
        (current.g * 255.0) as u8,
        (current.b * 255.0) as u8,
    ];

    let swatch = button::custom(
        text::body("\u{25A0}")
            .class(Color::from_rgb8(sr, sg, sb))
            .size(20),
    )
    .width(Length::Fixed(SWATCH_SIZE))
    .height(Length::Fixed(SWATCH_SIZE));

    let ok_color = current;
    let hex_row = row()
        .spacing(spacing.space_xs)
        .align_y(Alignment::Center)
        .push(text::body("Hex:").width(Length::Shrink))
        .push(
            text_input("ff0000", hex_input)
                .on_input(on_hex)
                .width(Length::Fixed(80.0)),
        )
        .push(swatch)
        .push(
            button::standard("OK")
                .on_press(on_ok(ok_color))
                .class(theme::Button::Suggested),
        );

    // Recent colors row
    let recent_row = if recent.is_empty() {
        row()
    } else {
        let mut r = row()
            .spacing(4)
            .align_y(Alignment::Center)
            .push(text::body("Recent:").width(Length::Shrink));
        for &color in recent.iter().take(8) {
            let [rr, rg, rb] = [
                (color.r * 255.0) as u8,
                (color.g * 255.0) as u8,
                (color.b * 255.0) as u8,
            ];
            r = r.push(
                button::custom(
                    text::body("\u{25CF}")
                        .class(Color::from_rgb8(rr, rg, rb))
                        .size(14),
                )
                .on_press(on_recent(color))
                .width(Length::Fixed(RECENT_SIZE))
                .height(Length::Fixed(RECENT_SIZE)),
            );
        }
        r
    };

    let content = column()
        .spacing(spacing.space_xs)
        .padding(spacing.space_s)
        .width(Length::Fixed(PICKER_WIDTH))
        .push(hue_row_top)
        .push(hue_row_bottom)
        .push(sat_row)
        .push(bright_row)
        .push(alpha_row)
        .push(hex_row)
        .push(recent_row);

    container(content).class(theme::Container::Dialog).into()
}
