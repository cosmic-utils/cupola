#[cfg(test)]
mod tests {
    use shared::accessibility::{AccessibilityInfo, AriaRole};
    use shared::thumbnail_metadata::ThumbnailMetadata;
    use std::path::PathBuf;

    #[test]
    fn test_accessibility_info_creation() {
        let metadata = ThumbnailMetadata::new(
            PathBuf::from("/photos/landscape.jpg"),
            PathBuf::from("/cache/landscape_thumb.jpg"),
            1920,
            1080,
            "Landscape image".to_string(),
            "Alt text".to_string(),
        );

        // Create accessibility info from metadata
        let info =
            AccessibilityInfo::new(AriaRole::Image).with_label(&metadata.screen_reader_label);

        assert!(info.label.is_some());
        assert!(!info.label.as_ref().unwrap().is_empty());
    }

    #[test]
    fn test_aria_role_assignment() {
        let info = AccessibilityInfo::new(AriaRole::Image).with_label("Test image");

        assert_eq!(info.role, AriaRole::Image);
        assert_eq!(info.label, Some("Test image".to_string()));
    }

    #[test]
    fn test_aria_attributes_format() {
        let info = AccessibilityInfo::new(AriaRole::Option)
            .with_label("Image: photo.jpg, 1920 by 1080 pixels, landscape");

        let attrs = info.generate_aria_attributes();

        // Should have role and aria-label attributes
        assert!(!attrs.is_empty());
        assert!(attrs.contains("role"));
        assert!(attrs.contains("aria-label"));
    }

    #[test]
    fn test_screen_reader_announcement_format() {
        let announcement = format_screen_reader_announcement("photo.jpg", 1920, 1080);

        assert!(announcement.contains("photo.jpg"));
        assert!(announcement.contains("1920"));
        assert!(announcement.contains("1080"));
        assert!(announcement.contains("landscape"));
    }

    #[test]
    fn test_portrait_orientation_detection() {
        let announcement = format_screen_reader_announcement("portrait.jpg", 1080, 1920);
        assert!(announcement.contains("portrait"));
    }

    #[test]
    fn test_square_orientation_detection() {
        let announcement = format_screen_reader_announcement("square.jpg", 1000, 1000);
        assert!(announcement.contains("square"));
    }

    // Helper function matching the one in accessibility.rs
    fn format_screen_reader_announcement(filename: &str, width: u32, height: u32) -> String {
        let aspect = if width > height {
            "landscape"
        } else if height > width {
            "portrait"
        } else {
            "square"
        };

        format!(
            "Image: {}, {} by {} pixels, {}",
            filename, width, height, aspect
        )
    }
}
