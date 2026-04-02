use crate::{
    fl,
    message::{EditMessage, Message, NavMessage, ViewMessage},
    views::ImageViewState,
    views::annotation_toolbar::AnnotationProps,
    widgets::{GalleryItem, gallery_grid},
};
use cosmic::{
    Element,
    iced::{Alignment, ContentFit, Length},
    iced_widget::{
        scrollable::{Direction, Scrollbar},
        stack,
    },
    theme,
    widget::{
        Id, Space, button, column, container, horizontal_space, icon, image, mouse_area,
        responsive, row, scrollable, text,
    },
};
use viewer_image::{CachedImage, ImageCache};
use viewer_nav::NavState;
use viewer_tools::annotate::{AnnotateColor, AnnotateTool, CropRatio, PenMode, TransformSubTool};
use viewer_widgets::annotation_widget;

pub struct AnnotationOverlay {
    pub tool: AnnotateTool,
    pub color: AnnotateColor,
    pub stroke_width: f32,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub font_size: f32,
    pub alignment: cosmic::iced::alignment::Horizontal,
    pub committed: Option<cosmic::widget::image::Handle>,
    pub preview: Option<cosmic::widget::image::Handle>,
}

#[derive(Debug, Clone, Default)]
pub struct GalleryView {
    pub selected: Vec<usize>,
    pub cols: usize,
    pub row_height: f32,
    pub focused_index: Option<usize>,
    pub viewport: Option<cosmic::iced::widget::scrollable::Viewport>,
}

impl GalleryView {
    pub const SCROLL_ID: &'static str = "gallery-scroll";

    pub fn new() -> Self {
        Self {
            selected: Vec::new(),
            cols: 4,
            row_height: 4.0,
            focused_index: None,
            viewport: None,
        }
    }

