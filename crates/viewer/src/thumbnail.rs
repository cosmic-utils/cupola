use shared::{
    accessibility::AccessibilityInfo,
    cache::ThumbnailCache,
    image::{ImageProcessingError, ImageResult, load_image_from_path, resize_with_letterbox},
    loading_state::LoadingState,
    thumbnail_metadata::ThumbnailMetadata,
};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

pub type ThumbnailResult<T> = Result<T, ThumbnailError>;

#[derive(thiserror::Error, Debug)]
pub enum ThumbnailError {
    #[error("Image processing failed: {0}")]
    ImageProcessing(#[from] ImageProcessingError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Exceeded retry limit for {0}")]
    RetryLimitExceeded(String),

    #[error("Generation timeout")]
    Timeout,

    #[error("Invalid source path: {0}")]
    InvalidSourcePath(String),
}

pub struct GenerateThumbnailRequest {
    pub source_path: PathBuf,
    pub max_size: u32,
    pub quality: u8,
    pub accessibility_mode: bool,
    pub retry_count: u8,
}

impl GenerateThumbnailRequest {
    pub fn new(source_path: PathBuf) -> Self {
        Self {
            source_path,
            max_size: 256,
            quality: 80,
            accessibility_mode: true,
            retry_count: 0,
        }
    }
}

pub struct GenerateThumbnailResponse {
    pub thumbnail_path: PathBuf,
    pub original_width: u32,
    pub original_height: u32,
    pub aspect_ratio: f64,
    pub letterbox_width: u32,
    pub letterbox_height: u32,
    pub generation_time_ms: u64,
    pub screen_reader_label: String,
    pub accessibility_alt_text: String,
    pub retry_used: u8,
}

pub struct ThumbnailService {
    cache: ThumbnailCache,
    thumbnail_dir: PathBuf,
    max_retries: u8,
    retry_delays: [Duration; 3], // 100ms, 200ms, 400ms
}

impl ThumbnailService {
    pub fn new(cache: ThumbnailCache, thumbnail_dir: PathBuf) -> Self {
        Self {
            cache,
            thumbnail_dir,
            max_retries: 3,
            retry_delays: [
                Duration::from_millis(100),
                Duration::from_millis(200),
                Duration::from_millis(400),
            ],
        }
    }

    pub async fn generate_thumbnail(
        &self,
        request: GenerateThumbnailRequest,
    ) -> ThumbnailResult<GenerateThumbnailResponse> {
        let start_time = Instant::now();

        // Check if we already have a cached thumbnail
        let cache_key = ThumbnailCache::generate_cache_key(&request.source_path);
        if let Some(_) = self.cache.get(&cache_key).await {
            debug!("Using cached thumbnail for: {:?}", request.source_path);

            // For now, return a simple response without complex metadata
            return Ok(GenerateThumbnailResponse {
                thumbnail_path: self.get_cached_thumbnail_path(&request.source_path),
                original_width: 0,
                original_height: 0,
                aspect_ratio: 0.0,
                letterbox_width: request.max_size,
                letterbox_height: request.max_size,
                generation_time_ms: 0,
                screen_reader_label: format!("Cached thumbnail for {:?}", request.source_path),
                accessibility_alt_text: "Cached thumbnail".to_string(),
                retry_used: 0,
            });
        }

        // Generate new thumbnail with retry logic
        self.generate_thumbnail_with_retry(request, start_time)
            .await
    }

    async fn generate_thumbnail_with_retry(
        &self,
        mut request: GenerateThumbnailRequest,
        start_time: Instant,
    ) -> ThumbnailResult<GenerateThumbnailResponse> {
        let mut last_error = ThumbnailError::Timeout;

        for attempt in 0..self.max_retries {
            request.retry_count = attempt as u8;

            match self.attempt_thumbnail_generation(&request).await {
                Ok(response) => {
                    info!(
                        "Successfully generated thumbnail for {:?} in {}ms",
                        request.source_path,
                        start_time.elapsed().as_millis()
                    );

                    // Cache the result
                    let cache_key = ThumbnailCache::generate_cache_key(&request.source_path);
                    if let Ok(thumbnail_data) = std::fs::read(&response.thumbnail_path).await {
                        self.cache.put(cache_key, thumbnail_data).await;
                    }

                    return Ok(response);
                }
                Err(error) => {
                    last_error = error.clone();
                    warn!(
                        "Thumbnail generation attempt {} failed for {:?}: {}",
                        attempt + 1,
                        request.source_path,
                        error
                    );

                    // Don't sleep on the last attempt
                    if attempt < self.max_retries - 1 {
                        let delay = self.retry_delays[attempt.min(2)];
                        debug!("Retrying thumbnail generation in {:?}", delay);
                        sleep(delay).await;
                    }
                }
            }
        }

        error!(
            "All thumbnail generation attempts failed for {:?}",
            request.source_path
        );
        Err(ThumbnailError::RetryLimitExceeded(
            request.source_path.display().to_string(),
        ))
    }

