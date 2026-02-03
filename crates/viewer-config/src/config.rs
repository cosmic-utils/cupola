use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppTheme {
    Dark,
    Light,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThumbnailSize {
    Small = 128,
    Medium = 256,
    Large = 512,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WallpaperBehavior {
    Ask,
    Set,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortMode {
    Name,
    Date,
    Size,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewerConfig {
    pub app_theme: AppTheme,
    pub thumbnail_size: ThumbnailSize,
    pub wallpaper_behavior: WallpaperBehavior,
    pub sort_mode: SortMode,
    pub sort_order: SortOrder,
}

impl Default for ViewerConfig {
    fn default() -> Self {
        Self {
            app_theme: AppTheme::System,
            thumbnail_size: ThumbnailSize::Medium,
            wallpaper_behavior: WallpaperBehavior::Ask,
            sort_mode: SortMode::Name,
            sort_order: SortOrder::Ascending,
        }
    }
}

pub fn config() -> ViewerConfig {
    ViewerConfig::default()
}
