use crate::message::{EditMessage, Message, ViewMessage};

use cosmic::{
    Element,
    iced::{Alignment, Length, alignment::Horizontal},
    theme,
    widget::{
        button, column, container, horizontal_space, icon, popover, row, slider, text, tooltip,
        vertical_space,
    },
};
use viewer_tools::annotate::{AnnotateColor, AnnotateTool, CropRatio, PenMode, TransformSubTool};

const STRIP_WIDTH: f32 = 44.0;
const BTN_SIZE: f32 = 36.0;
const ICON_SIZE: u16 = 20;
const BAR_HEIGHT: f32 = 40.0;

const STROKE_THIN: f32 = 2.0;
const STROKE_MED: f32 = 4.0;
const STROKE_THICK: f32 = 8.0;

#[derive(Clone)]
pub struct AnnotationProps {
    pub tool: AnnotateTool,
    pub color: AnnotateColor,
    pub stroke_width: f32,
    pub pen_mode: PenMode,
    pub opacity: f32,
    pub fill_mode: bool,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub font_size: f32,
    pub alignment: Horizontal,
    pub crop_ratio: CropRatio,
    pub can_undo: bool,
    pub can_redo: bool,
    pub is_drawing: bool,
    pub active_shape: AnnotateTool,
    pub active_transform: TransformSubTool,
    pub shape_popout_open: bool,
    pub transform_popout_open: bool,
    pub color_picker_open: bool,
    pub picker_hue: f32,
    pub picker_sat: f32,
    pub picker_bright: f32,
    pub picker_alpha: f32,
    pub picker_hex: String,
    pub recent_colors: Vec<cosmic::iced::Color>,
}

// --- Helpers ---

fn tool_btn(tool: AnnotateTool, active: AnnotateTool) -> Element<'static, Message> {
    let ic = icon::from_name(tool.icon_name()).size(ICON_SIZE);
    let mut btn = button::icon(ic)
        .width(Length::Fixed(BTN_SIZE))
        .height(Length::Fixed(BTN_SIZE))
        .on_press(Message::Edit(EditMessage::SetTool(tool)));
    if tool == active {
        btn = btn.class(theme::Button::Suggested);
    }
    btn.into()
}

/// Quick click sets the tool. Hold (300ms) opens the popout menu.
fn popout_btn(
    tool: AnnotateTool,
    active: AnnotateTool,
    _popout_msg: Message,
) -> Element<'static, Message> {
    let ic = icon::from_name(tool.icon_name()).size(ICON_SIZE);
    let is_active = tool == active;

    let indicator = text::body("\u{25B8}").size(8);
    let visual = cosmic::iced_widget::stack![
        container(ic)
            .width(Length::Fixed(BTN_SIZE))
            .height(Length::Fixed(BTN_SIZE))
            .center(Length::Fixed(BTN_SIZE))
            .class(if is_active {
                theme::Container::custom(|t| cosmic::widget::container::Style {
                    icon_color: Some(cosmic::iced::Color::WHITE),
                    text_color: Some(cosmic::iced::Color::WHITE),
                    background: Some(cosmic::iced::Background::Color(
                        t.cosmic().accent_color().into(),
                    )),
                    border: cosmic::iced::Border {
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
            } else {
                theme::Container::Transparent
            }),
        container(indicator)
            .width(Length::Fixed(BTN_SIZE))
            .height(Length::Fixed(BTN_SIZE))
            .align_x(Alignment::End)
            .align_y(Alignment::End)
            .padding([0, 2, 2, 0]),
    ];

    cosmic::widget::mouse_area(visual)
        .on_press(Message::Edit(EditMessage::PopoutPress(tool)))
        .on_release(Message::Edit(EditMessage::PopoutRelease(tool)))
        .into()
}

fn toggle_btn(label: String, active: bool, msg: Message) -> Element<'static, Message> {
    let mut btn = button::standard(label).on_press(msg);
    if active {
        btn = btn.class(theme::Button::Suggested);
    }
    btn.into()
}

fn stroke_btn(width: f32, active_width: f32) -> Element<'static, Message> {
    let label = match width as u32 {
        2 => "\u{2500}",
        4 => "\u{2501}",
        _ => "\u{2588}",
    };
    let active = (width - active_width).abs() < 0.5;
    let mut btn =
        button::standard(label).on_press(Message::Edit(EditMessage::SetStrokeWidth(width)));
    if active {
        btn = btn.class(theme::Button::Suggested);
    }
    btn.into()
}

fn stroke_group(active_width: f32) -> Element<'static, Message> {
    row()
        .spacing(2)
        .push(stroke_btn(STROKE_THIN, active_width))
        .push(stroke_btn(STROKE_MED, active_width))
        .push(stroke_btn(STROKE_THICK, active_width))
        .into()
}

