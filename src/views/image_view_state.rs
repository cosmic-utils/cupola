//! Zoom/pan state for the modal image view

use crate::message::Message;
use cosmic::{Task, iced_widget::scrollable, widget::Id};

/// ID for the modal's scrollable widget
const MODAL_SCROLL_ID: &str = "modal-image-scroll";

/// View state for single image display
#[derive(Debug, Clone)]
pub struct ImageViewState {
    /// Current zoom level (1.0 = 100%)
    pub zoom_level: f32,
    /// Fit mode enabled
    pub fit_to_window: bool,
    /// Calculated fit zoom from viewport
    pub fit_zoom: f32,
    /// Scrollable ID for programmatic scroll control
    pub scroll_id: Id,
    /// Current window dimensions for fit_zoom calculation
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

    /// Zoom in by 25%
    pub fn zoom_in(&mut self) -> Task<Message> {
        // Start from fit_zoom if coming from fit mode
        if self.fit_to_window {
            self.zoom_level = self.fit_zoom;
        }
        self.fit_to_window = false;
        self.zoom_level = (self.zoom_level * 1.25).min(10.);
        self.scroll_to_center()
    }

    /// Zoom out by 25%
    pub fn zoom_out(&mut self) -> Task<Message> {
        // Start from fit_zoom if coming from fit mode
        if self.fit_to_window {
            self.zoom_level = self.fit_zoom;
        }
        self.fit_to_window = false;
        self.zoom_level = (self.zoom_level / 1.25).max(0.1);
        self.scroll_to_center()
    }

    /// Reset zoom to 100%
    pub fn zoom_reset(&mut self) -> Task<Message> {
        self.fit_to_window = false;
        self.zoom_level = 1.0;
        self.scroll_to_center()
    }

    /// Enable fit-to-window mode
    pub fn zoom_fit(&mut self) {
        self.fit_to_window = true;
    }

    /// Update stored window dimensions
    pub fn set_window_size(&mut self, width: f32, height: f32) {
        self.window_width = width;
        self.window_height = height;
    }

    /// Calculates fit_zoom from window size and image dimensions
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

    /// Scroll to center of the image
    fn scroll_to_center(&self) -> Task<Message> {
        scrollable::snap_to(
            self.scroll_id.clone(),
            scrollable::RelativeOffset { x: 0.5, y: 0.5 },
        )
    }
}
