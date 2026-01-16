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
    /// Settings changes
    Settings(SettingsMessage),
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
    /// Watcher to report directory and file changes
    WatcherEvent(crate::watcher::WatcherEvent),
    /// Window was resized
    WindowResized {
        width: f32,
        height: f32,
    },
    /// Slideshow timer tick
    SlideshowTick,
    /// Quit the application
    Quit,
    Surface(cosmic::surface::Action),
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
    DirectoryScanned {
        images: Vec<PathBuf>,
        target: PathBuf,
    },
    /// Refresh the opened directory
    DirectoryRefreshed { images: Vec<PathBuf> },
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
    /// Toggle fullscreen mode
    ToggleFullScreen,
    /// Close Gallery single view modal
    CloseModal,
    /// Gallery - Focus on the thumbnail above currently focused
    FocusUp,
    /// Gallery - Focus on the thumbnail below currently focused
    FocusDown,
    /// Gallery - Open the modal for the currently focused thumbnail
    SelectFocused,
    /// Start slideshow
    StartSlideshow,
    /// Stop slideshow
    StopSlideshow,
    /// Toggle slideshow on/off
    ToggleSlideshow,
    /// Click event for image editing
    ImageEditEvent,
    /// Gallery scroll update
    GalleryScroll(cosmic::iced::widget::scrollable::Viewport),
    /// Let gallery columns know of a resize event
    GalleryColumnsChanged { cols: usize, row_height: f32 },
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    /// Change application theme
    AppTheme(crate::config::AppTheme),
    /// Change default zoom level
    DefaultZoom(f32),
    /// Toggle fit to window
    FitToWindow(bool),
    /// Toggle smooth scaling
    SmoothScaling(bool),
    /// Change thumbnail size
    ThumbnailSize(crate::config::ThumbnailSize),
    /// Toggle show hidden files
    ShowHiddenFiles(bool),
    /// Change slideshow interval
    SlideshowInterval(u32),
    /// Change cache size
    CacheSize(usize),
    /// Toggle remember last directory
    RememberLastDir(bool),
}
