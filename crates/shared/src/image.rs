use fast_image_resize::Resizer;
use fast_image_resize::images::Image;
use image::{DynamicImage, RgbaImage};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImageProcessingError {
    #[error("Failed to load image: {0}")]
    LoadError(#[from] image::ImageError),

    #[error("Failed to resize image: {0}")]
    ResizeError(String),

    #[error("Unsupported image format")]
    UnsupportedFormat,

    #[error("Invalid image dimensions")]
    InvalidDimensions,
}

pub type ImageResult<T> = Result<T, ImageProcessingError>;

#[derive(Debug, Clone)]
pub struct AspectRatio {
    pub width: u32,
    pub height: u32,
    pub ratio: f64,
}

impl AspectRatio {
    pub fn new(width: u32, height: u32) -> Self {
        let ratio = width as f64 / height as f64;
        Self {
            width,
            height,
            ratio,
        }
    }

    pub fn is_landscape(&self) -> bool {
        self.ratio > 1.0
    }

    pub fn is_portrait(&self) -> bool {
        self.ratio < 1.0
    }

    pub fn is_square(&self) -> bool {
        (self.ratio - 1.0).abs() < f64::EPSILON
    }
}

#[derive(Debug, Clone)]
pub struct LetterboxDimensions {
    pub final_width: u32,
    pub final_height: u32,
    pub image_width: u32,
    pub image_height: u32,
    pub letterbox_left: u32,
    pub letterbox_top: u32,
    pub letterbox_width: u32,
    pub letterbox_height: u32,
}

impl LetterboxDimensions {
    pub fn calculate(original_width: u32, original_height: u32, max_size: u32) -> Self {
        let aspect_ratio = AspectRatio::new(original_width, original_height);

        let (scaled_width, scaled_height) = if original_width > original_height {
            if original_width > max_size {
                let scale = max_size as f64 / original_width as f64;
                (max_size, (original_height as f64 * scale).round() as u32)
            } else {
                (original_width, original_height)
            }
        } else {
            if original_height > max_size {
                let scale = max_size as f64 / original_height as f64;
                ((original_width as f64 * scale).round() as u32, max_size)
            } else {
                (original_width, original_height)
            }
        };

        let (final_width, final_height, letterbox_left, letterbox_top) =
            if scaled_width != scaled_height {
                match aspect_ratio {
                    ar if ar.is_landscape() => {
                        let top = (max_size as i32 - scaled_height as i32) / 2;
                        (max_size, max_size, 0, top.max(0) as u32)
                    }
                    ar if ar.is_portrait() => {
                        let left = (max_size as i32 - scaled_width as i32) / 2;
                        (max_size, max_size, left.max(0) as u32, 0)
                    }
                    _ => (max_size, max_size, 0, 0),
                }
            } else {
                (max_size, max_size, 0, 0)
            };

        Self {
            final_width,
            final_height,
            image_width: scaled_width,
            image_height: scaled_height,
            letterbox_left,
            letterbox_top,
            letterbox_width: final_width,
            letterbox_height: final_height,
        }
    }
}

pub fn calculate_aspect_ratio(width: u32, height: u32) -> f64 {
    if height == 0 {
        return 0.0;
    }
    width as f64 / height as f64
}

pub fn load_image_from_bytes(data: &[u8]) -> ImageResult<DynamicImage> {
    let img = image::load_from_memory(data)?;
    Ok(img)
}

pub fn load_image_from_path<P: AsRef<Path>>(path: P) -> ImageResult<DynamicImage> {
    let img = image::open(path)?;
    Ok(img)
}

pub fn resize_with_letterbox(
    image: &DynamicImage,
    max_size: u32,
    background_color: [u8; 3],
) -> ImageResult<(RgbaImage, LetterboxDimensions)> {
    let original_width = image.width();
    let original_height = image.height();

    let dims = LetterboxDimensions::calculate(original_width, original_height, max_size);

    // Create the final square image with background color
    let mut final_image = RgbaImage::from_pixel(
        dims.final_width,
        dims.final_height,
        image::Rgba([
            background_color[0],
            background_color[1],
            background_color[2],
            255, // Full opacity
        ]),
    );

    // Convert the source image to RGBA and resize it
    let rgba_image = image.to_rgba8();
    let resized_image =
        if dims.image_width != original_width || dims.image_height != original_height {
            // Use fast_image_resize for better performance
            let src_image = Image::from_vec_u8(
                dims.image_width,
                dims.image_height,
                rgba_image.to_vec(),
                fast_image_resize::PixelType::U8x4,
            )
            .map_err(|e| ImageProcessingError::ResizeError(e.to_string()))?;

            let mut dst_image = Image::new(
                dims.image_width,
                dims.image_height,
                fast_image_resize::PixelType::U8x4,
            );

            let mut resizer = Resizer::new();
            resizer
                .resize(&src_image, &mut dst_image, None)
                .map_err(|e| ImageProcessingError::ResizeError(e.to_string()))?;

            DynamicImage::ImageRgba8(
                RgbaImage::from_raw(dims.image_width, dims.image_height, dst_image.into_vec())
                    .ok_or(ImageProcessingError::InvalidDimensions)?,
            )
        } else {
            DynamicImage::ImageRgba8(rgba_image)
        };

    // Paste the resized image onto the letterboxed background
    image::imageops::overlay(
        &mut final_image,
        &resized_image.to_rgba8(),
        dims.letterbox_left as i64,
        dims.letterbox_top as i64,
    );

    Ok((final_image, dims))
}

pub fn get_supported_formats() -> Vec<&'static str> {
    vec![
        "PNG", "JPEG", "JPG", "GIF", "WebP", "BMP", "TIFF", "TIF", "ICO", "AVIF",
    ]
}

