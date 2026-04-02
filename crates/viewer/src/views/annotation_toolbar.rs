use crate::message::{EditMessage, Message};
use cosmic::{
    Element,
    iced::{Alignment, Length, alignment::Horizontal},
    theme,
    widget::{button, column, container, horizontal_space, icon, row, text, tooltip},
};
use viewer_tools::annotate::{AnnotateColor, AnnotateTool};

const STROKE_WIDTHS: &[f32] = &[1.0, 2.0, 4.0, 8.0];
const FONT_SIZES: &[f32] = &[12.0, 16.0, 20.0, 24.0, 32.0, 48.0];

fn tool_btn(tool: AnnotateTool, active: AnnotateTool) -> Element<'static, Message> {
    let ic = icon::from_name(tool.icon_name()).size(20);
    let btn = button::icon(ic).on_press(Message::Edit(EditMessage::SetTool(tool)));
    if tool == active {
        container(btn.class(theme::Button::Suggested))
            .into()
    } else {
        container(btn).into()
    }
}

fn color_circle(c: AnnotateColor, active: AnnotateColor) -> Element<'static, Message> {
    let [r, g, b, _] = [
        (c.0.r * 255.0) as u8,
        (c.0.g * 255.0) as u8,
        (c.0.b * 255.0) as u8,
        (c.0.a * 255.0) as u8,
    ];

    let label = format!("#{r:02x}{g:02x}{b:02x}");
    let is_active = c == active;

    let btn = if is_active {
        button::standard(label)
            .on_press(Message::Edit(EditMessage::SetColor(c)))
            .class(theme::Button::Suggested)
    } else {
        button::standard(label).on_press(Message::Edit(EditMessage::SetColor(c)))
    };

    container(btn).into()
}

pub fn side_dock(active_tool: AnnotateTool) -> Element<'static, Message> {
    let spacing = theme::active().cosmic().spacing;

    let tools = [
        AnnotateTool::Pen,
        AnnotateTool::Highlighter,
        AnnotateTool::Rectangle,
        AnnotateTool::Ellipse,
        AnnotateTool::Line,
        AnnotateTool::Arrow,
        AnnotateTool::Star,
        AnnotateTool::Polygon,
        AnnotateTool::Text,
    ];

    let mut col = column().spacing(spacing.space_xxs).align_x(Alignment::Center);
    for t in tools {
        col = col.push(tool_btn(t, active_tool));
    }

    // Separator
    col = col.push(cosmic::widget::divider::horizontal::default());

    // Rotate/flip
    col = col.push(
        tooltip::tooltip(
            button::icon(icon::from_name("object-rotate-right-symbolic").size(20))
                .on_press(Message::Edit(EditMessage::Rotate90)),
            text::body("Rotate 90"),
            tooltip::Position::Right,
        ),
    );
    col = col.push(
        tooltip::tooltip(
            button::icon(icon::from_name("object-flip-horizontal-symbolic").size(20))
                .on_press(Message::Edit(EditMessage::FlipHorizontal)),
            text::body("Flip H"),
            tooltip::Position::Right,
        ),
    );
    col = col.push(
        tooltip::tooltip(
            button::icon(icon::from_name("object-flip-vertical-symbolic").size(20))
                .on_press(Message::Edit(EditMessage::FlipVertical)),
            text::body("Flip V"),
            tooltip::Position::Right,
        ),
    );

    container(col)
        .padding(spacing.space_xxs)
        .class(theme::Container::Dialog)
        .into()
}

