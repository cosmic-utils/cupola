pub mod gallery;
pub mod single;

pub use gallery::GalleryView;
pub use single::SingleView;

/// Current view mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Single,
    Gallery,
}
