use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailMetadata {
    pub source_path: PathBuf,
    pub thumbnail_path: PathBuf,
    pub original_width: u32,
    pub original_height: u32,
    pub aspect_ratio: f64,
    pub thumbnail_size: u32,
    pub letterbox_width: u32,
    pub letterbox_height: u32,
    pub created_at: SystemTime,
    pub cache_key: String,
    pub retry_count: u8,
    pub screen_reader_label: String,
    pub accessibility_alt_text: String,
}

impl ThumbnailMetadata {
    pub fn new(
        source_path: PathBuf,
        thumbnail_path: PathBuf,
        original_width: u32,
        original_height: u32,
        screen_reader_label: String,
        accessibility_alt_text: String,
    ) -> Self {
        let aspect_ratio = if original_height > 0 {
            original_width as f64 / original_height as f64
        } else {
            0.0
        };

        Self {
            source_path,
            thumbnail_path,
            original_width,
            original_height,
            aspect_ratio,
            thumbnail_size: 256, // Fixed thumbnail size
            letterbox_width: 256,
            letterbox_height: 256,
            created_at: SystemTime::now(),
            cache_key: String::new(), // Will be set by cache system
            retry_count: 0,
            screen_reader_label,
            accessibility_alt_text,
        }
    }

    pub fn with_cache_key(mut self, cache_key: String) -> Self {
        self.cache_key = cache_key;
        self
    }

    pub fn is_valid_aspect_ratio(&self) -> bool {
        self.aspect_ratio > 0.0 && self.aspect_ratio.is_finite()
    }

    pub fn is_landscape(&self) -> bool {
        self.aspect_ratio > 1.0
    }

    pub fn is_portrait(&self) -> bool {
        self.aspect_ratio < 1.0
    }

    pub fn is_square(&self) -> bool {
        (self.aspect_ratio - 1.0).abs() < f64::EPSILON
    }

    pub fn can_retry(&self) -> bool {
        self.retry_count < 3
    }

    pub fn increment_retry_count(mut self) -> Self {
        self.retry_count += 1;
        self
    }

    pub fn file_size_mb(&self) -> f64 {
        match std::fs::metadata(&self.thumbnail_path) {
            Ok(metadata) => metadata.len() as f64 / (1024.0 * 1024.0),
            Err(_) => 0.0,
        }
    }

    pub fn age_seconds(&self) -> u64 {
        SystemTime::now()
            .duration_since(self.created_at)
            .unwrap_or_default()
            .as_secs()
    }

    pub fn format_duration(&self) -> String {
        let seconds = self.age_seconds();
        if seconds < 60 {
            format!("{}s", seconds)
        } else if seconds < 3600 {
            format!("{}m {}s", seconds / 60, seconds % 60)
        } else {
            format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.aspect_ratio <= 0.0 || !self.aspect_ratio.is_finite() {
            return Err("Invalid aspect ratio".to_string());
        }

        if self.thumbnail_size != 256 {
            return Err("Thumbnail size must be 256px".to_string());
        }

        if self.letterbox_width != 256 || self.letterbox_height != 256 {
            return Err("Letterbox dimensions must be 256x256px".to_string());
        }

        if !self.source_path.exists() {
            return Err("Source file does not exist".to_string());
        }

        if self.retry_count > 3 {
            return Err("Retry count exceeds maximum".to_string());
        }

        if self.screen_reader_label.is_empty() {
            return Err("Screen reader label cannot be empty".to_string());
        }

        if self.accessibility_alt_text.is_empty() {
            return Err("Accessibility alt text cannot be empty".to_string());
        }

        Ok(())
    }
}

impl Default for ThumbnailMetadata {
    fn default() -> Self {
        Self {
            source_path: PathBuf::new(),
            thumbnail_path: PathBuf::new(),
            original_width: 0,
            original_height: 0,
            aspect_ratio: 0.0,
            thumbnail_size: 256,
            letterbox_width: 256,
            letterbox_height: 256,
            created_at: SystemTime::now(),
            cache_key: String::new(),
            retry_count: 0,
            screen_reader_label: String::new(),
            accessibility_alt_text: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thumbnail_metadata_creation() {
        let source = PathBuf::from("/test/image.jpg");
        let thumbnail = PathBuf::from("/test/thumb.jpg");

        let metadata = ThumbnailMetadata::new(
            source.clone(),
            thumbnail.clone(),
            800,
            600,
            "Test image".to_string(),
            "A test image for unit testing".to_string(),
        )
        .with_cache_key("test_key".to_string());

        assert_eq!(metadata.source_path, source);
        assert_eq!(metadata.thumbnail_path, thumbnail);
        assert_eq!(metadata.original_width, 800);
        assert_eq!(metadata.original_height, 600);
        assert_eq!(metadata.aspect_ratio, 4.0 / 3.0);
        assert!(metadata.is_landscape());
        assert!(!metadata.is_portrait());
        assert!(!metadata.is_square());
        assert_eq!(metadata.cache_key, "test_key");
    }

    #[test]
    fn test_aspect_ratio_detection() {
        let square = ThumbnailMetadata::new(
            PathBuf::from("/test.jpg"),
            PathBuf::from("/thumb.jpg"),
            100,
            100,
            "Square".to_string(),
            "Square image".to_string(),
        );
        assert!(square.is_square());
        assert!(!square.is_landscape());
        assert!(!square.is_portrait());

        let landscape = ThumbnailMetadata::new(
            PathBuf::from("/test.jpg"),
            PathBuf::from("/thumb.jpg"),
            200,
            100,
            "Landscape".to_string(),
            "Landscape image".to_string(),
        );
        assert!(landscape.is_landscape());
        assert!(!landscape.is_portrait());
        assert!(!landscape.is_square());

        let portrait = ThumbnailMetadata::new(
            PathBuf::from("/test.jpg"),
            PathBuf::from("/thumb.jpg"),
            100,
            200,
            "Portrait".to_string(),
            "Portrait image".to_string(),
        );
        assert!(portrait.is_portrait());
        assert!(!portrait.is_landscape());
        assert!(!portrait.is_square());
    }

    #[test]
    fn test_retry_logic() {
        let mut metadata = ThumbnailMetadata::new(
            PathBuf::from("/test.jpg"),
            PathBuf::from("/thumb.jpg"),
            100,
            100,
            "Test".to_string(),
            "Test image".to_string(),
        );

        assert_eq!(metadata.retry_count, 0);
        assert!(metadata.can_retry());

        metadata = metadata.increment_retry_count();
        assert_eq!(metadata.retry_count, 1);
        assert!(metadata.can_retry());

        metadata = metadata.increment_retry_count();
        metadata = metadata.increment_retry_count();
        assert_eq!(metadata.retry_count, 3);
        assert!(!metadata.can_retry());
    }

    #[test]
    fn test_validation() {
        let valid = ThumbnailMetadata::new(
            PathBuf::from("/nonexistent.jpg"),
            PathBuf::from("/thumb.jpg"),
            100,
            100,
            "Test".to_string(),
            "Test image".to_string(),
        );

        // This should fail because source file doesn't exist
        assert!(valid.validate().is_err());
    }

    #[test]
    fn test_edge_cases() {
        let zero_height = ThumbnailMetadata::new(
            PathBuf::from("/test.jpg"),
            PathBuf::from("/thumb.jpg"),
            100,
            0,
            "Test".to_string(),
            "Test image".to_string(),
        );
        assert_eq!(zero_height.aspect_ratio, 0.0);
        assert!(!zero_height.is_valid_aspect_ratio());
    }
}