fn color_btn(color: AnnotateColor) -> Element<'static, Message> {
    let [r, g, b] = [
        (color.0.r * 255.0) as u8,
        (color.0.g * 255.0) as u8,
        (color.0.b * 255.0) as u8,
    ];
    let hex = format!("#{r:02x}{g:02x}{b:02x}");

    tooltip::tooltip(
        button::icon(icon::from_name("color-select-symbolic").size(16))
            .on_press(Message::Edit(EditMessage::OpenColorPicker))
            .class(theme::Button::Standard),
        text::body(hex),
        tooltip::Position::Bottom,
    )
    .into()
}

fn color_plus(color: AnnotateColor) -> Element<'static, Message> {
    row()
        .spacing(4)
        .push(color_btn(color))
        .push(
            button::icon(icon::from_name("list-add-symbolic").size(14))
                .on_press(Message::Edit(EditMessage::OpenColorPicker)),
        )
        .into()
}

fn divider() -> Element<'static, Message> {
    cosmic::widget::divider::horizontal::default().into()
}

fn vdivider() -> Element<'static, Message> {
    cosmic::widget::divider::vertical::default()
        .height(28)
        .into()
}

fn right_actions(can_undo: bool, can_redo: bool) -> Element<'static, Message> {
    let undo = {
        let mut btn = button::icon(icon::from_name("edit-undo-symbolic").size(16));
        if can_undo {
            btn = btn.on_press(Message::Edit(EditMessage::Undo));
        }
        tooltip::tooltip(btn, text::body("Undo"), tooltip::Position::Bottom)
    };
    let redo = {
        let mut btn = button::icon(icon::from_name("edit-redo-symbolic").size(16));
        if can_redo {
            btn = btn.on_press(Message::Edit(EditMessage::Redo));
        }
        tooltip::tooltip(btn, text::body("Redo"), tooltip::Position::Bottom)
    };
    let save = tooltip::tooltip(
        button::icon(icon::from_name("document-save-symbolic").size(16))
            .on_press(Message::Edit(EditMessage::Save)),
        text::body("Save"),
        tooltip::Position::Bottom,
    );
    let cancel = tooltip::tooltip(
        button::icon(icon::from_name("window-close-symbolic").size(16))
            .on_press(Message::Edit(EditMessage::CancelAnnotation)),
        text::body("Cancel"),
        tooltip::Position::Bottom,
    );

    row()
        .spacing(4)
        .align_y(Alignment::Center)
        .push(undo)
        .push(redo)
        .push(vdivider())
        .push(save)
        .push(cancel)
        .into()
}

fn label_indicator(sym: &str, name: &str) -> Element<'static, Message> {
    text::body(format!("{sym} {name}")).into()
}

fn image_actions() -> Element<'static, Message> {
    row()
        .spacing(4)
        .push(
            tooltip::tooltip(
                button::icon(icon::from_name("object-flip-horizontal-symbolic").size(16))
                    .on_press(Message::Edit(EditMessage::FlipHorizontal)),
                text::body("Flip Horizontal"),
                tooltip::Position::Bottom,
            ),
        )
        .push(
            tooltip::tooltip(
                button::icon(icon::from_name("object-flip-vertical-symbolic").size(16))
                    .on_press(Message::Edit(EditMessage::FlipVertical)),
                text::body("Flip Vertical"),
                tooltip::Position::Bottom,
            ),
        )
        .push(
            tooltip::tooltip(
                button::icon(icon::from_name("object-rotate-right-symbolic").size(16))
                    .on_press(Message::Edit(EditMessage::Rotate90)),
                text::body("Rotate 90\u{00B0}"),
                tooltip::Position::Bottom,
            ),
        )
        .into()
}

// --- Pop-out menus ---

fn popout_item(
    ic_name: &'static str,
    shortcut: &'static str,
    active: bool,
    msg: Message,
) -> Element<'static, Message> {
    let content = row()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(icon::from_name(ic_name).size(ICON_SIZE))
        .push(
            text::body(shortcut)
                .size(11)
                .width(Length::Fixed(14.0)),
        );

    let mut btn = button::custom(content)
        .padding([6, 10])
        .on_press(msg);
    if active {
        btn = btn.class(theme::Button::Suggested);
    }
    btn.into()
}

fn shape_popout_menu(active: AnnotateTool) -> Element<'static, Message> {
    let mut col = column().spacing(2).padding(4);
    for &shape in AnnotateTool::shape_tools() {
        col = col.push(popout_item(
            shape.icon_name(),
            shape.shortcut_key(),
            shape == active,
            Message::Edit(EditMessage::SetTool(shape)),
        ));
    }
    container(col).class(theme::Container::Dialog).into()
}

