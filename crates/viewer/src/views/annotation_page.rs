use crate::{
    message::{EditMessage, Message},
    views::{ImageViewState, annotation_toolbar::{self, AnnotationProps}},
};

use cosmic::{
    Element,
    iced::Length,
    widget::{column, container, image::Handle, popover, row},
};
use viewer_image::CachedImage;
use viewer_widgets::{annotation_widget, color_picker};

pub fn annotation_page<'a>(
    cached: &CachedImage,
    image_state: &ImageViewState,
    props: AnnotationProps,
    committed: Option<&Handle>,
    preview: Option<&Handle>,
) -> Element<'a, Message> {
    let handle = cached.handle.clone();
    let img_w = cached.width;
    let img_h = cached.height;

    let zoom = if image_state.fit_to_window {
        image_state.fit_zoom
    } else {
        image_state.zoom_level
    };

    let tool_strip = annotation_toolbar::tool_strip(&props);
    let props_bar = annotation_toolbar::properties_bar(&props);

    let mut widget = annotation_widget(handle, img_w, img_h)
        .zoom(zoom)
        .on_tool_start(|pt| Message::Edit(EditMessage::ToolStart(pt)))
        .on_tool_drag(|pt| Message::Edit(EditMessage::ToolDrag(pt)))
        .on_tool_end(|| Message::Edit(EditMessage::ToolEnd));

    if let Some(committed) = committed {
        widget = widget.committed_overlay(committed.clone());
    }
    if let Some(preview) = preview {
        widget = widget.preview_overlay(preview.clone());
    }

    let canvas = container(cosmic::Element::from(widget))
        .width(Length::Fill)
        .height(Length::Fill);

    // Wrap properties bar in popover for color picker
    let props_bar_with_picker = if props.color_picker_open {
        let picker = color_picker::color_picker(
            props.picker_hue,
            props.picker_sat,
            props.picker_bright,
            props.picker_alpha,
            props.picker_hex.clone(),
            props.recent_colors.clone(),
            |h, s| Message::Edit(EditMessage::PickerHueSat(h, s)),
            |b| Message::Edit(EditMessage::PickerBrightness(b)),
            |a| Message::Edit(EditMessage::PickerAlpha(a)),
            |hex| Message::Edit(EditMessage::PickerHexInput(hex)),
            |color| Message::Edit(EditMessage::SetCustomColor(color)),
            |color| Message::Edit(EditMessage::SetCustomColor(color)),
        );

        popover::popover(props_bar)
            .popup(picker)
            .position(popover::Position::Bottom)
            .on_close(Message::Edit(EditMessage::CloseColorPicker))
            .into()
    } else {
        props_bar
    };

    let right_pane = column()
        .push(props_bar_with_picker)
        .push(canvas)
        .width(Length::Fill)
        .height(Length::Fill);

    row()
        .push(tool_strip)
        .push(right_pane)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
