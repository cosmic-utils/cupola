use crate::message::{EditMessage, Message};
use cosmic::{
    Element,
    iced::{Alignment, Background, Border, Color, Length},
    iced_widget::stack,
    theme,
    widget::{button, container, divider, icon, mouse_area, text},
};
use viewer_tools::annotate::AnnotateTool;

pub(super) const STRIP_WIDTH: f32 = 44.0;
pub(super) const BTN_SIZE: f32 = 36.0;
pub(super) const ICON_SIZE: u16 = 20;
pub(super) const BAR_HEIGHT: f32 = 40.0;

pub(super) fn tool_btn(tool: AnnotateTool, active: AnnotateTool) -> Element<'static, Message> {
    let ico = icon::from_name(tool.icon_name()).size(ICON_SIZE);
    let desc = format!("{} ({})", tool.display_name(), tool.shortcut_key());
    let mut btn = button::icon(ico)
        .width(Length::Fixed(BTN_SIZE))
        .height(Length::Fixed(BTN_SIZE))
        .on_press(Message::Edit(EditMessage::SetTool(tool)))
        .description(desc);

    if tool == active {
        btn = btn.class(theme::Button::Suggested);
    }

    btn.into()
}

/// Quick click sets the tool. Hold (300ms) opens the popout menu.
pub(super) fn popout_btn(
    tool: AnnotateTool,
    icon_name: &'static str,
    active: AnnotateTool,
    _popout_msg: Message,
) -> Element<'static, Message> {
    let ico = icon::from_name(icon_name).size(ICON_SIZE);
    let is_active = tool == active;

    let indicator = text::body("\u{25B8}").size(8);
    let visual = stack![
        container(ico)
            .width(Length::Fixed(BTN_SIZE))
            .height(Length::Fixed(BTN_SIZE))
            .center(Length::Fixed(BTN_SIZE))
            .class(if is_active {
                theme::Container::custom(|theme| container::Style {
                    icon_color: Some(Color::WHITE),
                    text_color: Some(Color::WHITE),
                    background: Some(Background::Color(theme.cosmic().accent_color().into())),
                    border: Border {
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

    mouse_area(visual)
        .on_press(Message::Edit(EditMessage::PopoutPress(tool)))
        .on_release(Message::Edit(EditMessage::PopoutRelease(tool)))
        .into()
}

pub(super) fn toggle_btn(label: String, active: bool, msg: Message) -> Element<'static, Message> {
    let mut btn = button::standard(label).on_press(msg);
    if active {
        btn = btn.class(theme::Button::Suggested);
    }

    btn.into()
}

pub(super) fn divider() -> Element<'static, Message> {
    divider::horizontal::default().into()
}

pub(super) fn vert_divider() -> Element<'static, Message> {
    divider::vertical::default().into()
}

pub(super) fn label_indicator(sym: &str, name: &str) -> Element<'static, Message> {
    text::body(format!("{sym} {name}")).into()
}
