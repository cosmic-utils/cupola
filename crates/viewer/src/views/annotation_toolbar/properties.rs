use super::{
    AnnotationProps,
    color::{color_plus, labeled_swatch},
    common::{BAR_HEIGHT, label_indicator, toggle_btn, vert_divider},
};
use crate::message::{EditMessage, Message, PickerTarget};
use cosmic::{
    Element,
    iced::{Alignment, Length, alignment::Horizontal},
    theme,
    widget::{button, container, dropdown, horizontal_space, icon, row, slider, text},
};
use viewer_tools::annotate::{AnnotateTool, CropRatio, PenMode, TransformSubTool};

const STROKE_THIN: f32 = 2.0;
const STROKE_MED: f32 = 4.0;
const STROKE_MAX: f32 = 8.0;

const FONT_FAMILIES: [&str; 3] = ["Sans", "Serif", "Monospace"];
const FONT_FAMILY_VALUES: [&str; 3] = ["sans-serif", "serif", "monospace"];
const FONT_SIZES: [&str; 9] = ["8", "10", "12", "14", "16", "18", "24", "32", "48"];
const FONT_SIZE_VALUES: [f32; 9] = [8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 24.0, 32.0, 48.0];

fn stroke_btn(width: f32, active_width: f32) -> Element<'static, Message> {
    let label = match width as u32 {
        2 => "\u{2500}",
        4 => "\u{2501}",
        _ => "\u{2588}",
    };
    let desc = format!("Stroke Width {width:.0}");
    let active = (width - active_width).abs() < 0.5;
    let mut btn = button::standard(label)
        .on_press(Message::Edit(EditMessage::SetStrokeWidth(width)))
        .description(desc);

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
        .push(stroke_btn(STROKE_MAX, active_width))
        .into()
}

fn right_actions(can_undo: bool, can_redo: bool) -> Element<'static, Message> {
    let undo = {
        let mut btn =
            button::icon(icon::from_name("edit-undo-symbolic").size(16)).description("Undo");

        if can_undo {
            btn = btn.on_press(Message::Edit(EditMessage::Undo));
        }

        btn
    };

    let redo = {
        let mut btn =
            button::icon(icon::from_name("edit-redo-symbolic").size(16)).description("Redo");

        if can_redo {
            btn = btn.on_press(Message::Edit(EditMessage::Redo));
        }

        btn
    };

    let save = button::icon(icon::from_name("document-save-symbolic").size(16))
        .on_press(Message::Edit(EditMessage::Save))
        .description("Save");

    let cancel = button::icon(icon::from_name("window-close-symbolic").size(16))
        .on_press(Message::Edit(EditMessage::CancelAnnotation))
        .description("Cancel");

    row()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(undo)
        .push(redo)
        .push(vert_divider())
        .push(save)
        .push(cancel)
        .into()
}

fn image_actions() -> Element<'static, Message> {
    row()
        .spacing(4)
        .align_y(Alignment::Center)
        .push(text::body("Image:").size(12))
        .push(
            button::icon(icon::from_name("object-rotate-right-symbolic").size(16))
                .on_press(Message::Edit(EditMessage::Rotate90))
                .description("Rotate Right"),
        )
        .push(
            button::icon(icon::from_name("object-rotate-left-symbolic").size(16))
                .on_press(Message::Edit(EditMessage::Rotate270))
                .description("Rotate Left"),
        )
        .push(
            button::icon(icon::from_name("object-flip-horizontal-symbolic").size(16))
                .on_press(Message::Edit(EditMessage::FlipHorizontal))
                .description("Flip Horizontal"),
        )
        .push(
            button::icon(icon::from_name("object-flip-vertical-symbolic").size(16))
                .on_press(Message::Edit(EditMessage::FlipVertical))
                .description("Flip Vertical"),
        )
        .into()
}

fn align_btn(align: Horizontal, active: Horizontal) -> Element<'static, Message> {
    let icon_name = match align {
        Horizontal::Left => "format-justify-left-symbolic",
        Horizontal::Center => "format-justify-center-symbolic",
        Horizontal::Right => "format-justify-right-symbolic",
    };

    let mut btn = button::icon(icon::from_name(icon_name).size(16))
        .on_press(Message::Edit(EditMessage::SetAlignment(align)));

    if align == active {
        btn = btn.class(theme::Button::Suggested);
    }

    btn.into()
}

