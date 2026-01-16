use cosmic::widget::image::Handle;
use std::{
    fmt::{self, Debug, Formatter},
    path::{Path, PathBuf},
};
use thiserror::Error;

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

pub async fn load_image(path: PathBuf) -> Result<LoadedImage, LoadError> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    rayon::spawn(move || {
        let result = load_image_sync(&path);
        let _ = tx.send(result);
    });

    rx.await.map_err(|_| LoadError::Cancelled)?
}

fn load_image_sync(path: &Path) -> Result<LoadedImage, LoadError> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .unwrap_or_default();

    // Handle HEIC separately if feature is enabled
    #[cfg(feature = "heif")]
    if extension = "heif" || extension == "heif" {
        return load_heif(path);
    }

    if is_zune_supported(&extension) {
        match load_with_zune(path) {
            Ok(img) => return Ok(img),
            Err(_) => {
                return load_with_image(path);
            }
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

fn is_zune_supported(extension: &str) -> bool {
    matches!(
        extension,
        "jpg"
            | "jpeg"
            | "png"
            | "ppm"
            | "pgm"
            | "pbm"
            | "pnm"
            | "bmp"
            | "qoi"
            | "ff"
            | "farbfeld"
            | "hdr"
            | "jxl"
    )
}

fn load_with_zune(path: &Path) -> Result<LoadedImage, LoadError> {
    use zune_image::image::Image;

    let mut img = Image::open(path).map_err(|e| LoadError::UnsupportedFormat(e.to_string()))?;

    img.convert_color(zune_image::codecs::bmp::zune_core::colorspace::ColorSpace::RGBA)
        .map_err(|e| LoadError::UnsupportedFormat(e.to_string()))?;

    let (width, height) = img.dimensions();

    let pixels = img
        .flatten_to_u8()
        .into_iter()
        .next()
        .ok_or_else(|| LoadError::UnsupportedFormat("No pixel data".into()))?;

    let handle = Handle::from_rgba(width as u32, height as u32, pixels);

    Ok(LoadedImage {
        handle,
        width: width as u32,
        height: height as u32,
        path: path.to_path_buf(),
    })
}

fn load_with_image(path: &Path) -> Result<LoadedImage, LoadError> {
    let img = image::open(path)?;
    let rgba = img.into_rgba8();
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

pub async fn load_thumbnail(path: PathBuf, max_size: u32) -> Result<LoadedImage, LoadError> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    rayon::spawn(move || {
        let result = load_thumbnail_sync(&path, max_size);
        let _ = tx.send(result);
    });

    rx.await.map_err(|_| LoadError::Cancelled)?
}

fn load_thumbnail_sync(path: &Path, max_size: u32) -> Result<LoadedImage, LoadError> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .unwrap_or_default();

    if is_zune_supported(&extension) {
        match load_thumbnail_zune(path, max_size) {
            Ok(img) => return Ok(img),
            Err(_) => {
                return load_thumbnail_image(path, max_size);
            }
        }
    }

    load_thumbnail_image(path, max_size)
}

fn load_thumbnail_zune(path: &Path, max_size: u32) -> Result<LoadedImage, LoadError> {
    use zune_image::image::Image;

    let mut img = Image::open(path).map_err(|e| LoadError::UnsupportedFormat(e.to_string()))?;

    img.convert_color(zune_image::codecs::bmp::zune_core::colorspace::ColorSpace::RGBA)
        .map_err(|e| LoadError::UnsupportedFormat(e.to_string()))?;

    let (width, height) = img.dimensions();

    // If already small enough, return directly
    if width <= max_size as usize && height <= max_size as usize {
        let pixels = img
            .flatten_to_u8()
            .into_iter()
            .next()
            .ok_or_else(|| LoadError::UnsupportedFormat("No pixel data".into()))?;

        let handle = Handle::from_rgba(width as u32, height as u32, pixels);

        return Ok(LoadedImage {
            handle,
            width: width as u32,
            height: height as u32,
            path: path.to_path_buf(),
        });
    }

    // Extract pixels for resizing
    let pixels = img
        .flatten_to_u8()
        .into_iter()
        .next()
        .ok_or_else(|| LoadError::UnsupportedFormat("No pixel data".into()))?;

    // Create image buffer and resize
    let rgba_image = image::RgbaImage::from_raw(width as u32, height as u32, pixels)
        .ok_or_else(|| LoadError::UnsupportedFormat("Failed to create image buffer".into()))?;

    let thumbnail = image::imageops::thumbnail(&rgba_image, max_size, max_size);
    let (thumb_width, thumb_height) = thumbnail.dimensions();
    let thumb_pixels = thumbnail.into_raw();

    let handle = Handle::from_rgba(thumb_width, thumb_height, thumb_pixels);

    Ok(LoadedImage {
        handle,
        width: thumb_width,
        height: thumb_height,
        path: path.to_path_buf(),
    })
}

fn load_thumbnail_image(path: &Path, max_size: u32) -> Result<LoadedImage, LoadError> {
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
