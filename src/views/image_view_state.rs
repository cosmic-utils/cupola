use crate::message::Message;
use cosmic::{Task, iced_widget::scrollable, widget::Id};

const MODAL_SCROLL_ID: &str = "modal-image-scroll";

#[derive(Debug, Clone)]
pub struct ImageViewState {
    pub zoom_level: f32,
    pub fit_to_window: bool,
    pub fit_zoom: f32,
    pub scroll_id: Id,
    pub window_width: f32,
    pub window_height: f32,
}

impl Default for ImageViewState {
    fn default() -> Self {
        Self {
            zoom_level: 1.0,
            fit_to_window: true,
            fit_zoom: 1.0,
            scroll_id: Id::new(MODAL_SCROLL_ID),
            window_width: 0.0,
            window_height: 0.0,
        }
    }
}

impl ImageViewState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn zoom_in(&mut self) -> Task<Message> {
        if self.fit_to_window {
            self.zoom_level = self.fit_zoom;
        }
        self.fit_to_window = false;
        self.zoom_level = (self.zoom_level * 1.25).min(10.);
        self.scroll_to_center()
    }

    pub fn zoom_out(&mut self) -> Task<Message> {
        if self.fit_to_window {
            self.zoom_level = self.fit_zoom;
        }
        self.fit_to_window = false;
        self.zoom_level = (self.zoom_level / 1.25).max(0.1);
        self.scroll_to_center()
    }

    pub fn zoom_reset(&mut self) -> Task<Message> {
        self.fit_to_window = false;
        self.zoom_level = 1.0;
        self.scroll_to_center()
    }

    pub fn zoom_fit(&mut self) {
        self.fit_to_window = true;
    }

    pub fn set_window_size(&mut self, width: f32, height: f32) {
        self.window_width = width;
        self.window_height = height;
    }

    pub fn calculate_fit_zoom(&mut self, img_width: u32, img_height: u32) {
        if self.window_width <= 0.0 || self.window_height <= 0.0 {
            return; // No valid window dimensions yet
        }

        // Modal padding from window edges
        let modal_pad_x = 80.0 * 2.0;
        let modal_pad_y = 60.0 * 2.0;
        // Header and footer height
        let header_height = 48.0;
        let footer_height = 48.0;
        // Nav button widths
        let nav_btn_width = 48.0 * 2.0;
        // Container padding inside modal
        let container_pad = 16.0;

        let available_width = self.window_width - modal_pad_x - nav_btn_width - container_pad;
        let available_height =
            self.window_height - modal_pad_y - header_height - footer_height - container_pad;

        if available_width <= 0.0 || available_height <= 0.0 {
            return; // Window too small
        }

        let zoom_x = available_width / img_width as f32;
        let zoom_y = available_height / img_height as f32;
        self.fit_zoom = zoom_x.min(zoom_y).min(1.0);
    }

    fn scroll_to_center(&self) -> Task<Message> {
        scrollable::snap_to(
            self.scroll_id.clone(),
            scrollable::RelativeOffset { x: 0.5, y: 0.5 },
        )
    }
}
