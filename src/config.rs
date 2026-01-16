use cosmic::cosmic_config::{self, Config, ConfigGet, ConfigSet, CosmicConfigEntry};
use serde::{Deserialize, Serialize};

pub const CONFIG_VERSION: u64 = 1;
const APP_ID: &str = "org.codeberg.bhh32.CosmicViewer";

/// Thumbnail size presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThumbnailSize {
    Small,
    #[default]
    Medium,
    Large,
    XLarge,
}

impl ThumbnailSize {
    pub fn pixels(self) -> u32 {
        match self {
            ThumbnailSize::Small => 64,
            ThumbnailSize::Medium => 128,
            ThumbnailSize::Large => 192,
            ThumbnailSize::XLarge => 256,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AppTheme {
    #[default]
    System,
    // Dark themes
    Dracula,
    TokyoNight,
    TokyoNightStorm,
    KanagawaWave,
    KanagawaDragon,
    CatppuccinMocha,
    CatppuccinMacchiato,
    CatppuccinFrappe,
    Nord,
    GruvboxDark,
    // Light themes
    TokyoNightLight,
    KanagawaLotus,
    CatppuccinLatte,
    GruvboxLight,
}

impl AppTheme {
    /// Get all available themes for dropdown
    pub const ALL: &'static [Self] = &[
        Self::System,
        Self::Dracula,
        Self::TokyoNight,
        Self::TokyoNightStorm,
        Self::TokyoNightLight,
        Self::KanagawaWave,
        Self::KanagawaDragon,
        Self::KanagawaLotus,
        Self::CatppuccinMocha,
        Self::CatppuccinMacchiato,
        Self::CatppuccinFrappe,
        Self::CatppuccinLatte,
        Self::Nord,
        Self::GruvboxDark,
        Self::GruvboxLight,
    ];

    /// Check if the theme is a light theme
    fn is_light(self) -> bool {
        matches!(
            self,
            AppTheme::TokyoNightLight
                | AppTheme::KanagawaLotus
                | AppTheme::CatppuccinLatte
                | AppTheme::GruvboxLight
        )
    }

    pub fn to_cosmic_theme(self) -> cosmic::Theme {
        use cosmic::cosmic_theme::ThemeBuilder;
        use cosmic::iced_core::theme::Theme as IcedTheme;
        use palette::{Srgb, Srgba};
        use std::sync::Arc;

        match self {
            AppTheme::System => cosmic::theme::system_preference(),
            other => {
                // Map AppTheme to iced's built-in Theme
                let iced_theme = match other {
                    AppTheme::System => unreachable!(),
                    AppTheme::Dracula => IcedTheme::Dracula,
                    AppTheme::TokyoNight => IcedTheme::TokyoNight,
                    AppTheme::TokyoNightStorm => IcedTheme::TokyoNightStorm,
                    AppTheme::TokyoNightLight => IcedTheme::TokyoNightLight,
                    AppTheme::KanagawaWave => IcedTheme::KanagawaWave,
                    AppTheme::KanagawaDragon => IcedTheme::KanagawaDragon,
                    AppTheme::KanagawaLotus => IcedTheme::KanagawaLotus,
                    AppTheme::CatppuccinMocha => IcedTheme::CatppuccinMocha,
                    AppTheme::CatppuccinMacchiato => IcedTheme::CatppuccinMacchiato,
                    AppTheme::CatppuccinFrappe => IcedTheme::CatppuccinFrappe,
                    AppTheme::CatppuccinLatte => IcedTheme::CatppuccinLatte,
                    AppTheme::Nord => IcedTheme::Nord,
                    AppTheme::GruvboxDark => IcedTheme::GruvboxDark,
                    AppTheme::GruvboxLight => IcedTheme::GruvboxLight,
                };

                // Get the palette from iced theme
                let palette = iced_theme.palette();

                // Helper to convert iced Color to palette Srgba
                let to_srgba = |c: cosmic::iced_core::Color| Srgba::new(c.r, c.g, c.b, c.a);
                let to_srgb = |c: cosmic::iced_core::Color| Srgb::new(c.r, c.g, c.b);

                // Use light or dark builder based on theme
                let builder = if other.is_light() {
                    ThemeBuilder::light()
                } else {
                    ThemeBuilder::dark()
                };

                let theme = builder
                    .bg_color(to_srgba(palette.background))
                    .accent(to_srgb(palette.primary))
                    .success(to_srgb(palette.success))
                    .destructive(to_srgb(palette.danger))
                    .build();

                cosmic::Theme::custom(Arc::new(theme))
            }
        }
    }
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppTheme::System => write!(f, "System"),
            AppTheme::Dracula => write!(f, "Dracula"),
            AppTheme::TokyoNight => write!(f, "Tokyo Night"),
            AppTheme::TokyoNightStorm => write!(f, "Tokyo Night Storm"),
            AppTheme::TokyoNightLight => write!(f, "Tokyo Night Light"),
            AppTheme::KanagawaWave => write!(f, "Kanagawa Wave"),
            AppTheme::KanagawaDragon => write!(f, "Kanagawa Dragon"),
            AppTheme::KanagawaLotus => write!(f, "Kanagawa Lotus"),
            AppTheme::CatppuccinMocha => write!(f, "Catppuccin Mocha"),
            AppTheme::CatppuccinMacchiato => write!(f, "Catppuccin Macchiato"),
            AppTheme::CatppuccinFrappe => write!(f, "Catppuccin Frappé"),
            AppTheme::CatppuccinLatte => write!(f, "Catppuccin Latte"),
            AppTheme::Nord => write!(f, "Nord"),
            AppTheme::GruvboxDark => write!(f, "Gruvbox Dark"),
            AppTheme::GruvboxLight => write!(f, "Gruvbox Light"),
        }
    }
}