    async fn attempt_thumbnail_generation(
        &self,
        request: &GenerateThumbnailRequest,
    ) -> ThumbnailResult<GenerateThumbnailResponse> {
        // Validate source path
        if !request.source_path.exists() {
            return Err(ThumbnailError::InvalidSourcePath(
                request.source_path.display().to_string(),
            ));
        }

        // Load original image
        let original_image = load_image_from_path(&request.source_path)
            .await
            .map_err(|e| {
                error!("Failed to load image {:?}: {}", request.source_path, e);
                ThumbnailError::ImageProcessing(e)
            })?;

        let (original_width, original_height) = (original_image.width(), original_image.height());
        let aspect_ratio = if original_height > 0 {
            original_width as f64 / original_height as f64
        } else {
            0.0
        };

        // Resize with letterboxing
        let (resized_image, letterbox_dims) = resize_with_letterbox(
            &original_image,
            request.max_size,
            [40, 40, 40], // Default dark gray background
        )
        .map_err(|e| {
            error!("Failed to resize image {:?}: {}", request.source_path, e);
            ThumbnailError::ImageProcessing(e)
        })?;

        // Generate thumbnail path
        let thumbnail_path = self.generate_thumbnail_path(&request.source_path);

        // Save thumbnail
        resized_image.save(&thumbnail_path).map_err(|e| {
            error!("Failed to save thumbnail {:?}: {}", thumbnail_path, e);
            ThumbnailError::ImageProcessing(ImageProcessingError::LoadError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )))
        })?;

        // Generate accessibility information
        let accessibility_info = if request.accessibility_mode {
            AccessibilityInfo::thumbnail_default()
        } else {
            AccessibilityInfo::default()
        };

        let screen_reader_label = self.generate_screen_reader_label(
            &request.source_path,
            original_width,
            original_height,
        );

        let accessibility_alt_text = self.generate_accessibility_alt_text(
            &request.source_path,
            original_width,
            original_height,
        );

        Ok(GenerateThumbnailResponse {
            thumbnail_path,
            original_width,
            original_height,
            aspect_ratio,
            letterbox_width: letterbox_dims.final_width,
            letterbox_height: letterbox_dims.final_height,
            generation_time_ms: start_time.elapsed().as_millis(),
            screen_reader_label,
            accessibility_alt_text,
            retry_used: request.retry_count,
        })
    }

    fn generate_thumbnail_path(&self, source_path: &PathBuf) -> PathBuf {
        use std::ffi::OsStr;

        // Create a deterministic thumbnail filename based on source path
        let filename = source_path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("unknown");

        let thumbnail_name = format!("thumb_{}.jpg", filename);
        self.thumbnail_dir.join(thumbnail_name)
    }

    fn get_cached_thumbnail_path(&self, source_path: &PathBuf) -> PathBuf {
        self.generate_thumbnail_path(source_path)
    }

    fn generate_screen_reader_label(&self, path: &PathBuf, width: u32, height: u32) -> String {
        let filename = path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("unknown image");

        format!("{} ({}×{})", filename, width, height)
    }

    fn generate_accessibility_alt_text(&self, path: &PathBuf, width: u32, height: u32) -> String {
        let filename = path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("unknown image");

        let aspect_desc = if width > height {
            "landscape"
        } else if height > width {
            "portrait"
        } else {
            "square"
        };

        format!(
            "{} ({} image, {}×{} pixels)",
            filename, aspect_desc, width, height
        )
    }
}

impl Default for ThumbnailService {
    fn default() -> Self {
        let cache = ThumbnailCache::new();
        let thumbnail_dir = std::env::temp_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("cupola_thumbnails");

        // Ensure thumbnail directory exists
        std::fs::create_dir_all(&thumbnail_dir).ok();

        Self::new(cache, thumbnail_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_thumbnail_service_basic() {
        let temp_dir = TempDir::new().unwrap();
        let cache = ThumbnailCache::new();
        let service = ThumbnailService::new(cache, temp_dir.path().to_path_buf());

        let source_path = PathBuf::from("nonexistent.jpg");
        let request = GenerateThumbnailRequest::new(source_path);

        // Should fail for non-existent file
        let result = service.generate_thumbnail(request).await;
        assert!(result.is_err());

        if let Err(ThumbnailError::InvalidSourcePath(path)) = result {
            assert_eq!(path, source_path.display().to_string());
        } else {
            panic!("Expected InvalidSourcePath error");
        }
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let path1 = PathBuf::from("/test/image.jpg");
        let path2 = PathBuf::from("/test/image.jpg");

        let key1 = ThumbnailCache::generate_cache_key(&path1);
        let key2 = ThumbnailCache::generate_cache_key(&path2);

        assert_eq!(key1, key2);
    }
}
