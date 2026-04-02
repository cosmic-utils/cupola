use cosmic::iced::alignment::Horizontal;
use cosmic::iced::keyboard::Key;
use cosmic::iced::{Color, Point};
use cosmic::widget::image::Handle;
use std::{path::PathBuf, sync::Arc};
use viewer_config::{AppTheme, SortMode, SortOrder, ThumbnailSize, WallpaperBehavior};
use viewer_tools::annotate::{AnnotateColor, AnnotateTool, CropRatio, PenMode};

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
    Edit(EditMessage),
    Settings(SettingsMessage),
    KeyBind(MenuAction),
    AnnotationShortcut(Key),
    ToggleContextPage(ContextPage),
    OpenFileDialog,
    OpenFolderDialog,
    OpenRecentFolder(usize),
    ClearRecentFolders,
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
    /// Preview of edited image - should not be cached
    EditedPreview {
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
        width: u32,
        height: u32,
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
    GalleryFocus(usize),
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
    GalleryScrollTo(f32),
    ToggleShapePopout,
    ToggleTransformPopout,
    ClosePopouts,
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    AppTheme(AppTheme),
    DefaultZoom(f32),
    FitToWindow(bool),
    SmoothScaling(bool),
    ThumbnailSize(ThumbnailSize),
    ShowHiddenFiles(bool),
    SlideshowInterval(u32),
    CacheSize(usize),
    RememberLastDir(bool),
    WallpaperBehavior(WallpaperBehavior),
    SortMode(SortMode),
    SortOrder(SortOrder),
}

#[derive(Debug, Clone)]
pub enum EditMessage {
    Rotate90,
    Rotate180,
    FlipHorizontal,
    FlipVertical,
    Save,
    SaveAs,
    SaveAsPathSelected(PathBuf),
    SaveComplete(Result<PathBuf, String>),
    Undo,
    Redo,
    SetTool(AnnotateTool),
    SetColor(AnnotateColor),
    SetStrokeWidth(f32),
    ToolStart(Point),
    ToolDrag(Point),
    ToolEnd,
    CommitTool,
    CancelTool,
    SetFontSize(f32),
    SetBold(bool),
    SetItalic(bool),
    SetUnderline(bool),
    SetAlignment(Horizontal),
    TextInput(String),
    SetStrikethrough(bool),
    SetPenMode(PenMode),
    SetOpacity(f32),
    SetFillMode(bool),
    SetCropRatio(CropRatio),
    CancelAnnotation,
    OpenColorPicker,
    CloseColorPicker,
    SetCustomColor(Color),
    PickerHueSat(f32, f32),
    PickerBrightness(f32),
    PickerAlpha(f32),
    PickerHexInput(String),
    ApplyCrop,
}
