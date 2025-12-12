//! Gallery view

use crate::{fl, image::ImageCache, message::Message, nav::NavState};
use cosmic::{
    Element,
    iced::{Alignment, Length},
    theme,
    widget::{self, container},
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

    /// Render the gallery view
    pub fn view(
        &self,
        _nav: &NavState,
        _cache: &ImageCache,
        _thumbnail_size: u32,
    ) -> Element<'_, Message> {
        let spacing = theme::active().cosmic().spacing;

        // Placeholder for gallery implementation
        let content = container(
            widget::column()
                .push(widget::icon::from_name("view-grid-symbolic").size(64))
                .push(widget::text(fl!("menu-gallery")).size(20))
                .push(widget::text("Gallery view coming soon...").size(14))
                .spacing(spacing.space_m)
                .align_x(Alignment::Center),
        )
        .center(Length::Fill);

        content.into()
    }
}
