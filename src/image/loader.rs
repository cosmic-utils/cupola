//! Async image loading

use cosmic::widget::image::Handle;
use std::{
    fmt::{self, Debug, Formatter},
    path::{Path, PathBuf},
};
use thiserror::Error;
use tokio::task::spawn_blocking;

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to decode image: {0}")]
    Decode(#[from] image::ImageError),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("Task cancelled")]
    Cancelled,
}

/// Result of loading an image
#[derive(Clone)]
pub struct LoadedImage {
    pub handle: Handle,
    pub width: u32,
    pub height: u32,
    pub path: PathBuf,
}

impl Debug for LoadedImage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoadedImage")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("path", &self.path)
            .finish()
    }
}

/// Load an image async and conver to COSMIC Handle
pub async fn load_image(path: PathBuf) -> Result<LoadedImage, LoadError> {
    spawn_blocking(move || load_image_sync(&path))
        .await
        .map_err(|_| LoadError::Cancelled)?
}

/// Sync image loading (runs in blocking thread)
fn load_image_sync(path: &Path) -> Result<LoadedImage, LoadError> {
    // Handle HEIC separately if feature is enabled
    #[cfg(feature = "heif")]
    {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .unwrap_or_default();

        if extension = "heif" || extension == "heif" {
            return load_heif(path);
        }
    }

    // Standard image formats via the 'image' crate
    let img = image::open(path)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let pixels = rgba.into_raw();

    let handle = Handle::from_rgba(width, height, pixels);

    Ok(LoadedImage {
        handle,
        width,
        height,
        path: path.to_path_buf(),
    })
}

/// Load HEIF/HEIC images (requires libheif system library)
#[cfg(feature = "heif")]
fn load_heif(path: &Path) -> Result<LoadedImage, LoadError> {
    use libeif_rs::{ColorSpace, HeifContext, RgbChroma};

    let ctx = HeifContext::read_from_file(path.to_str().unwrap()).map_err(|e| {
        LoadError::Decode(image::ImageError::Decoding(
            image::error::DecodingError::new(image::error::ImageFormatHint::Unknown, e),
        ))
    })?;

    let handle = ctx.primary_image_handle().map_err(|e| {
        LoadError::Decode(image::ImageError::Decoding(
            image::error::DecodingError::new(image::error::ImageFormatHint::Unknown, e),
        ))
    })?;

    let width = image.width();
    let height = image.height();
    let planes = image.planes();
    let interleaved = planes.interleaved.unwrap();
    let pixels = interleaved.data.to_vec();

    let cosmic_handle = Handle::from_rgba(width, height, pixels);

    Ok(LoadedImage {
        handle: cosmic_handle,
        width,
        height,
        path: path.to_path_buf(),
    })
}

/// Generate a thumbnail for an image
pub async fn load_thumbnail(path: PathBuf, max_size: u32) -> Result<LoadedImage, LoadError> {
    spawn_blocking(move || load_thumbnail_sync(&path, max_size))
        .await
        .map_err(|_| LoadError::Cancelled)?
}

fn load_thumbnail_sync(path: &Path, max_size: u32) -> Result<LoadedImage, LoadError> {
    let img = image::open(path)?;
    let thumbnail = img.thumbnail(max_size, max_size);
    let rgba = thumbnail.to_rgba8();
    let (width, height) = rgba.dimensions();
    let pixels = rgba.into_raw();

    let handle = Handle::from_rgba(width, height, pixels);

    Ok(LoadedImage {
        handle,
        width,
        height,
        path: path.to_path_buf(),
    })
}
