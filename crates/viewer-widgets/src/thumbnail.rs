use cosmic::iced_core::alignment;
use cosmic::widget::image::Handle;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ThumbnailWidget {
    handle: Option<cosmic::widget::image::Handle>,
    loading_state: crate::loading_state::LoadingState,
    metadata: Option<viewer_types::ThumbnailMetadata>,
    letterbox_width: u32,
    letterbox_height: u32,
    letterbox_background_color: cosmic::iced::Color,
    aspect_ratio: f64,
    generation_time_ms: u64,
    retry_used: u8,
    screen_reader_label: String,
    accessibility_alt_text: String,
}

impl ThumbnailWidget {
    pub fn new() -> Self {
        Self {
            handle: None,
            loading_state: crate::loading_state::LoadingState::default(),
            metadata: None,
            letterbox_width: 256,
            letterbox_height: 256,
            letterbox_background_color: cosmic::iced::Color::from_rgb(0.0, 0.0, 0.0), // Default black
            aspect_ratio: 0.0,
            generation_time_ms: 0,
            retry_used: 0,
            screen_reader_label: String::new(),
            accessibility_alt_text: String::new(),
        }
    }

    pub fn handle(&mut self, handle: cosmic::widget::image::Handle) {
        self.handle = Some(handle);
    }

    pub fn set_loading_state(&mut self, state: crate::loading_state::LoadingState) {
        self.loading_state = state;
    }

    pub fn set_metadata(&mut self, metadata: Option<viewer_types::ThumbnailMetadata>) {
        self.metadata = metadata;
    }

    pub fn letterbox_dimensions(&self) -> (u32, u32) {
        (self.letterbox_width, self.letterbox_height)
    }

    pub fn center_image_in_cell(
        &self,
        cell_width: u32,
        cell_height: u32,
    ) -> cosmic::iced_core::Rectangle {
        let image_width = if self.letterbox_width > 0 {
            self.letterbox_width
        } else {
            self.metadata.as_ref().map_or(cell_width, |m| {
                if m.aspect_ratio > 1.0 {
                    ((cell_width as f64) * cell_height as f64).round() as u32
                } else {
                    cell_width
                }
            })
        };

        let image_height = if self.letterbox_height > 0 {
            self.letterbox_height
        } else {
            self.metadata.as_ref().map_or(cell_height, |_m| cell_height)
        };

        let x = ((cell_width.saturating_sub(image_width)) as f64 / 2.0).round() as f32;
        let y = ((cell_height.saturating_sub(image_height)) as f64 / 2.0).round() as f32;

        cosmic::iced_core::Rectangle {
            x,
            y,
            width: image_width as f32,
            height: image_height as f32,
        }
    }

    pub fn letterbox_background_color(&self) -> cosmic::iced::Color {
        self.letterbox_background_color
    }

    pub fn set_letterbox_background_color(&mut self, color: cosmic::iced::Color) {
        self.letterbox_background_color = color;
    }

    pub fn accessibility_label(&self) -> String {
        if let Some(ref metadata) = self.metadata {
            metadata.screen_reader_label.clone()
        } else {
            String::new()
        }
    }

    pub fn accessibility_alt_text(&self) -> String {
        if let Some(ref metadata) = self.metadata {
            metadata.accessibility_alt_text.clone()
        } else {
            String::new()
        }
    }

    pub fn is_loading(&self) -> bool {
        matches!(
            self.loading_state,
            crate::loading_state::LoadingState::Loading
        )
    }

    pub fn is_ready(&self) -> bool {
        matches!(
            self.loading_state,
            crate::loading_state::LoadingState::Ready { .. }
        )
    }

    pub fn is_error(&self) -> bool {
        matches!(
            self.loading_state,
            crate::loading_state::LoadingState::Error { .. }
        )
    }

    pub fn aspect_ratio(&self) -> f64 {
        self.aspect_ratio
    }

    pub fn generation_time_ms(&self) -> u64 {
        self.generation_time_ms
    }

    pub fn retry_used(&self) -> u8 {
        self.retry_used
    }

    pub fn set_handle(&mut self, handle: Option<cosmic::widget::image::Handle>) {
        self.handle = handle;
    }

    pub fn set_letterbox_dimensions(&mut self, width: u32, height: u32) {
        self.letterbox_width = width;
        self.letterbox_height = height;
    }

    pub fn metadata(&self) -> Option<&viewer_types::ThumbnailMetadata> {
        self.metadata.as_ref()
    }
}

impl Default for ThumbnailWidget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thumbnail_widget_new() {
        let widget = ThumbnailWidget::new();
        assert!(widget.handle.is_none());
        assert_eq!(widget.letterbox_width, 256);
        assert_eq!(widget.letterbox_height, 256);
        assert_eq!(widget.aspect_ratio, 0.0);
        assert_eq!(widget.generation_time_ms, 0);
        assert_eq!(widget.retry_used, 0);
    }

    #[test]
    fn test_thumbnail_widget_default() {
        let widget = ThumbnailWidget::default();
        assert!(widget.handle.is_none());
        assert_eq!(widget.letterbox_width, 256);
        assert_eq!(widget.letterbox_height, 256);
    }

    #[test]
    fn test_letterbox_dimensions() {
        let widget = ThumbnailWidget::new();
        assert_eq!(widget.letterbox_dimensions(), (256, 256));
    }

    #[test]
    fn test_accessibility_methods_empty() {
        let widget = ThumbnailWidget::new();
        assert_eq!(widget.accessibility_label(), "");
        assert_eq!(widget.accessibility_alt_text(), "");
    }

    #[test]
    fn test_loading_state_methods() {
        let widget = ThumbnailWidget::new();
        // Default state should not be loading, ready, or error
        // (assuming default is Idle or similar)
        assert!(!widget.is_loading());
        assert!(!widget.is_ready());
        assert!(!widget.is_error());
    }

    #[test]
    fn test_aspect_ratio() {
        let widget = ThumbnailWidget::new();
        assert_eq!(widget.aspect_ratio(), 0.0);
    }

    #[test]
    fn test_generation_time_ms() {
        let widget = ThumbnailWidget::new();
        assert_eq!(widget.generation_time_ms(), 0);
    }

    #[test]
    fn test_retry_used() {
        let widget = ThumbnailWidget::new();
        assert_eq!(widget.retry_used(), 0);
    }
}