    pub fn toggle_selection(&mut self, idx: usize) {
        if let Some(pos) = self.selected.iter().position(|&i| i == idx) {
            self.selected.remove(pos);
        } else {
            self.selected.push(idx);
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected.clear();
    }

    pub fn is_selected(&self, idx: usize) -> bool {
        self.selected.contains(&idx)
    }

    fn modal_loading(&self) -> Element<'static, Message> {
        let spacing = theme::active().cosmic().spacing;

        let close_btn = button::icon(icon::from_name("window-close-symbolic"))
            .on_press(Message::View(ViewMessage::CloseModal))
            .padding(spacing.space_xs)
            .class(theme::Button::Destructive);

        let header = row()
            .push(horizontal_space())
            .push(close_btn)
            .width(Length::Fill)
            .padding(spacing.space_xs);

        let loading = container(
            column()
                .push(icon::from_name("content-loading-symbolic").size(48))
                .push(text("Loading...").size(14))
                .spacing(spacing.space_s)
                .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill);

        container(
            mouse_area(
                container(
                    column()
                        .push(header)
                        .push(loading)
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .class(theme::Container::Dialog),
            )
            .on_press(Message::View(ViewMessage::ImageEditEvent)),
        )
        .padding([60, 80])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn modal_content(
        &self,
        cached: &CachedImage,
        image_state: &ImageViewState,
    ) -> Element<'static, Message> {
        let spacing = theme::active().cosmic().spacing;

        let handle = cached.handle.clone();
        let img_width = cached.width as f32;
        let img_height = cached.height as f32;
        let fit_to_window = image_state.fit_to_window;
        let zoom_level = image_state.zoom_level;
        let scroll_id = image_state.scroll_id.clone();

        let prev_btn = container(
            button::icon(icon::from_name("go-previous-symbolic"))
                .on_press(Message::Nav(NavMessage::Prev)),
        )
        .width(Length::Shrink)
        .height(Length::Fill)
        .center_y(Length::Fill);

        let next_btn = container(
            button::icon(icon::from_name("go-next-symbolic"))
                .on_press(Message::Nav(NavMessage::Next)),
        )
        .width(Length::Shrink)
        .height(Length::Fill)
        .center_y(Length::Fill);

        let annotate_btn = button::icon(icon::from_name("edit-symbolic"))
            .on_press(Message::Edit(EditMessage::SetTool(
                viewer_tools::annotate::AnnotateTool::Pen,
            )))
            .padding(spacing.space_xs)
            .class(theme::Button::Standard);

        let crop_btn = button::icon(icon::from_name("edit-cut-symbolic"))
            .on_press(Message::Edit(EditMessage::SetTool(
                viewer_tools::annotate::AnnotateTool::Crop,
            )))
            .padding(spacing.space_xs)
            .class(theme::Button::Standard);

        let close_btn = button::icon(icon::from_name("window-close-symbolic"))
            .on_press(Message::View(ViewMessage::CloseModal))
            .padding(spacing.space_xs)
            .class(theme::Button::Destructive);

        let header = row()
            .push(annotate_btn)
            .push(crop_btn)
            .push(horizontal_space())
            .push(close_btn)
            .spacing(spacing.space_xxs)
            .width(Length::Fill)
            .padding(spacing.space_xs);

        let image_area = responsive(move |size| {
            let available_width = size.width - (spacing.space_xs * 2) as f32;
            let available_height = size.height - (spacing.space_xs * 2) as f32;

            let fit_zoom_calc = {
                let zoom_x = available_width / img_width;
                let zoom_y = available_height / img_height;
                zoom_x.min(zoom_y).min(1.0)
            };

            let effective_zoom = if fit_to_window {
                fit_zoom_calc
            } else {
                zoom_level
            };

            let scaled_width = img_width * effective_zoom;
            let scaled_height = img_height * effective_zoom;

            let pad_x = ((available_width - scaled_width) / 2.0).max(0.0);
            let pad_y = ((available_height - scaled_height) / 2.0).max(0.0);

            let image_widget = image(handle.clone())
                .content_fit(ContentFit::Fill)
                .width(Length::Fixed(scaled_width))
                .height(Length::Fixed(scaled_height));

            if scaled_width > available_width || scaled_height > available_height {
                container(
                    scrollable(
                        container(image_widget)
                            .width(Length::Shrink)
                            .height(Length::Shrink)
                            .padding([pad_y, pad_x]),
                    )
                    .id(scroll_id.clone())
                    .direction(Direction::Both {
                        vertical: Scrollbar::default(),
                        horizontal: Scrollbar::default(),
                    })
                    .width(Length::Fill)
                    .height(Length::Fill),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            } else {
                // Just center it
                container(image_widget)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center(Length::Fill)
                    .into()
            }
        });

        // Zoom controls
        let fit_zoom_display = image_state.fit_zoom;
        let zoom_ctrls = row()
            .push(
                button::icon(icon::from_name("zoom-out-symbolic"))
                    .on_press(Message::View(ViewMessage::ZoomOut))
                    .padding(spacing.space_xs),
            )
            .push(if fit_to_window {
                container(text::body(format!(
                    "Fit ({}%)",
                    (fit_zoom_display * 100.0) as u32
                )))
                .padding(spacing.space_xs)
            } else {
                container(
                    button::text(format!("{}%", (zoom_level * 100.0) as u32))
                        .on_press(Message::View(ViewMessage::ZoomReset)),
                )
                .padding(spacing.space_xs)
            })
            .push(
                button::icon(icon::from_name("zoom-in-symbolic"))
                    .on_press(Message::View(ViewMessage::ZoomIn))
                    .padding(spacing.space_xs),
            )
            .push(
                button::icon(icon::from_name("zoom-fit-best-symbolic"))
                    .on_press(Message::View(ViewMessage::ZoomFit))
                    .padding(spacing.space_xs),
            )
            .spacing(spacing.space_xs)
            .align_y(Alignment::Center);

        let footer = row()
            .push(horizontal_space())
            .push(zoom_ctrls)
            .push(horizontal_space())
            .width(Length::Fill)
            .padding(spacing.space_xs);

        let content_row = row()
            .push(prev_btn)
            .push(image_area)
            .push(next_btn)
            .width(Length::Fill)
            .height(Length::Fill);

        // Outer padding lets gallery peek through
        container(
            mouse_area(
                container(
                    column()
                        .push(header)
                        .push(content_row)
                        .push(footer)
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .class(theme::Container::Dialog),
            )
            .on_press(Message::View(ViewMessage::ImageEditEvent)),
        )
        .padding([60, 80])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn modal_annotate(
        &self,
        cached: &CachedImage,
        image_state: &ImageViewState,
        ann: &AnnotationOverlay,
    ) -> Element<'static, Message> {
        let spacing = theme::active().cosmic().spacing;
        let handle = cached.handle.clone();
        let img_w = cached.width;
        let img_h = cached.height;
        let zoom = if image_state.fit_to_window {
            image_state.fit_zoom
        } else {
            image_state.zoom_level
        };

        // Close / done button
        let close_btn = button::icon(icon::from_name("window-close-symbolic"))
            .on_press(Message::View(ViewMessage::CloseModal))
            .padding(spacing.space_xs)
            .class(theme::Button::Destructive);

        let done_btn = button::suggested("Done").on_press(Message::Edit(EditMessage::CommitTool));
        let save_btn =
            button::suggested("Save").on_press(Message::Edit(EditMessage::Save));

        let header = row()
            .push(horizontal_space())
            .push(done_btn)
            .push(save_btn)
            .push(close_btn)
            .spacing(spacing.space_xs)
            .width(Length::Fill)
            .padding(spacing.space_xs);

        // Build annotation props from overlay
        let ann_props = AnnotationProps {
            tool: ann.tool,
            color: ann.color,
            stroke_width: ann.stroke_width,
            pen_mode: PenMode::default(),
            opacity: 1.0,
            fill_mode: false,
            bold: ann.bold,
            italic: ann.italic,
            underline: ann.underline,
            strikethrough: false,
            font_size: ann.font_size,
            alignment: ann.alignment,
            crop_ratio: CropRatio::default(),
            can_undo: true,
            can_redo: false,
            is_drawing: true,
            active_shape: AnnotateTool::Rectangle,
            active_transform: TransformSubTool::default(),
            shape_popout_open: false,
            transform_popout_open: false,
        };

        // Properties bar at top
        let props_bar = super::annotation_toolbar::properties_bar(&ann_props);

        // Side dock
        let dock = super::annotation_toolbar::tool_strip(&ann_props);

        // Annotation widget
        let mut widget = annotation_widget(handle, img_w, img_h)
            .zoom(zoom)
            .on_tool_start(|pt| Message::Edit(EditMessage::ToolStart(pt)))
            .on_tool_drag(|pt| Message::Edit(EditMessage::ToolDrag(pt)))
            .on_tool_end(|| Message::Edit(EditMessage::ToolEnd));

        if let Some(ref committed) = ann.committed {
            widget = widget.committed_overlay(committed.clone());
        }
        if let Some(ref preview) = ann.preview {
            widget = widget.preview_overlay(preview.clone());
        }

        let center_area = row()
            .push(dock)
            .push(cosmic::Element::from(widget))
            .width(Length::Fill)
            .height(Length::Fill);

        container(
            mouse_area(
                container(
                    column()
                        .push(header)
                        .push(props_bar)
                        .push(center_area)
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .class(theme::Container::Dialog),
            )
            .on_press(Message::View(ViewMessage::ImageEditEvent)),
        )
        .padding([20, 20])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub fn view<'a>(
        &'a self,
        nav: &NavState,
        cache: &ImageCache,
        thumbnail_size: u32,
        image_state: &ImageViewState,
        annotation: Option<AnnotationOverlay>,
    ) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;
        let images = nav.images();

        if images.is_empty() {
            return container(
                column()
                    .push(icon::from_name("folder-pictures-symbolic").size(64))
                    .push(text(fl!("status-no-image")).size(16))
                    .push(text("Open a folder to view images").size(12))
                    .spacing(spacing.space_m)
                    .align_x(Alignment::Center),
            )
            .center(Length::Fill)
            .into();
        }

        // Build gallery items with dimensions for proper aspect ratio
        let items: Vec<GalleryItem> = images
            .iter()
            .map(|path| {
                // Get thumbnail handle and dimensions from cache
                let (handle, width, height) = if let Some(cached) = cache.get_thumbnail(path) {
                    (Some(cached.handle), cached.width, cached.height)
                } else {
                    (None, thumbnail_size, thumbnail_size)
                };

                GalleryItem::new(path.clone(), handle, width, height)
            })
            .collect();

        // Disable keyboard nav when modal is open (modal handles arrow keys)
        let modal_open = nav.index().is_some();

        let content = gallery_grid(items)
            .thumbnail_size(thumbnail_size)
            .focused(self.focused_index)
            .selected(self.selected.clone())
            .spacing(spacing.space_xs)
            .padding(spacing.space_s)
            .scrollable(Id::new(Self::SCROLL_ID))
            .keyboard_navigation(!modal_open)
            .on_focus(|idx| Message::Nav(NavMessage::GalleryFocus(idx)))
            .on_activate(|idx| Message::Nav(NavMessage::GallerySelect(idx)))
            .on_scroll_request(|req| Message::View(ViewMessage::GalleryScrollTo(req.offset_y)))
            .into_element();

        // Status bar
        let status = row()
            .push(text(format!("{} images", images.len())).size(12))
            .padding([spacing.space_xxs, spacing.space_s])
            .align_y(Alignment::Center);

        let gallery: Element<'_, Message> = column()
            .push(content)
            .push(status)
            .width(Length::Fill)
            .into();

        if let Some(idx) = nav.index()
            && let Some(path) = images.get(idx)
        {
            // Use mouse-area to close the modal when the backdrop is clicked.
            let backdrop = mouse_area(
                container(Space::new(Length::Fill, Length::Fill))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .class(theme::Container::Transparent),
            )
            .on_press(Message::View(ViewMessage::CloseModal));

            // Show modal with image
            // Use preview image if available (contains edits), otherwise use cached
            let modal = if let Some(ref ann) = annotation {
                // Annotation mode: use AnnotationWidget
                if let Some(ref preview) = image_state.preview_image {
                    self.modal_annotate(preview, image_state, ann)
                } else if let Some(cached) = cache.get_full(path) {
                    self.modal_annotate(&cached, image_state, ann)
                } else {
                    self.modal_loading()
                }
            } else if let Some(ref preview) = image_state.preview_image {
                self.modal_content(preview, image_state)
            } else if let Some(cached) = cache.get_full(path) {
                self.modal_content(&cached, image_state)
            } else {
                self.modal_loading()
            };

            // Create a stack as a modal; this avoids the modal blocking other
            // UI elements.
            return stack![gallery, backdrop, modal].into();
        }

        gallery
    }
}
