use crate::{
    fl,
    image::{CachedImage, ImageCache},
    message::{Message, NavMessage, ViewMessage},
    nav::NavState,
    views::ImageViewState,
    widgets::flex_grid::{gallery_grid, GalleryItem},
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

        // Build gallery items
        let items: Vec<GalleryItem> = images
            .iter()
            .map(|path| GalleryItem::new(path.clone(), cache.get_thumbnail(path)))
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
            .on_scroll_request(|req| {
                Message::View(ViewMessage::GalleryScrollTo(req.offset_y))
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
        {
            // Use mouse-area to close the modal when the backdrop is clicked.
            let backdrop = mouse_area(
                container(Space::new(Length::Fill, Length::Fill))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .class(theme::Container::Transparent),
            )
            .on_press(Message::View(ViewMessage::CloseModal));

            // Show modal with image if cached, or loading state if not
            let modal = if let Some(cached) = cache.get_full(path) {
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
