use crate::{
    message::{EditMessage, Message},
    views::{ImageViewState, annotation_toolbar::{self, AnnotationProps}},
};

use cosmic::{
    Element,
    iced::Length,
    widget::{column, container, image::Handle, row},
};
use viewer_image::CachedImage;
use viewer_widgets::annotation_widget;

pub fn annotation_page<'a>(
    cached: &CachedImage,
    image_state: &ImageViewState,
    props: &AnnotationProps,
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

    let tool_strip = annotation_toolbar::tool_strip(props);
    let props_bar = annotation_toolbar::properties_bar(props);

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

    let right_pane = column()
        .push(props_bar)
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
