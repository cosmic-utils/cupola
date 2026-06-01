use crate::message::{EditMessage, Message, PickerTarget};
use cosmic::{
    Element,
    iced::{Alignment, Background, Border, Color},
    theme,
    widget::{Space, button, container, icon, mouse_area, row, text},
};
use viewer_tools::annotate::AnnotateColor;

const SWATCH_SIZE: f32 = 22.0;

/// Filled circular swatch of `color`. Clicking it opens the picker for `target`.
fn swatch(color: Color, target: PickerTarget) -> Element<'static, Message> {
    mouse_area(
        container(Space::new(SWATCH_SIZE, SWATCH_SIZE)).class(theme::Container::custom(
            move |_| container::Style {
                background: Some(Background::Color(color)),
                border: Border {
                    radius: (SWATCH_SIZE / 2.0).into(),
                    width: 1.0,
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.3),
                },
                ..Default::default()
            },
        )),
    )
    .on_press(Message::Edit(EditMessage::OpenColorPicker(target)))
    .into()
}

/// Current-color swatch + a (+) button for `target`. No hex text.
pub(super) fn color_plus(color: AnnotateColor, target: PickerTarget) -> Element<'static, Message> {
    row()
        .spacing(4)
        .align_y(Alignment::Center)
        .push(swatch(color.0, target))
        .push(
            button::icon(icon::from_name("list-add-symbolic").size(14))
                .on_press(Message::Edit(EditMessage::OpenColorPicker(target)))
                .class(theme::Button::Standard)
                .description("Color Picker"),
        )
        .into()
}

/// A labeled swatch ("Stroke"/"Fill") that opens the picker for `target`.
pub(super) fn labeled_swatch(
    label: &'static str,
    color: Color,
    target: PickerTarget,
) -> Element<'static, Message> {
    row()
        .spacing(4)
        .align_y(Alignment::Center)
        .push(text::body(label).size(12))
        .push(swatch(color, target))
        .into()
}