fn ratio_btn(
    label: &'static str,
    ratio: CropRatio,
    active: CropRatio,
) -> Element<'static, Message> {
    let mut btn = button::standard(label).on_press(Message::Edit(EditMessage::SetCropRatio(ratio)));

    if ratio == active {
        btn = btn.class(theme::Button::Suggested);
    }

    btn.into()
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
                .push(vert_divider())
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
                .push(vert_divider())
                .push(text::body("Angle:"))
                .push(text::body("0\u{00B0}").size(12));
        }
        AnnotateTool::Pen => {
            let freeform_active = props.pen_mode == PenMode::Freeform;
            bar = bar
                .push(label_indicator("\u{270F}", "Pen"))
                .push(vert_divider())
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
                .push(vert_divider())
                .push(color_plus(props.color, PickerTarget::Stroke))
                .push(vert_divider())
                .push(stroke_group(props.stroke_width));
        }
        AnnotateTool::Highlighter => {
            bar = bar
                .push(label_indicator("\u{1F58D}", "Highlighter"))
                .push(vert_divider())
                .push(color_plus(props.color, PickerTarget::Stroke))
                .push(vert_divider())
                .push(stroke_group(props.stroke_width))
                .push(vert_divider())
                .push(text::body("Opacity"))
                .push(
                    slider(0.1..=1.0, props.opacity, |opacity| {
                        Message::Edit(EditMessage::SetOpacity(opacity))
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
            bar = bar
                .push(label_indicator(
                    shape_glyph(props.tool),
                    props.tool.display_name(),
                ))
                .push(vert_divider())
                .push(labeled_swatch("Stroke", props.color.0, PickerTarget::Stroke))
                .push(toggle_btn(
                    "Fill".into(),
                    props.fill_mode,
                    Message::Edit(EditMessage::SetFillMode(!props.fill_mode)),
                ))
                .push(labeled_swatch("Fill", props.fill_color, PickerTarget::Fill))
                .push(vert_divider())
                .push(stroke_group(props.stroke_width));
        }
        AnnotateTool::Text => {
            let family_idx = FONT_FAMILY_VALUES.iter().position(|&f| f == props.font_family);
            let size_idx = FONT_SIZE_VALUES
                .iter()
                .position(|&s| (s - props.font_size).abs() < 0.5);
            bar = bar
                .push(label_indicator("T", "Text"))
                .push(vert_divider())
                .push(dropdown(
                    FONT_FAMILIES.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                    family_idx,
                    |idx| Message::Edit(EditMessage::SetFontFamily(FONT_FAMILY_VALUES[idx])),
                ))
                .push(dropdown(
                    FONT_SIZES.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                    size_idx,
                    |idx| Message::Edit(EditMessage::SetFontSize(FONT_SIZE_VALUES[idx])),
                ))
                .push(vert_divider())
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
                .push(vert_divider())
                .push(align_btn(Horizontal::Left, props.alignment))
                .push(align_btn(Horizontal::Center, props.alignment))
                .push(align_btn(Horizontal::Right, props.alignment))
                .push(vert_divider())
                .push(color_plus(props.color, PickerTarget::Stroke));
        }
        AnnotateTool::Crop => {
            bar = bar
                .push(label_indicator("\u{2B12}", "Crop"))
                .push(vert_divider())
                .push(ratio_btn("Free", CropRatio::Free, props.crop_ratio))
                .push(ratio_btn("1:1", CropRatio::Square, props.crop_ratio))
                .push(ratio_btn("4:3", CropRatio::FourThree, props.crop_ratio))
                .push(ratio_btn("16:9", CropRatio::SixteenNine, props.crop_ratio))
                .push(vert_divider())
                .push(
                    button::standard("Apply")
                        .on_press(Message::Edit(EditMessage::ApplyCrop))
                        .class(theme::Button::Suggested),
                )
                .push(button::standard("Cancel").on_press(Message::Edit(EditMessage::CancelTool)));
        }
    }

    if !props.is_drawing {
        bar = bar.push(vert_divider()).push(image_actions());
    }

    bar = bar
        .push(horizontal_space())
        .push(right_actions(props.can_undo, props.can_redo));

    container(bar)
        .padding([0, spacing.space_s])
        .width(Length::Fill)
        .height(Length::Fixed(BAR_HEIGHT))
        .class(theme::Container::Dialog)
        .into()
}

fn shape_glyph(tool: AnnotateTool) -> &'static str {
    match tool {
        AnnotateTool::Ellipse => "\u{25CB}",
        AnnotateTool::Line => "\u{2571}",
        AnnotateTool::Arrow => "\u{2197}",
        AnnotateTool::Star => "\u{2605}",
        AnnotateTool::Polygon => "\u{25B3}",
        _ => "\u{25A1}",
    }
}
