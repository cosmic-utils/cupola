use cosmic::widget::image::Handle;
use std::{path::PathBuf, sync::Arc};

pub use crate::key_binds::MenuAction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WallpaperTarget {
    All,
    Output(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteAction {
    Trash,
    Permanent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextPage {
    About,
    Settings,
    ImageInfo,
}

#[derive(Debug, Clone)]
pub enum Message {
    Image(ImageMessage),
    Nav(NavMessage),
    View(ViewMessage),
    Settings(SettingsMessage),
    KeyBind(MenuAction),
    ToggleContextPage(ContextPage),
    OpenFileDialog,
    OpenFolderDialog,
    Cancelled,
    OpenError(Arc<String>),
    FilesSelected(Vec<PathBuf>),
    OpenPath(PathBuf),
    SystemThemeChanged,
    ConfigChanged,
    WatcherEvent(crate::watcher::WatcherEvent),
    WindowResized { width: f32, height: f32 },
    SlideshowTick,
    SetWallpaper,
    ShowWallpaperDialog(std::path::PathBuf),
    SetWallpaperOn(std::path::PathBuf, WallpaperTarget),
    CloseWallpaperDialog,
    WallpaperResult(Result<(), String>),
    DeleteImage,
    ShowDeleteDialog(std::path::PathBuf),
    ConfirmDelete(std::path::PathBuf, DeleteAction),
    CloseDeleteDialog,
    DeleteResult(Result<(), String>),
    Quit,
    Surface(cosmic::surface::Action),
}

#[derive(Debug, Clone)]
pub enum ImageMessage {
    Loading(PathBuf),
    Loaded {
        path: PathBuf,
        handle: Handle,
        width: u32,
        height: u32,
    },
    LoadFailed {
        path: PathBuf,
        error: String,
    },
    ThumbnailReady {
        path: PathBuf,
        handle: Handle,
    },
    Clear,
}

#[derive(Debug, Clone)]
pub enum NavMessage {
    Next,
    Prev,
    First,
    Last,
    GoTo(usize),
    DirectoryScanned {
        images: Vec<PathBuf>,
        target: PathBuf,
    },
    DirectoryRefreshed {
        images: Vec<PathBuf>,
    },
    GallerySelect(usize),
}

#[derive(Debug, Clone)]
pub enum ViewMessage {
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ZoomFit,
    ToggleFullScreen,
    CloseModal,
    FocusUp,
    FocusDown,
    SelectFocused,
    StartSlideshow,
    StopSlideshow,
    ToggleSlideshow,
    ImageEditEvent,
    GalleryScroll(cosmic::iced::widget::scrollable::Viewport),
    GalleryLayoutChanged {
        cols: usize,
        row_height: f32,
        scroll_request: Option<crate::widgets::ScrollRequest>,
    },
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    AppTheme(crate::config::AppTheme),
    DefaultZoom(f32),
    FitToWindow(bool),
    SmoothScaling(bool),
    ThumbnailSize(crate::config::ThumbnailSize),
    ShowHiddenFiles(bool),
    SlideshowInterval(u32),
    CacheSize(usize),
    RememberLastDir(bool),
    WallpaperBehavior(crate::config::WallpaperBehavior),
}
