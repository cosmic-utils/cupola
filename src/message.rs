use cosmic::widget::image::Handle;
use std::{path::PathBuf, sync::Arc};

/// Menu action type alias (re-exported from key_binds module)
pub use crate::key_binds::MenuAction;

/// Context drawer pages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextPage {
    About,
    Settings,
    ImageInfo,
}

/// Top-level application message type
#[derive(Debug, Clone)]
pub enum Message {
    /// Image loading and processing
    Image(ImageMessage),
    /// Navigation between images
    Nav(NavMessage),
    /// View state changes (zoom, pan, mode)
    View(ViewMessage),
    /// Keyboard binding action
    KeyBind(MenuAction),
    /// Toggle context drawer page
    ToggleContextPage(ContextPage),
    /// Open file dialog
    OpenFileDialog,
    /// Open folder dialog
    OpenFolderDialog,
    Cancelled,
    OpenError(Arc<String>),
    /// Files selected from dialog
    FilesSelected(Vec<PathBuf>),
    /// Open a specific path
    OpenPath(PathBuf),
    /// System theme changed
    SystemThemeChanged,
    /// Config changed
    ConfigChanged,
}

#[derive(Debug, Clone)]
pub enum ImageMessage {
    /// Image is loading
    Loading(PathBuf),
    /// Image finished loading
    Loaded {
        path: PathBuf,
        handle: Handle,
        width: u32,
        height: u32,
    },
    /// Image load failed
    LoadFailed { path: PathBuf, error: String },
    /// Thumbnail ready for gallery
    ThumbnailReady { path: PathBuf, handle: Handle },
    /// Clear the current image
    Clear,
}

#[derive(Debug, Clone)]
pub enum NavMessage {
    /// Nav to next image in directory
    Next,
    /// Nav to prev image in directory
    Prev,
    /// Jump to first image
    First,
    /// Jump to last image
    Last,
    /// Jump to specific index
    GoTo(usize),
    /// Directory scan completed
    DirectoryScanned(Vec<PathBuf>),
    /// Select an image in the gallery
    GallerySelect(usize),
}

#[derive(Debug, Clone)]
pub enum ViewMessage {
    /// Increase zoom level
    ZoomIn,
    /// Decrease zoom level
    ZoomOut,
    /// Reset zoom to 100%
    ZoomReset,
    /// Fit image to window
    ZoomFit,
    /// Set specific zoom level (1.0 = 100%)
    ZoomSet(f32),
    /// Toggle fullscreen mode
    ToggleFullScreen,
    /// Switch to gallery view
    ShowGallery,
    /// Switch to single image view
    ShowSingle,
    /// Pan the image
    Pan { dx: f32, dy: f32 },
}
