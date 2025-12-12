//! Main app state

use crate::{
    config::ViewerConfig,
    fl,
    image::{self, CachedImage, ImageCache},
    key_binds::{self, MenuAction},
    message::{ContextPage, ImageMessage, Message, NavMessage, ViewMessage},
    nav::{self, NavState},
    views::{GalleryView, SingleView, ViewMode},
};
use cosmic::{
    Action, Application, Core, Element, Task,
    app::context_drawer,
    cosmic_config::{Config, CosmicConfigEntry},
    iced::keyboard::{Key, Modifiers},
    widget::menu::key_bind::KeyBind,
};
use std::{collections::HashMap, path::PathBuf};

/// Main app state
pub struct ImageViewer {
    core: Core,
    config: ViewerConfig,
    config_handler: Option<Config>,
    key_binds: HashMap<KeyBind, MenuAction>,
    nav: NavState,
    cache: ImageCache,
    view_mode: ViewMode,
    single_view: SingleView,
    gallery_view: GalleryView,
    context_page: Option<ContextPage>,
    is_loading: bool,
}

impl ImageViewer {
    pub const APP_ID: &'static str = "org.codeberg.bhh32.CosmicViewer";

    /// Load an image async
    fn load_image(&mut self, path: PathBuf) -> Task<Action<Message>> {
        if self.cache.get_full(&path).is_some() || self.cache.is_pending(&path) {
            return Task::none();
        }

        self.cache.set_pending(path.clone());
        self.is_loading = true;

        cosmic::task::future(async move {
            match image::load_image(path.clone()).await {
                Ok(img) => Message::Image(ImageMessage::Loaded {
                    path,
                    handle: img.handle,
                    width: img.width,
                    height: img.height,
                }),
                Err(e) => Message::Image(ImageMessage::LoadFailed {
                    path,
                    error: e.to_string(),
                }),
            }
        })
    }

    /// Load the current image in nav
    fn load_current_image(&mut self) -> Task<Action<Message>> {
        if let Some(path) = self.nav.current().cloned() {
            self.load_image(path)
        } else {
            Task::none()
        }
    }

    /// Scan directory and navigate to image
    fn scan_and_nav(&mut self, path: PathBuf) -> Task<Action<Message>> {
        let dir = nav::get_image_dir(&path);
        let include_hidden = self.config.show_hidden_files;

        cosmic::task::future(async move {
            let images = if let Some(dir) = dir {
                nav::scan_dir(&dir, include_hidden).await
            } else {
                Vec::new()
            };

            Message::Nav(NavMessage::DirectoryScanned(images))
        })
    }

    /// Update window title based on current image
    fn update_title(&self) -> Task<Action<Message>> {
        let title = if let Some(path) = self.nav.current()
            && let Some(name) = path.file_name().and_then(|name| name.to_str())
        {
            format!("{} - {}", name, fl!("app-title"))
        } else {
            fl!("app-title")
        };

        self.set_window_title(title)
    }
}

impl Application for ImageViewer {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = Self::APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Action<None>>) {
        image::register_format_hooks();

        let (config, config_handler) = match crate::config::config() {
            Ok(handler) => {
                let config = match ViewerConfig::get_entry(&handler) {
                    Ok(c) => c,
                    Err((_, c)) = c,
                };
                (config, Some(handler))
            },
            Err(_) => (ViewerConfig::default(), None),
        };

        let app = Self {
            core,
            config,
            config_handler,
            key_binds: key_binds::init_key_binds(),
            nav: NavState::new(),
            cache: ImageCache::with_defaults(),
            view_mode: ViewMode::Single,
            single_view: SingelView::new(),
            gallery_view: GalleryView::new(),
            context_page: None,
            is_loading: false,
        };

        let task = app.set_window_title(fl!("app-title"));
        (app, task)
    }
}