#[allow(clippy::too_many_arguments)]
pub fn properties_bar(
    active_tool: AnnotateTool,
    color: AnnotateColor,
    stroke_width: f32,
    bold: bool,
    italic: bool,
    underline: bool,
    font_size: f32,
    alignment: Horizontal,
) -> Element<'static, Message> {
    let spacing = theme::active().cosmic().spacing;

    let mut bar = row().spacing(spacing.space_s).align_y(Alignment::Center);

    // Color presets
    let mut colors = row().spacing(spacing.space_xxs);
    for c in AnnotateColor::presets() {
        colors = colors.push(color_circle(c, color));
    }
    bar = bar.push(colors);

    bar = bar.push(cosmic::widget::divider::vertical::default().height(28));

    // Stroke width
    let mut widths = row().spacing(spacing.space_xxs);
    for &w in STROKE_WIDTHS {
        let label = format!("{w:.0}");
        let btn = if (w - stroke_width).abs() < 0.5 {
            button::standard(label)
                .on_press(Message::Edit(EditMessage::SetStrokeWidth(w)))
                .class(theme::Button::Suggested)
        } else {
            button::standard(label).on_press(Message::Edit(EditMessage::SetStrokeWidth(w)))
        };
        widths = widths.push(btn);
    }
    bar = bar.push(widths);

    // Text-specific controls
    if active_tool == AnnotateTool::Text {
        bar = bar.push(cosmic::widget::divider::vertical::default().height(28));

        let bold_btn = if bold {
            button::standard("B")
                .on_press(Message::Edit(EditMessage::SetBold(!bold)))
                .class(theme::Button::Suggested)
        } else {
            button::standard("B").on_press(Message::Edit(EditMessage::SetBold(!bold)))
        };

        let italic_btn = if italic {
            button::standard("I")
                .on_press(Message::Edit(EditMessage::SetItalic(!italic)))
                .class(theme::Button::Suggested)
        } else {
            button::standard("I").on_press(Message::Edit(EditMessage::SetItalic(!italic)))
        };

        let underline_btn = if underline {
            button::standard("U")
                .on_press(Message::Edit(EditMessage::SetUnderline(!underline)))
                .class(theme::Button::Suggested)
        } else {
            button::standard("U").on_press(Message::Edit(EditMessage::SetUnderline(!underline)))
        };

        bar = bar.push(bold_btn);
        bar = bar.push(italic_btn);
        bar = bar.push(underline_btn);

        // Font size
        let mut sizes = row().spacing(spacing.space_xxs);
        for &s in FONT_SIZES {
            let label = format!("{s:.0}");
            let btn = if (s - font_size).abs() < 0.5 {
                button::standard(label)
                    .on_press(Message::Edit(EditMessage::SetFontSize(s)))
                    .class(theme::Button::Suggested)
            } else {
                button::standard(label).on_press(Message::Edit(EditMessage::SetFontSize(s)))
            };
            sizes = sizes.push(btn);
        }
        bar = bar.push(sizes);

        // Alignment
        let align_l = if alignment == Horizontal::Left {
            button::icon(icon::from_name("format-justify-left-symbolic").size(16))
                .on_press(Message::Edit(EditMessage::SetAlignment(Horizontal::Left)))
                .class(theme::Button::Suggested)
        } else {
            button::icon(icon::from_name("format-justify-left-symbolic").size(16))
                .on_press(Message::Edit(EditMessage::SetAlignment(Horizontal::Left)))
        };
        let align_c = if alignment == Horizontal::Center {
            button::icon(icon::from_name("format-justify-center-symbolic").size(16))
                .on_press(Message::Edit(EditMessage::SetAlignment(Horizontal::Center)))
                .class(theme::Button::Suggested)
        } else {
            button::icon(icon::from_name("format-justify-center-symbolic").size(16))
                .on_press(Message::Edit(EditMessage::SetAlignment(Horizontal::Center)))
        };
        let align_r = if alignment == Horizontal::Right {
            button::icon(icon::from_name("format-justify-right-symbolic").size(16))
                .on_press(Message::Edit(EditMessage::SetAlignment(Horizontal::Right)))
                .class(theme::Button::Suggested)
        } else {
            button::icon(icon::from_name("format-justify-right-symbolic").size(16))
                .on_press(Message::Edit(EditMessage::SetAlignment(Horizontal::Right)))
        };

        bar = bar.push(align_l);
        bar = bar.push(align_c);
        bar = bar.push(align_r);
    }

    bar = bar.push(horizontal_space());

    // Done / Undo / Redo
    bar = bar.push(
        button::icon(icon::from_name("edit-undo-symbolic").size(16))
            .on_press(Message::Edit(EditMessage::Undo)),
    );
    bar = bar.push(
        button::icon(icon::from_name("edit-redo-symbolic").size(16))
            .on_press(Message::Edit(EditMessage::Redo)),
    );

    container(bar)
        .padding(spacing.space_xxs)
        .width(Length::Fill)
        .class(theme::Container::Dialog)
        .into()
}
