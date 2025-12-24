//! Gallery view

use crate::{
    fl,
    image::{CachedImage, ImageCache},
    message::{Message, NavMessage, ViewMessage},
    nav::NavState,
    views::ImageViewState,
};
use cosmic::{
    Element,
    iced::{
        Alignment, ContentFit, Length,
        alignment::{Horizontal, Vertical},
    },
    iced_widget::scrollable::{Direction, Scrollbar},
    theme,
    widget::{
        self, button, column, container, flex_row, horizontal_space, icon, image, popover, row,
        scrollable, text, tooltip, vertical_space,
    },
};
use std::path::PathBuf;

/// Gallery view state
#[derive(Debug, Clone, Default)]
pub struct GalleryView {
    /// Selected image indices
    pub selected: Vec<usize>,
    /// Number of cols in the grid
    pub cols: usize,
    /// Currently open modal image index
    pub modal_index: Option<usize>,
}

impl GalleryView {
    pub fn new() -> Self {
        Self {
            selected: Vec::new(),
            cols: 4,
            modal_index: None,
        }
    }

    /// Open modal for an image selection
    pub fn open_modal(&mut self, index: usize) {
        self.modal_index = Some(index);
    }

    /// Close an open modal
    pub fn close_modal(&mut self) {
        self.modal_index = None;
    }

    /// Check if an image selected modal is open
    pub fn is_modal_open(&self) -> bool {
        self.modal_index.is_some()
    }

    /// Toggle selection of an image
    pub fn toggle_selection(&mut self, idx: usize) {
        if let Some(pos) = self.selected.iter().position(|&i| i == idx) {
            self.selected.remove(pos);
        } else {
            self.selected.push(idx);
        }
    }

    /// Clear all selections
    pub fn clear_selection(&mut self) {
        self.selected.clear();
    }

    /// Check if an index is selected
    pub fn is_selected(&self, idx: usize) -> bool {
        self.selected.contains(&idx)
    }

    /// Create a thumbnail cell
    fn thumbnail_cell<'a>(
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

        let cell = button::custom(content)
            .on_press(Message::Nav(NavMessage::GallerySelect(index)))
            .padding(spacing.space_xxs)
            .class(if is_selected {
                theme::Button::Suggested
            } else {
                theme::Button::Image
            });

        tooltip(cell, text(file_name), tooltip::Position::Bottom).into()
    }

    /// Build the modal content for viewing a single image
    fn modal_content<'a>(
        &self,
        cached: &CachedImage,
        image_state: &ImageViewState,
    ) -> Element<'static, Message> {
        let spacing = theme::active().cosmic().spacing;

        let image_widget = if image_state.fit_to_window {
            image(cached.handle.clone())
                .content_fit(ContentFit::Contain)
                .width(Length::Shrink)
                .height(Length::Shrink)
        } else {
            let scaled_width = cached.width as f32 * image_state.zoom_level;
            let scaled_height = cached.height as f32 * image_state.zoom_level;

            image(cached.handle.clone())
                .content_fit(ContentFit::Fill)
                .width(Length::Fixed(scaled_width))
                .height(Length::Fixed(scaled_height))
        };

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

        let img_container = if image_state.fit_to_window {
            container(image_widget)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .center(Length::Fill)
                .padding(spacing.space_xs)
        } else {
            // Wrap in scrollable for zoomed images, centering content
            container(
                scrollable(
                    container(image_widget)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center(Length::Fill)
                        .padding(spacing.space_xxs),
                )
                .direction(Direction::Both {
                    vertical: Scrollbar::default(),
                    horizontal: Scrollbar::default(),
                })
                .width(Length::Fill)
                .height(Length::Fill),
            )
        };

        let content_row = row()
            .push(prev_btn)
            .push(img_container)
            .push(next_btn)
            .width(Length::Fill)
            .height(Length::Fill);

        let close_btn = button::icon(icon::from_name("window-close-symbolic"))
            .on_press(Message::View(ViewMessage::CloseModal))
            .padding(spacing.space_xs)
            .class(theme::Button::Destructive);

        let zoom_ctrls = row()
            .push(
                button::icon(icon::from_name("zoom-out-symbolic"))
                    .on_press(Message::View(ViewMessage::ZoomOut))
                    .padding(spacing.space_xs),
            )
            .push(if image_state.fit_to_window {
                container(text::body("Fit to Window")).padding(spacing.space_xs)
            } else {
                container(
                    button::text(format!("{}%", image_state.zoom_percent()))
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

        let header = row()
            .push(horizontal_space())
            .push(close_btn)
            .width(Length::Fill)
            .padding(spacing.space_xs);

        let footer = row()
            .push(horizontal_space())
            .push(zoom_ctrls)
            .push(horizontal_space())
            .width(Length::Fill)
            .padding(spacing.space_xs);

        // Outer container creates the gaps around the modal to show the gallery,
        // title bar, and status bar.
        container(
            // Inner container serves as a container for the header, image, and footer.
            container(
                // Column keeps the header, image, and footer aligned.
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
        .padding([60, 80])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    /// Render the gallery view
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
            cells.push(cell.into());
        }

        let grid = flex_row(cells)
            .column_spacing(spacing.space_xs)
            .row_spacing(spacing.space_xs);

        let content = scrollable(container(grid).padding(spacing.space_s).width(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill);

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
        if let Some(idx) = self.modal_index
            && let Some(path) = images.get(idx)
            && let Some(cached) = cache.get_full(path)
        {
            let modal = self.modal_content(&cached, image_state);

            return popover(gallery)
                .popup(modal)
                .modal(true)
                .position(popover::Position::Center)
                .on_close(Message::View(ViewMessage::CloseModal))
                .into();
        }

        gallery
    }
}
