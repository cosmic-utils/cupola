pub mod cache;
pub mod loader;

pub use cache::{CachedImage, ImageCache};
pub use loader::{LoadedImage, load_image, load_thumbnail};

pub fn register_format_hooks() {
    #[cfg(feature = "heif")]
    {
        // libheif-rs automatically registers decoders when imported
    }
}
