//! Gallery view

use std::path::PathBuf;

use crate::{
    fl,
    image::ImageCache,
    message::{Message, NavMessage, ViewMessage},
    nav::NavState,
};
use cosmic::{
    Element,
    iced::{Alignment, ContentFit, Length},
    theme,
    widget::{
        self, button, column, container, horizontal_space, icon, row, scrollable, text, tooltip,
    },
};

/// Gallery view state
#[derive(Debug, Clone, Default)]
pub struct GalleryView {
    /// Selected image indices
    pub selected: Vec<usize>,
    /// Number of cols in the grid
    pub cols: usize,
}

impl GalleryView {
    pub fn new() -> Self {
        Self {
            selected: Vec::new(),
            cols: 4,
        }
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

    /// Render the gallery view
    pub fn view(
        &self,
        nav: &NavState,
        cache: &ImageCache,
        thumbnail_size: u32,
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
        let mut grid = column().spacing(spacing.space_xs);
        let mut current_row = row().spacing(spacing.space_xs);
        let mut col_count = 0;

        for (index, path) in images.iter().enumerate() {
            let cell = self.thumbnail_cell(index, path, cache, thumbnail_size);
            current_row = current_row.push(cell);
            col_count += 1;

            if col_count >= self.cols {
                grid = grid.push(current_row);
                current_row = row().spacing(spacing.space_xs);
                col_count = 0;
            }
        }

        if col_count > 0 {
            grid = grid.push(current_row);
        }

        let content = scrollable(container(grid).padding(spacing.space_s).width(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill);

        // Status bar
        let status = row()
            .push(text(format!("{} images", images.len())).size(12))
            .push(horizontal_space())
            .push(
                button::icon(icon::from_name("view-restore-symbolic"))
                    .on_press(Message::View(ViewMessage::ShowSingle))
                    .padding(spacing.space_xxs),
            )
            .padding([spacing.space_xxs, spacing.space_s])
            .align_y(Alignment::Center);

        column()
            .push(content)
            .push(status)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
