pub mod state;

pub use state::{EditState, Transform};

use cosmic::widget::image::Handle;
use image::{DynamicImage, GenericImageView};
use std::path::{Path, PathBuf};
use thiserror::Error;

use viewer_types::CropRegion;

#[derive(Debug, Error)]
pub enum EditError {
    #[error("Failed to load image: {0}")]
    LoadError(#[from] image::ImageError),
    #[error("Failed to save image: {0}")]
    SaveError(String),
    #[error("Invalid crop region")]
    InvalidCrop,
    #[error("No image loaded")]
    NoImage,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn apply_transform(img: &DynamicImage, transform: Transform) -> DynamicImage {
    match transform {
        Transform::Rotate90 => img.rotate90(),
        Transform::Rotate180 => img.rotate180(),
        Transform::FlipHorizontal => img.fliph(),
        Transform::FlipVertical => img.flipv(),
    }
}

pub fn apply_transforms(mut img: DynamicImage, transforms: &[Transform]) -> DynamicImage {
    for transform in transforms {
        img = apply_transform(&img, *transform);
    }

    img
}

pub fn crop_image(img: &DynamicImage, region: CropRegion) -> Result<DynamicImage, EditError> {
    let (width, height) = img.dimensions();

    // Validate crop region
    if region.x as u32 >= width || region.y as u32 >= height {
        return Err(EditError::InvalidCrop);
    }

    let crop_width = (region.width as u32).min(width - region.x as u32);
    let crop_height = (region.height as u32).min(height - region.y as u32);

    if crop_width == 0 || crop_height == 0 {
        return Err(EditError::InvalidCrop);
    }

    let cropped = img.crop_imm(region.x as u32, region.y as u32, crop_width, crop_height);
    Ok(cropped)
}

pub async fn save_image(img: DynamicImage, path: &Path) -> Result<(), EditError> {
    let path = path.to_path_buf();

    tokio::task::spawn_blocking(move || {
        img.save(&path)
            .map_err(|err| EditError::SaveError(err.to_string()))
    })
    .await
    .map_err(|err| EditError::SaveError(err.to_string()))?
}

pub async fn apply_edits_to_image(
    original_path: &Path,
    transforms: &[Transform],
    crop: Option<CropRegion>,
) -> Result<(DynamicImage, Handle, u32, u32, PathBuf), EditError> {
    let path = original_path.to_path_buf();
    let transforms = transforms.to_vec();

    let (tx, rx) = tokio::sync::oneshot::channel();

    rayon::spawn(move || {
        let result = (|| -> Result<_, EditError> {
            // Load image
            let mut img = image::open(&path)?;

            img = apply_transforms(img, &transforms);

            if let Some(crop_region) = crop {
                img = crop_image(&img, crop_region)?;
            }

            // Convert to rgba for COSMIC handle
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            let pixels = rgba.into_raw();
            let handle = Handle::from_rgba(width, height, pixels);

            Ok((img, handle, width, height, path.clone()))
        })();

        let _ = tx.send(result);
    });

    rx.await
        .map_err(|_| EditError::SaveError("Task cancelled".into()))?
}