pub fn is_supported_extension(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "tiff" | "tif" | "ico" | "avif"
    )
}

pub fn format_image_size(size_bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size_bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aspect_ratio() {
        let square = AspectRatio::new(100, 100);
        assert!(square.is_square());
        assert_eq!(square.ratio, 1.0);

        let landscape = AspectRatio::new(200, 100);
        assert!(landscape.is_landscape());
        assert_eq!(landscape.ratio, 2.0);

        let portrait = AspectRatio::new(100, 200);
        assert!(portrait.is_portrait());
        assert_eq!(portrait.ratio, 0.5);
    }

    #[test]
    fn test_letterbox_dimensions() {
        // Square image should remain square
        let dims = LetterboxDimensions::calculate(100, 100, 256);
        assert_eq!(dims.final_width, 256);
        assert_eq!(dims.final_height, 256);

        // Landscape image should be letterboxed
        let dims = LetterboxDimensions::calculate(400, 200, 256);
        assert_eq!(dims.final_width, 256);
        assert_eq!(dims.final_height, 256);
        assert_eq!(dims.image_width, 256);
        assert_eq!(dims.image_height, 128);
        assert_eq!(dims.letterbox_top, 64); // (256 - 128) / 2

        // Portrait image should be pillarboxed
        let dims = LetterboxDimensions::calculate(200, 400, 256);
        assert_eq!(dims.final_width, 256);
        assert_eq!(dims.final_height, 256);
        assert_eq!(dims.image_width, 128);
        assert_eq!(dims.image_height, 256);
        assert_eq!(dims.letterbox_left, 64); // (256 - 128) / 2
    }

    #[test]
    fn test_supported_formats() {
        assert!(is_supported_extension("png"));
        assert!(is_supported_extension("JPG"));
        assert!(is_supported_extension("AvIf"));
        assert!(!is_supported_extension("txt"));
        assert!(!is_supported_extension("mp4"));
    }

    #[test]
    fn test_format_image_size() {
        assert_eq!(format_image_size(512), "512.0 B");
        assert_eq!(format_image_size(1536), "1.5 KB");
        assert_eq!(format_image_size(1048576), "1.0 MB");
    }
}