fn transform_popout_menu(active: TransformSubTool) -> Element<'static, Message> {
    let items = [
        (TransformSubTool::Resize, "image-resize-symbolic", "1"),
        (TransformSubTool::Skew, "object-skew-symbolic", "2"),
        (TransformSubTool::Rotate, "object-rotate-right-symbolic", "3"),
    ];

    let mut col = column().spacing(2).padding(4);
    for (sub, ic_name, key) in items {
        col = col.push(popout_item(
            ic_name,
            key,
            sub == active,
            Message::Edit(EditMessage::SetTool(AnnotateTool::Transform)),
        ));
    }
    container(col).class(theme::Container::Dialog).into()
}

// --- Public functions ---

pub fn tool_strip(props: &AnnotationProps) -> Element<'static, Message> {
    let active = props.tool;

    let mut strip = column()
        .spacing(2)
        .align_x(Alignment::Center)
        .width(Length::Fill);

    // Pointer group
    let transform_btn = popout_btn(
        AnnotateTool::Transform,
        active,
        Message::View(ViewMessage::ToggleTransformPopout),
    );
    let transform_popover = if props.transform_popout_open {
        popover::popover(transform_btn)
            .popup(transform_popout_menu(props.active_transform))
            .position(popover::Position::Point(cosmic::iced::Point::new(
                STRIP_WIDTH, 0.0,
            )))
            .on_close(Message::View(ViewMessage::ClosePopouts))
    } else {
        popover::popover(transform_btn)
    };

    strip = strip
        .push(tool_btn(AnnotateTool::Select, active))
        .push(tool_btn(AnnotateTool::Move, active))
        .push(transform_popover);

    strip = strip.push(divider());

    // Drawing group
    strip = strip
        .push(tool_btn(AnnotateTool::Pen, active))
        .push(tool_btn(AnnotateTool::Highlighter, active));

    strip = strip.push(divider());

    // Shape group (single popout button with popover)
    let shape_btn = popout_btn(
        props.active_shape,
        active,
        Message::View(ViewMessage::ToggleShapePopout),
    );
    let shape_popover = if props.shape_popout_open {
        popover::popover(shape_btn)
            .popup(shape_popout_menu(props.active_shape))
            .position(popover::Position::Point(cosmic::iced::Point::new(
                STRIP_WIDTH, 0.0,
            )))
            .on_close(Message::View(ViewMessage::ClosePopouts))
    } else {
        popover::popover(shape_btn)
    };

    strip = strip.push(shape_popover);

    strip = strip.push(divider());

    // Special group
    strip = strip
        .push(tool_btn(AnnotateTool::Text, active))
        .push(tool_btn(AnnotateTool::Crop, active));

    strip = strip.push(vertical_space());

    container(strip)
        .padding(4)
        .class(theme::Container::Dialog)
        .height(Length::Fill)
        .width(Length::Fixed(STRIP_WIDTH))
        .into()
}


