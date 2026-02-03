#[cfg(test)]
mod tests {
    use shared::image::{LetterboxDimensions, calculate_aspect_ratio};
    use shared::thumbnail_metadata::ThumbnailMetadata;
    use std::path::PathBuf;

    #[test]
    fn test_extreme_aspect_ratio_10_1() {
        let dims = LetterboxDimensions::calculate(2560, 256, 256);
        assert_eq!(dims.final_width, 256);
        assert_eq!(dims.final_height, 256);
        assert_eq!(dims.image_width, 256);
        assert_eq!(dims.image_height, 26); // Should be ~25.6, rounded to 26
        assert!(dims.letterbox_top > 100); // Significant vertical padding
    }

    #[test]
    fn test_extreme_aspect_ratio_1_10() {
        let dims = LetterboxDimensions::calculate(256, 2560, 256);
        assert_eq!(dims.final_width, 256);
        assert_eq!(dims.final_height, 256);
        assert_eq!(dims.image_height, 256);
        assert_eq!(dims.image_width, 26); // Should be ~25.6, rounded to 26
        assert!(dims.letterbox_left > 100); // Significant horizontal padding
    }

    #[test]
    fn test_tiny_image_preserved() {
        let dims = LetterboxDimensions::calculate(16, 16, 256);
        // Small images (smaller than max_size) should preserve original dimensions
        // No upscaling is performed
        assert_eq!(dims.image_width, 16);
        assert_eq!(dims.image_height, 16);
        assert_eq!(dims.letterbox_left, 0);
        assert_eq!(dims.letterbox_top, 0);
    }

    #[test]
    fn test_panoramic_image() {
        let dims = LetterboxDimensions::calculate(2560, 1080, 256);
        // 21:9 panoramic should be pillarboxed
        assert!(dims.image_width <= 256);
        assert!(dims.image_height <= 256);
        assert_eq!(dims.final_width, 256);
        assert_eq!(dims.final_height, 256);
        // Should have vertical padding (pillarbox)
        assert!(dims.letterbox_top > 0);
    }

    #[test]
    fn test_aspect_ratio_calculation_precision() {
        let ratio = calculate_aspect_ratio(1920, 1080);
        let expected = 1920.0 / 1080.0;
        assert!(
            (ratio - expected).abs() < 0.0001,
            "Aspect ratio calculation imprecise"
        );
    }

    #[test]
    fn test_thumbnail_metadata_validation() {
        let metadata = ThumbnailMetadata::new(
            PathBuf::from("/test/image.jpg"),
            PathBuf::from("/cache/thumb.jpg"),
            1920,
            1080,
            "Landscape image".to_string(),
            "Alt text".to_string(),
        );

        assert!(metadata.aspect_ratio > 0.0);
        assert!(metadata.original_width > 0);
        assert!(metadata.original_height > 0);
    }

    #[test]
    fn test_letterbox_with_square_image() {
        let dims = LetterboxDimensions::calculate(100, 100, 256);
        // Square image smaller than max_size preserves original dimensions
        assert_eq!(dims.image_width, 100);
        assert_eq!(dims.image_height, 100);
        assert_eq!(dims.letterbox_left, 0);
        assert_eq!(dims.letterbox_top, 0);
    }

    #[test]
    fn test_centering_calculation() {
        let cell_width = 256;
        let image_width = 171; // ~2:3 aspect ratio

        let expected_x = ((cell_width - image_width) as f64 / 2.0).round() as u32;

        assert_eq!(expected_x, 43); // (256 - 171) / 2 = 42.5, rounded to 43
    }
}