/// App config
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewerConfig {
    /// Application theme
    pub app_theme: AppTheme,
    /// Default zoom level (1.0 = 100%)
    pub default_zoom: f32,
    /// Whether to fit images to window by default
    pub fit_to_window: bool,
    /// Remember last opened directory
    pub remember_last_dir: bool,
    /// Last opened directory path
    pub last_dir: Option<String>,
    /// Slideshow interval in seconds
    pub slideshow_interval: u32,
    /// Enable smooth image scaling
    pub smooth_scaling: bool,
    /// Thumbnail size for gallery view
    pub thumbnail_size: ThumbnailSize,
    /// Max cache in memory
    pub cache_size: usize,
    /// Show hidden files in file browser
    pub show_hidden_files: bool,
}

impl Default for ViewerConfig {
    fn default() -> Self {
        Self {
            app_theme: AppTheme::default(),
            default_zoom: 1.0,
            fit_to_window: true,
            remember_last_dir: true,
            last_dir: None,
            slideshow_interval: 5,
            smooth_scaling: true,
            thumbnail_size: ThumbnailSize::default(),
            cache_size: 20,
            show_hidden_files: false,
        }
    }
}

impl CosmicConfigEntry for ViewerConfig {
    const VERSION: u64 = CONFIG_VERSION;

    fn write_entry(&self, config: &cosmic_config::Config) -> Result<(), cosmic_config::Error> {
        config.set("app_theme", self.app_theme)?;
        config.set("default_zoom", self.default_zoom)?;
        config.set("fit_to_window", self.fit_to_window)?;
        config.set("remember_last_dir", self.remember_last_dir)?;
        config.set("last_dir", self.last_dir.clone())?;
        config.set("slideshow_interval", self.slideshow_interval)?;
        config.set("smooth_scaling", self.smooth_scaling)?;
        config.set("thumbnail_size", self.thumbnail_size)?;
        config.set("cache_size", self.cache_size)?;
        config.set("show_hidden_files", self.show_hidden_files)?;
        Ok(())
    }

    fn get_entry(
        config: &cosmic_config::Config,
    ) -> Result<Self, (Vec<cosmic_config::Error>, Self)> {
        let mut errors = Vec::new();
        let mut cfg = ViewerConfig::default();

        macro_rules! get_field {
            ($name:literal, $field:ident, $type:ty) => {
                match config.get::<$type>($name) {
                    Ok(val) => cfg.$field = val,
                    Err(e) => errors.push(e),
                }
            };
        }

        get_field!("app_theme", app_theme, AppTheme);
        get_field!("default_zoom", default_zoom, f32);
        get_field!("fit_to_window", fit_to_window, bool);
        get_field!("remember_last_dir", remember_last_dir, bool);
        get_field!("last_dir", last_dir, Option<String>);
        get_field!("slideshow_interval", slideshow_interval, u32);
        get_field!("thumbnail_size", thumbnail_size, ThumbnailSize);
        get_field!("cache_size", cache_size, usize);
        get_field!("show_hidden_files", show_hidden_files, bool);

        if errors.is_empty() {
            Ok(cfg)
        } else {
            Err((errors, cfg))
        }
    }

    fn update_keys<T: AsRef<str>>(
        &mut self,
        config: &cosmic_config::Config,
        changed_keys: &[T],
    ) -> (Vec<cosmic_config::Error>, Vec<&'static str>) {
        let mut errors = Vec::new();
        let mut updated = Vec::new();

        for key in changed_keys {
            match key.as_ref() {
                "default_zoom" => match config.get::<f32>("default_zoom") {
                    Ok(val) => {
                        self.default_zoom = val;
                        updated.push("default_zoom");
                    }
                    Err(e) => errors.push(e),
                },
                "fit_to_window" => match config.get::<bool>("fit_to_window") {
                    Ok(val) => {
                        self.fit_to_window = val;
                        updated.push("fit_to_window");
                    }
                    Err(e) => errors.push(e),
                },
                "show_hidden_files" => match config.get::<bool>("show_hidden_files") {
                    Ok(val) => {
                        self.show_hidden_files = val;
                        updated.push("show_hidden_files");
                    }
                    Err(e) => errors.push(e),
                },
                _ => {}
            }
        }

        (errors, updated)
    }
}

/// Get or create the config handler
pub fn config() -> Result<Config, cosmic_config::Error> {
    Config::new(APP_ID, CONFIG_VERSION)
}
