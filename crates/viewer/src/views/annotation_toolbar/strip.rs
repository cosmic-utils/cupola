use super::{
    AnnotationProps,
    common::{BTN_SIZE, ICON_SIZE, STRIP_WIDTH, divider, popout_btn, tool_btn},
    popouts::{shape_popout_menu, transform_popout_menu},
};
use crate::message::{EditMessage, Message, ViewMessage};
use cosmic::{
    Element,
    iced::{Alignment, Length, Point},
    widget::{button, column, container, icon, popover, vertical_space},
};
use viewer_tools::annotate::AnnotateTool;

/// A fixed-size icon button for image actions on the strip (no active/tool state).
fn strip_btn(
    icn_name: &'static str,
    desc: &'static str,
    msg: Message,
) -> Element<'static, Message> {
    button::icon(icon::from_name(icn_name).size(ICON_SIZE))
        .width(Length::Fixed(BTN_SIZE))
        .height(Length::Fixed(BTN_SIZE))
        .on_press(msg)
        .description(desc)
        .into()
}

pub fn tool_strip(props: &AnnotationProps) -> Element<'static, Message> {
    let active = props.tool;
    let mut strip = column()
        .spacing(2)
        .align_x(Alignment::Center)
        .width(Length::Fill);
    let transform_btn = popout_btn(
        AnnotateTool::Transform,
        props.active_transform.icon_name(),
        active,
        Message::View(ViewMessage::ToggleTransformPopout),
    );
    let transform_popout = if props.transform_popout_open {
        popover(transform_btn)
            .popup(transform_popout_menu(props.active_transform))
            .position(popover::Position::Point(Point::new(STRIP_WIDTH, 0.0)))
            .on_close(Message::View(ViewMessage::ClosePopouts))
    } else {
        popover(transform_btn)
    };

    strip = strip
        .push(tool_btn(AnnotateTool::Select, active))
        .push(tool_btn(AnnotateTool::Move, active))
        .push(transform_popout)
        .push(divider())
        .push(tool_btn(AnnotateTool::Pen, active))
        .push(tool_btn(AnnotateTool::Highlighter, active))
        .push(divider());

    let shape_btn = popout_btn(
        props.active_shape,
        props.active_shape.icon_name(),
        active,
        Message::View(ViewMessage::ToggleShapePopout),
    );
    let shape_popover = if props.shape_popout_open {
        popover(shape_btn)
            .popup(shape_popout_menu(props.active_shape))
            .position(popover::Position::Point(Point::new(STRIP_WIDTH, 0.0)))
            .on_close(Message::View(ViewMessage::ClosePopouts))
    } else {
        popover(shape_btn)
    };

    strip = strip
        .push(shape_popover)
        .push(divider())
        .push(tool_btn(AnnotateTool::Text, active))
        .push(tool_btn(AnnotateTool::Crop, active))
        .push(vertical_space())
        .push(divider())
        .push(strip_btn(
            "object-flip-horizontal-symbolic",
            "Flip Horizontal",
            Message::Edit(EditMessage::FlipHorizontal),
        ))
        .push(strip_btn(
            "object-flip-vertical-symbolic",
            "Flip Vertical",
            Message::Edit(EditMessage::FlipVertical),
        ));

    container(strip)
        .padding(4)
        .height(Length::Fill)
        .width(Length::Fixed(STRIP_WIDTH))
        .into()
}