pub fn properties_bar(props: &AnnotationProps) -> Element<'static, Message> {
    let spacing = theme::active().cosmic().spacing;

    let mut bar = row()
        .spacing(spacing.space_s)
        .align_y(Alignment::Center)
        .height(Length::Fixed(BAR_HEIGHT));

    match props.tool {
        AnnotateTool::Select => {}

        AnnotateTool::Move => {
            bar = bar
                .push(label_indicator("\u{271D}", "Move"))
                .push(text::body("Drag to reposition").size(12));
        }

        AnnotateTool::Transform => {
            let sub_label = match props.active_transform {
                TransformSubTool::Resize => "\u{2921} Resize",
                TransformSubTool::Skew => "\u{2922} Skew",
                TransformSubTool::Rotate => "\u{21BB} Rotate",
            };
            bar = bar
                .push(text::body(sub_label))
                .push(vdivider())
                .push(text::body("Angle:"))
                .push(text::body("0\u{00B0}").size(12));
        }

        AnnotateTool::Pen => {
            let freeform_active = props.pen_mode == PenMode::Freeform;
            bar = bar
                .push(label_indicator("\u{270F}", "Pen"))
                .push(vdivider())
                .push(toggle_btn(
                    "Freeform".into(),
                    freeform_active,
                    Message::Edit(EditMessage::SetPenMode(PenMode::Freeform)),
                ))
                .push(toggle_btn(
                    "B\u{00E9}zier".into(),
                    !freeform_active,
                    Message::Edit(EditMessage::SetPenMode(PenMode::Bezier)),
                ))
                .push(vdivider())
                .push(color_plus(props.color))
                .push(vdivider())
                .push(stroke_group(props.stroke_width));
        }

        AnnotateTool::Highlighter => {
            bar = bar
                .push(label_indicator("\u{1F58D}", "Highlighter"))
                .push(vdivider())
                .push(color_plus(props.color))
                .push(vdivider())
                .push(stroke_group(props.stroke_width))
                .push(vdivider())
                .push(text::body("Opacity"))
                .push(
                    slider(0.1..=1.0, props.opacity, |v| {
                        Message::Edit(EditMessage::SetOpacity(v))
                    })
                    .width(Length::Fixed(80.0)),
                );
        }

        AnnotateTool::Rectangle
        | AnnotateTool::Ellipse
        | AnnotateTool::Line
        | AnnotateTool::Arrow
        | AnnotateTool::Star
        | AnnotateTool::Polygon => {
            let name = props.tool.display_name();
            bar = bar
                .push(label_indicator("\u{25A1}", name))
                .push(vdivider())
                .push(toggle_btn(
                    "Stroke".into(),
                    !props.fill_mode,
                    Message::Edit(EditMessage::SetFillMode(false)),
                ))
                .push(toggle_btn(
                    "Fill".into(),
                    props.fill_mode,
                    Message::Edit(EditMessage::SetFillMode(true)),
                ))
                .push(vdivider())
                .push(color_plus(props.color))
                .push(vdivider())
                .push(stroke_group(props.stroke_width));
        }

        AnnotateTool::Text => {
            bar = bar
                .push(label_indicator("T", "Text"))
                .push(vdivider())
                .push(text::body(format!("{:.0}px", props.font_size)))
                .push(vdivider())
                .push(toggle_btn(
                    "B".into(),
                    props.bold,
                    Message::Edit(EditMessage::SetBold(!props.bold)),
                ))
                .push(toggle_btn(
                    "I".into(),
                    props.italic,
                    Message::Edit(EditMessage::SetItalic(!props.italic)),
                ))
                .push(toggle_btn(
                    "U".into(),
                    props.underline,
                    Message::Edit(EditMessage::SetUnderline(!props.underline)),
                ))
                .push(toggle_btn(
                    "S\u{0336}".into(),
                    props.strikethrough,
                    Message::Edit(EditMessage::SetStrikethrough(!props.strikethrough)),
                ))
                .push(vdivider())
                .push(align_btn(Horizontal::Left, props.alignment))
                .push(align_btn(Horizontal::Center, props.alignment))
                .push(align_btn(Horizontal::Right, props.alignment))
                .push(vdivider())
                .push(color_plus(props.color));
        }

        AnnotateTool::Crop => {
            bar = bar
                .push(label_indicator("\u{2B12}", "Crop"))
                .push(vdivider())
                .push(ratio_btn("Free", CropRatio::Free, props.crop_ratio))
                .push(ratio_btn("1:1", CropRatio::Square, props.crop_ratio))
                .push(ratio_btn("4:3", CropRatio::FourThree, props.crop_ratio))
                .push(ratio_btn("16:9", CropRatio::SixteenNine, props.crop_ratio))
                .push(ratio_btn("Custom", CropRatio::Custom, props.crop_ratio))
                .push(vdivider())
                .push(
                    button::standard("Apply")
                        .on_press(Message::Edit(EditMessage::ApplyCrop))
                        .class(theme::Button::Suggested),
                )
                .push(
                    button::standard("Cancel")
                        .on_press(Message::Edit(EditMessage::CancelTool)),
                );
        }
    }

    if !props.is_drawing {
        bar = bar.push(vdivider()).push(image_actions());
    }

    bar = bar.push(horizontal_space());
    bar = bar.push(right_actions(props.can_undo, props.can_redo));

    container(bar)
        .padding([0, spacing.space_s])
        .width(Length::Fill)
        .height(Length::Fixed(BAR_HEIGHT))
        .class(theme::Container::Dialog)
        .into()
}

fn align_btn(align: Horizontal, active: Horizontal) -> Element<'static, Message> {
    let ic_name = match align {
        Horizontal::Left => "format-justify-left-symbolic",
        Horizontal::Center => "format-justify-center-symbolic",
        Horizontal::Right => "format-justify-right-symbolic",
    };
    let mut btn = button::icon(icon::from_name(ic_name).size(16))
        .on_press(Message::Edit(EditMessage::SetAlignment(align)));
    if align == active {
        btn = btn.class(theme::Button::Suggested);
    }
    btn.into()
}

fn ratio_btn(label: &'static str, ratio: CropRatio, active: CropRatio) -> Element<'static, Message> {
    let mut btn =
        button::standard(label).on_press(Message::Edit(EditMessage::SetCropRatio(ratio)));
    if ratio == active {
        btn = btn.class(theme::Button::Suggested);
    }
    btn.into()
}
