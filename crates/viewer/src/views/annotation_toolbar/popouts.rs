use super::common::ICON_SIZE;
use crate::message::{EditMessage, Message};
use cosmic::{
    Element,
    iced::{Alignment, Length},
    theme,
    widget::{button, column, container, icon, row, text},
};
use viewer_tools::annotate::{AnnotateTool, TransformSubTool};

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
        .push(text::body(shortcut).size(11).width(Length::Fixed(14.0)));

    let mut btn = button::custom(content).padding([6, 10]).on_press(msg);

    if active {
        btn = btn.class(theme::Button::Suggested);
    }

    btn.into()
}

pub(super) fn shape_popout_menu(active: AnnotateTool) -> Element<'static, Message> {
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

pub(super) fn transform_popout_menu(active: TransformSubTool) -> Element<'static, Message> {
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
            Message::Edit(EditMessage::SetTransformSubTool(sub)),
        ));
    }

    container(col).class(theme::Container::Dialog).into()
}
