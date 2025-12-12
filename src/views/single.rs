//! Single image view

use crate::{fl, image::ImageCache, message::Message, nav::NavState};
use cosmic::{
    Element,
    iced::{Alignment, ContentFit, Length},
    theme,
    widget::{column, container, horizontal_space, icon, image, row, scrollable, text},
};

/// View state for single image display
#[derive(Debug, Clone)]
pub struct SingleView {
    /// Current zoom level (1.0 = 100%)
    pub zoom_level: f32,
    /// Fit mode enabled
    pub fit_to_window: bool,
    /// Pan offset
    pan_offset: (f32, f32),
}

impl Default for SingleView {
    fn default() -> Self {
        Self {
            zoom_level: 1.0,
            fit_to_window: true,
            pan_offset: (0., 0.),
        }
    }
}

impl SingleView {
    pub fn new() -> Self {
        Self::default()
    }

    /// Zoom in by 25%
    pub fn zoom_in(&mut self) {
        self.fit_to_window = false;
        self.zoom_level = (self.zoom_level * 1.25).min(10.);
    }

    /// Zoom out by 25%
    pub fn zoom_out(&mut self) {
        self.fit_to_window = false;
        self.zoom_level = (self.zoom_level / 1.25).max(0.1);
    }

    /// Reset zoom to 100%
    pub fn zoom_reset(&mut self) {
        self.fit_to_window = false;
        self.zoom_level = 1.0;
        self.pan_offset = (0., 0.);
    }

    /// Enable fit-to-window mode
    pub fn zoom_fit(&mut self) {
        self.fit_to_window = true;
        self.zoom_level = 1.0;
        self.pan_offset = (0., 0.);
    }

    /// Pan the image
    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.pan_offset.0 += dx;
        self.pan_offset.1 += dy;
    }

    /// Get zoom as percentage for display
    pub fn zoom_percent(&self) -> u32 {
        (self.zoom_level * 100.) as u32
    }

    /// Render the single image view
    pub fn view(
        &self,
        nav: &NavState,
        cache: &ImageCache,
        is_loading: bool,
    ) -> Element<'_, Message> {
        let spacing = theme::active().cosmic().spacing;

        let content: Element<'_, Message> = if is_loading {
            container(
                column()
                    .push(text(fl!("status-loading")))
                    .spacing(spacing.space_s)
                    .align_x(Alignment::Center),
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        } else if let Some(path) = nav.current() {
            if let Some(cached) = cache.get_full(path) {
                let image_widget = if self.fit_to_window {
                    image(cached.handle.clone())
                        .content_fit(ContentFit::Contain)
                        .width(Length::Fill)
                        .height(Length::Fill)
                } else {
                    image(cached.handle.clone())
                        .content_fit(ContentFit::None)
                        .width(Length::Shrink)
                        .height(Length::Shrink)
                };

                scrollable(
                    container(image_widget)
                        .center(Length::Fill)
                        .padding(spacing.space_xxs),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            } else {
                container(
                    column()
                        .push(text(fl!("status-loading")))
                        .spacing(spacing.space_s)
                        .align_x(Alignment::Center),
                )
                .center(Length::Fill)
                .into()
            }
        } else {
            container(
                column()
                    .push(icon::from_name("image-x-generic-symbolic").size(64))
                    .push(text(fl!("status-no-image")).size(16))
                    .spacing(spacing.space_m)
                    .align_x(Alignment::Center),
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        };

        let status_bar = if nav.total() > 0 {
            let status_text = fl!(
                "status-image-count",
                current = (nav.index() + 1).to_string(),
                total = nav.total().to_string()
            );

            let zoom_text = fl!(
                "status-zoom-level",
                percent = self.zoom_percent().to_string()
            );

            row()
                .push(text(status_text).size(12))
                .push(horizontal_space())
                .push(text(zoom_text).size(12))
                .padding([spacing.space_xxs, spacing.space_s])
        } else {
            row()
        };

        column().push(content).push(status_bar).into()
    }
}
