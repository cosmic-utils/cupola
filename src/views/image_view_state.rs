//! Single image view

use crate::{
    fl,
    image::ImageCache,
    message::{Message, NavMessage, ViewMessage},
    nav::NavState,
};
use cosmic::{
    Element,
    iced::{
        Alignment, ContentFit, Length,
        alignment::{Horizontal, Vertical},
    },
    theme,
    widget::{
        Space, button, column, container, horizontal_space, icon, image, mouse_area, row,
        scrollable, text,
    },
};

/// View state for single image display
#[derive(Debug, Clone)]
pub struct ImageViewState {
    /// Current zoom level (1.0 = 100%)
    pub zoom_level: f32,
    /// Fit mode enabled
    pub fit_to_window: bool,
    /// Pan offset
    pan_offset: (f32, f32),
    /// Show the prev button
    pub show_prev_btn: bool,
    /// Show the next button
    pub show_next_btn: bool,
}

impl Default for ImageViewState {
    fn default() -> Self {
        Self {
            zoom_level: 1.0,
            fit_to_window: true,
            pan_offset: (0., 0.),
            show_prev_btn: false,
            show_next_btn: false,
        }
    }
}

impl ImageViewState {
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
                        .height(Length::Fill)
                } else {
                    let scaled_width = cached.width as f32 * self.zoom_level;
                    let scaled_height = cached.height as f32 * self.zoom_level;

                    image(cached.handle.clone())
                        .content_fit(ContentFit::Fill)
                        .width(Length::Fixed(scaled_width))
                        .height(Length::Fixed(scaled_height))
                };

                if self.fit_to_window {
                    row()
                        .push(horizontal_space().width(Length::Fill))
                        .push(image_widget)
                        .push(horizontal_space().width(Length::Fill))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_y(Vertical::Center)
                        .padding(spacing.space_xxs)
                        .into()
                } else {
                    container(scrollable(
                        row()
                            .push(horizontal_space().width(Length::Fill))
                            .push(image_widget)
                            .push(horizontal_space().width(Length::Fill))
                            .align_y(Vertical::Center)
                            .padding(spacing.space_xxs),
                    ))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center(Length::Fill)
                    .into()
                }
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

        // Wrap content in prev/next overlay
        let has_prev = nav.index() > 0;
        let has_next = nav.index() < nav.total().saturating_sub(1);

        // Prev button zone
        let prev_zone = mouse_area(
            container(if self.show_prev_btn && has_prev {
                container(
                    button::icon(icon::from_name("go-previous-symbolic").size(32))
                        .on_press(Message::Nav(NavMessage::Prev))
                        .padding(spacing.space_s)
                        .class(theme::Button::Standard),
                )
            } else {
                container(Space::new(Length::Fixed(64.), Length::Fill))
            })
            .height(Length::Fill)
            .center_y(Length::Fill)
            .padding([0, spacing.space_xs]),
        )
        .on_enter(Message::View(ViewMessage::HoverPrev(true)))
        .on_exit(Message::View(ViewMessage::HoverPrev(false)));

        // Next button zone
        let next_zone = mouse_area(
            container(if self.show_next_btn && has_next {
                container(
                    button::icon(icon::from_name("go-next-symbolic").size(32))
                        .on_press(Message::Nav(NavMessage::Next))
                        .padding(spacing.space_s)
                        .class(theme::Button::Standard),
                )
            } else {
                container(Space::new(Length::Fixed(64.), Length::Fill))
            })
            .height(Length::Fill)
            .center_y(Length::Fill)
            .padding([0, spacing.space_xs]),
        )
        .on_enter(Message::View(ViewMessage::HoverNext(true)))
        .on_exit(Message::View(ViewMessage::HoverNext(false)));

        let main_row = row()
            .push(prev_zone)
            .push(content)
            .push(next_zone)
            .width(Length::Fill)
            .height(Length::Fill);

        let status_bar = if nav.total() > 0 {
            let status_text = fl!(
                "status-image-count",
                current = (nav.index() + 1).to_string(),
                total = nav.total().to_string()
            );

            let zoom_text = if self.fit_to_window {
                // TODO: Add status-zoom-fit to i18n: "Fit to Window"
                "Fit to Window".into()
            } else {
                fl!(
                    "status-zoom-level",
                    percent = self.zoom_percent().to_string()
                )
            };

            row()
                .push(text(status_text).size(12))
                .push(horizontal_space())
                .push(text(zoom_text).size(12))
                .padding([spacing.space_xxs, spacing.space_s])
        } else {
            row()
        };

        column()
            .push(main_row)
            .push(status_bar)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .into()
    }
}
