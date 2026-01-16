use crate::{
    fl,
    image::{CachedImage, ImageCache},
    message::{Message, NavMessage, ViewMessage},
    nav::NavState,
    views::ImageViewState,
    widgets::flex_grid,
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
        self, Id, Space, button, column, container, horizontal_space, icon, image, mouse_area,
        responsive, row, scrollable, text, tooltip,
    },
};
use std::path::PathBuf;

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

    fn thumbnail_cell(
        &self,
        index: usize,
        path: &PathBuf,
        cache: &ImageCache,
        size: u32,
    ) -> Element<'static, Message> {
        let spacing = theme::active().cosmic().spacing;
        let is_selected = self.is_selected(index);

        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let content: Element<'_, Message> = if let Some(handle) = cache.get_thumbnail(path) {
            widget::image(handle)
                .width(Length::Fixed(size as f32))
                .height(Length::Fixed(size as f32))
                .content_fit(ContentFit::Contain)
                .into()
        } else {
            container(icon::from_name("image-x-generic-symbolic").size((size / 2) as u16))
                .width(Length::Fixed(size as f32))
                .height(Length::Fixed(size as f32))
                .center(Length::Fill)
                .into()
        };

        let button_id = Id::new(format!("thumbnail-{index}"));

        let cell = button::custom(content)
            .id(button_id)
            .selected(self.focused_index == Some(index))
            .on_press(Message::Nav(NavMessage::GallerySelect(index)))
            .padding(spacing.space_xxs)
            .class(if is_selected {
                theme::Button::Suggested
            } else {
                theme::Button::Image
            });

        tooltip(cell, text(file_name), tooltip::Position::Bottom).into()
    }

    fn modal_content(
        &self,
        cached: &CachedImage,
        image_state: &ImageViewState,
    ) -> Element<'static, Message> {
        let spacing = theme::active().cosmic().spacing;

        // Data for responsive closure
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

        let close_btn = button::icon(icon::from_name("window-close-symbolic"))
            .on_press(Message::View(ViewMessage::CloseModal))
            .padding(spacing.space_xs)
            .class(theme::Button::Destructive);

        let header = row()
            .push(horizontal_space())
            .push(close_btn)
            .width(Length::Fill)
            .padding(spacing.space_xs);

        // Responsive gives us viewport size for zoom calc
        let image_area = responsive(move |size| {
            // Account for container padding
            let available_width = size.width - (spacing.space_xs * 2) as f32;
            let available_height = size.height - (spacing.space_xs * 2) as f32;

            // Calculate fit zoom level
            let fit_zoom_calc = {
                let zoom_x = available_width / img_width;
                let zoom_y = available_height / img_height;
                zoom_x.min(zoom_y).min(1.0)
            };

            // Pick zoom based on mode
            let effective_zoom = if fit_to_window {
                fit_zoom_calc
            } else {
                zoom_level
            };

            let scaled_width = img_width * effective_zoom;
            let scaled_height = img_height * effective_zoom;

            // Center padding
            let pad_x = ((available_width - scaled_width) / 2.0).max(0.0);
            let pad_y = ((available_height - scaled_height) / 2.0).max(0.0);

            let image_widget = image(handle.clone())
                .content_fit(ContentFit::Fill)
                .width(Length::Fixed(scaled_width))
                .height(Length::Fixed(scaled_height));

            // Scrollable only when zoomed past viewport
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

    pub fn view(
        &self,
        nav: &NavState,
        cache: &ImageCache,
        thumbnail_size: u32,
        image_state: &ImageViewState,
    ) -> Element<'_, Message> {
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

        // Build grid
        let mut cells = Vec::new();

        for (index, path) in images.iter().enumerate() {
            let cell = self.thumbnail_cell(index, path, cache, thumbnail_size);
            cells.push(cell);
        }

        let item_width = thumbnail_size as f32 + (spacing.space_xxs * 2) as f32;

        let content = flex_grid(cells)
            .item_width(item_width)
            .column_spacing(spacing.space_xs)
            .row_spacing(spacing.space_xs)
            .padding(spacing.space_s)
            .scrollable(Id::new(Self::SCROLL_ID))
            .scroll_to_item(self.focused_index.unwrap_or(0))
            .on_scroll(|vp| Message::View(ViewMessage::GalleryScroll(vp)))
            .on_layout_changed(|cols, row_height, scroll_request| {
                Message::View(ViewMessage::GalleryLayoutChanged {
                    cols,
                    row_height,
                    scroll_request,
                })
            })
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
            .height(Length::Fill)
            .into();

        // If modal is open wrap with popover
        if let Some(idx) = nav.index()
            && let Some(path) = images.get(idx)
            && let Some(cached) = cache.get_full(path)
        {
            let modal = self.modal_content(&cached, image_state);

            // Use mouse-area to close the modal when the backdrop is clicked.
            let backdrop = mouse_area(
                container(Space::new(Length::Fill, Length::Fill))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .class(theme::Container::Transparent),
            )
            .on_press(Message::View(ViewMessage::CloseModal));

            // Create a stack as a modal; this avoids the modal blocking other
            // UI elements.
            return stack![gallery, backdrop, modal].into();
        }

        gallery
    }
}
