//! Main app state

use crate::{
    config::ViewerConfig,
    fl,
    image::{self, CachedImage, ImageCache},
    key_binds::{self, MenuAction},
    message::{ContextPage, ImageMessage, Message, NavMessage, ViewMessage},
    nav::{self, EXTENSIONS, NavState},
    views::{GalleryView, ImageViewState},
};
use cosmic::{
    Action, Application, ApplicationExt, Core, Element, Task,
    app::context_drawer,
    cosmic_config::{Config, CosmicConfigEntry},
    dialog::file_chooser::{self, FileFilter, open::Dialog},
    iced::keyboard::{Key, Modifiers},
    task::future,
    widget::{
        column,
        menu::key_bind::{KeyBind, Modifier},
        text,
    },
};
use std::{collections::HashMap, path::PathBuf, sync::Arc};

/// Main app state
pub struct ImageViewer {
    core: Core,
    config: ViewerConfig,
    config_handler: Option<Config>,
    key_binds: HashMap<KeyBind, MenuAction>,
    nav: NavState,
    cache: ImageCache,
    image_state: ImageViewState,
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

    /// Load the thumbnails for gallery view
    fn load_thumbnails(&mut self) -> Task<Action<Message>> {
        let thumbnail_size = self.config.thumbnail_size.pixels();
        let mut tasks = Vec::new();

        for path in self.nav.images().iter().cloned() {
            if self.cache.get_thumbnail(&path).is_some() {
                continue;
            }

            tasks.push(cosmic::task::future(async move {
                match image::load_thumbnail(path.clone(), thumbnail_size).await {
                    Ok(img) => Message::Image(ImageMessage::ThumbnailReady {
                        path,
                        handle: img.handle,
                    }),
                    Err(e) => {
                        tracing::warn!("Thumbnail failed to load: {e}");
                        Message::Image(ImageMessage::LoadFailed {
                            path,
                            error: e.to_string(),
                        })
                    }
                }
            }));
        }

        Task::batch(tasks)
    }

    /// Preload single view images
    fn preload_images(&mut self) -> Task<Action<Message>> {
        let mut tasks = Vec::new();

        for path in self.nav.images().iter().cloned() {
            if self.cache.get_full(&path).is_some() || self.cache.is_pending(&path) {
                continue;
            }

            self.cache.set_pending(path.clone());

            tasks.push(cosmic::task::future(async move {
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
            }));
        }

        Task::batch(tasks)
    }

    /// Recalculate fit_zoom for the current image
    fn update_fit_zoom(&mut self) {
        if let Some(path) = self.nav.current()
            && let Some(cached) = self.cache.get_full(path)
        {
            self.image_state.calculate_fit_zoom(cached.width, cached.height);
        }
    }

    /// Scan directory and navigate to image
    fn scan_and_nav(&mut self, path: PathBuf) -> Task<Action<Message>> {
        let dir = nav::get_image_dir(&path);
        let include_hidden = self.config.show_hidden_files;
        let target = path.clone();

        cosmic::task::future(async move {
            let images = if let Some(dir) = dir {
                nav::scan_dir(&dir, include_hidden).await
            } else {
                Vec::new()
            };

            Message::Nav(NavMessage::DirectoryScanned { images, target })
        })
    }

    /// Update window title based on current image
    fn update_title(&mut self) -> Task<Action<Message>> {
        let title = if let Some(path) = self.nav.current()
            && let Some(name) = path.file_name().and_then(|name| name.to_str())
        {
            format!("{} - {}", name, fl!("app-title"))
        } else {
            fl!("app-title")
        };

        self.set_window_title(title, self.core.main_window_id().unwrap())
    }
}

impl Application for ImageViewer {
    type Executor = cosmic::executor::Default;
    type Flags = Option<PathBuf>;
    type Message = Message;

    const APP_ID: &'static str = Self::APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Action<Self::Message>>) {
        image::register_format_hooks();

        let mut tasks = vec![];

        let (config, config_handler) = match crate::config::config() {
            Ok(handler) => {
                let config = match ViewerConfig::get_entry(&handler) {
                    Ok(c) => c,
                    Err((_, c)) => c,
                };
                (config, Some(handler))
            }
            Err(_) => (ViewerConfig::default(), None),
        };

        let mut app = Self {
            core,
            config,
            config_handler,
            key_binds: key_binds::init_key_binds(),
            nav: NavState::new(),
            cache: ImageCache::with_defaults(),
            image_state: ImageViewState::new(),
            gallery_view: GalleryView::new(),
            context_page: None,
            is_loading: false,
        };

        let startup_path = if let Some(path) = flags {
            Some(path)
        } else if app.config.remember_last_dir {
            app.config.last_dir.as_ref().map(PathBuf::from)
        } else {
            None
        };

        let startup_path = startup_path.or_else(dirs::picture_dir);

        tasks.push(app.set_window_title(fl!("app-title"), app.core.main_window_id().unwrap()));
        if let Some(path) = startup_path {
            tasks.push(app.scan_and_nav(path));
        }

        (app, Task::batch(tasks))
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        vec![crate::menu::menu_bar(&self.key_binds).into()]
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.gallery_view.view(
            &self.nav,
            &self.cache,
            self.config.thumbnail_size.pixels(),
            &self.image_state,
        )
    }

    fn update(&mut self, message: Message) -> Task<Action<Self::Message>> {
        let mut tasks = Vec::new();

        match message {
            Message::Image(img_msg) => match img_msg {
                ImageMessage::Loading(path) => {
                    self.is_loading = true;
                    tasks.push(self.load_image(path));
                }
                ImageMessage::Loaded {
                    path,
                    handle,
                    width,
                    height,
                } => {
                    self.is_loading = false;
                    self.cache.insert_full(
                        path.clone(),
                        CachedImage {
                            handle,
                            width,
                            height,
                        },
                    );
                    // Update fit_zoom if this is the current image
                    if self.nav.current() == Some(&path) {
                        self.image_state.calculate_fit_zoom(width, height);
                    }
                    tasks.push(self.update_title());
                }
                ImageMessage::LoadFailed { path, error } => {
                    self.is_loading = false;
                    self.cache.clear_pending(&path);
                    tracing::error!("Failed to load {}: {error}", path.display());
                }
                ImageMessage::ThumbnailReady { path, handle } => {
                    self.cache.insert_thumbnail(path, handle);
                }
                ImageMessage::Clear => {
                    self.nav = NavState::new();
                    self.cache.clear();
                    tasks.push(self.update_title());
                }
            },
            Message::Nav(nav_msg) => match nav_msg {
                NavMessage::Next => {
                    self.nav.go_next();
                    self.gallery_view.modal_index = Some(self.nav.index());
                    self.image_state.zoom_fit(); // Reset to fit mode for new image
                    self.update_fit_zoom();
                    tasks.push(self.load_current_image());
                }
                NavMessage::Prev => {
                    self.nav.go_prev();
                    self.gallery_view.modal_index = Some(self.nav.index());
                    self.image_state.zoom_fit(); // Reset to fit mode for new image
                    self.update_fit_zoom();
                    tasks.push(self.load_current_image());
                }
                NavMessage::First => {
                    self.nav.first();
                    self.gallery_view.modal_index = Some(self.nav.index());
                    self.image_state.zoom_fit(); // Reset to fit mode for new image
                    self.update_fit_zoom();
                    tasks.push(self.load_current_image());
                }
                NavMessage::Last => {
                    self.nav.last();
                    self.gallery_view.modal_index = Some(self.nav.index());
                    self.image_state.zoom_fit(); // Reset to fit mode for new image
                    self.update_fit_zoom();
                    tasks.push(self.load_current_image());
                }
                NavMessage::GoTo(idx) => {
                    self.nav.go_to(idx);
                    self.gallery_view.modal_index = Some(self.nav.index());
                    self.image_state.zoom_fit(); // Reset to fit mode for new image
                    self.update_fit_zoom();
                    tasks.push(self.load_current_image());
                }
                NavMessage::DirectoryScanned { images, target } => {
                    self.nav.set_images(images, Some(&target));
                    // Save last directory if enabled
                    if self.config.remember_last_dir {
                        // Get the directory
                        let dir = if target.is_file() {
                            target.parent().map(|parent| parent.to_path_buf())
                        } else {
                            Some(target.clone())
                        };

                        if let Some(dir) = dir {
                            self.config.last_dir = Some(dir.to_string_lossy().to_string());
                        }
                    }

                    // Open modal if started with an image
                    if self.nav.total() > 0 {
                        self.gallery_view.open_modal(self.nav.index());
                    }

                    tasks.push(self.load_thumbnails());
                    tasks.push(self.load_current_image());
                    tasks.push(self.preload_images());
                }
                NavMessage::GallerySelect(idx) => {
                    self.gallery_view.open_modal(idx);
                    self.nav.go_to(idx);
                    self.image_state.zoom_fit(); // Reset to fit mode for new image
                    self.update_fit_zoom();
                    tasks.push(self.load_current_image());
                }
            },
            Message::View(view_msg) => match view_msg {
                ViewMessage::ZoomIn => {
                    tasks.push(self.image_state.zoom_in().map(Action::from))
                }
                ViewMessage::ZoomOut => {
                    tasks.push(self.image_state.zoom_out().map(Action::from))
                }
                ViewMessage::ZoomReset => {
                    tasks.push(self.image_state.zoom_reset().map(Action::from))
                }
                ViewMessage::ZoomFit => self.image_state.zoom_fit(),
                ViewMessage::ToggleFullScreen => {
                    tracing::info!("Toggle Fullscreen clicked!");
                }
                ViewMessage::CloseModal => {
                    // Close the modal
                    self.gallery_view.close_modal();
                    // Reset zoom state
                    if self.image_state.zoom_level != 1.0 {
                        self.image_state.zoom_level = 1.0;
                        self.image_state.fit_to_window = true;
                    }
                }
            },
            Message::KeyBind(action) => tasks.push(self.update(action.message())),
            Message::ToggleContextPage(page) => {
                if self.context_page == Some(page) {
                    self.context_page = None;
                } else {
                    self.context_page = Some(page);
                }
            }
            Message::OpenFileDialog => {
                return future(async {
                    let mut dialog = Dialog::new()
                        .title(fl!("menu-open"))
                        .filter(FileFilter::new("All (*.*)").extension("*.*"));

                    for ext in EXTENSIONS {
                        let filter = FileFilter::new(format!("*.{ext}")).extension(ext.to_string());
                        dialog = dialog.filter(filter);
                    }

                    match dialog.open_file().await {
                        Ok(response) => {
                            if let Ok(path) = response.url().to_file_path() {
                                Message::FilesSelected(vec![path])
                            } else {
                                Message::OpenError(Arc::new(
                                    "Failed to open image file".to_string(),
                                ))
                            }
                        }
                        Err(file_chooser::Error::Cancelled) => Message::Cancelled,
                        Err(why) => Message::OpenError(Arc::new(why.to_string())),
                    }
                });
            }
            Message::OpenFolderDialog => {
                return future(async {
                    let dialog = Dialog::new().title(fl!("menu-open-folder"));
                    match dialog.open_folder().await {
                        Ok(response) => {
                            if let Ok(dir) = response.url().to_file_path() {
                                Message::OpenPath(dir)
                            } else {
                                Message::OpenError(Arc::new("Failed to open folder".to_string()))
                            }
                        }
                        Err(file_chooser::Error::Cancelled) => Message::Cancelled,
                        Err(why) => Message::OpenError(Arc::new(why.to_string())),
                    }
                });
            }
            Message::Cancelled => {}
            Message::OpenError(why) => eprintln!("{why}"),
            Message::FilesSelected(paths) => {
                if let Some(path) = paths.first() {
                    tasks.push(self.scan_and_nav(path.clone()));
                }
            }
            Message::OpenPath(path) => tasks.push(self.scan_and_nav(path)),
            Message::SystemThemeChanged => {
                tracing::info!("Theme change requested");
                // TODO: Implement theme changing
            }
            Message::ConfigChanged => {
                if let Some(ref handler) = self.config_handler {
                    match ViewerConfig::get_entry(handler) {
                        Ok(config) => self.config = config,
                        Err((_, config)) => self.config = config,
                    }
                }
            }
            Message::WindowResized { width, height } => {
                self.image_state.set_window_size(width, height);
                // Update fit_zoom for current image
                if let Some(path) = self.nav.current()
                    && let Some(cached) = self.cache.get_full(path)
                {
                    self.image_state.calculate_fit_zoom(cached.width, cached.height);
                }
            }
        }

        if tasks.is_empty() {
            Task::none()
        } else {
            Task::batch(tasks)
        }
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        let page = self.context_page?;
        let content = match page {
            ContextPage::About => self.about_page(),
            ContextPage::Settings => self.settings_page(),
            ContextPage::ImageInfo => self.image_info_page(),
        };

        Some(context_drawer::context_drawer(
            content,
            Message::ToggleContextPage(page),
        ))
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        cosmic::iced::Subscription::batch([
            cosmic::iced::keyboard::on_key_press(key_press_handler),
            cosmic::iced::window::events().map(|(_, event)| {
                if let cosmic::iced::window::Event::Resized(size) = event {
                    Message::WindowResized {
                        width: size.width,
                        height: size.height,
                    }
                } else {
                    Message::Cancelled // Use existing no-op message for other window events
                }
            }),
        ])
    }

    fn on_app_exit(&mut self) -> Option<Self::Message> {
        if let Some(ref handler) = self.config_handler {
            let _ = self.config.write_entry(handler);
        }

        None
    }
}

impl ImageViewer {
    fn about_page(&self) -> Element<'_, Message> {
        column()
            .push(text::title3(fl!("app-title")))
            .push(text::body(fl!("app-description")))
            .push(text::caption("Version 0.1.0"))
            .spacing(cosmic::theme::active().cosmic().spacing.space_s)
            .into()
    }

    fn settings_page(&self) -> Element<'_, Message> {
        column()
            .push(text::title3("Settings"))
            .push(text::body("Settings page coming soon"))
            .spacing(cosmic::theme::active().cosmic().spacing.space_s)
            .into()
    }

    fn image_info_page(&self) -> Element<'_, Message> {
        let mut content = column()
            .push(text::title3("Image Information"))
            .spacing(cosmic::theme::active().cosmic().spacing.space_s);

        if let Some(path) = self.nav.current() {
            if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                content = content.push(text::body(format!("Name: {name}")));
            }

            content = content.push(text::body(format!("Path: {}", path.display())));

            if let Some(cached) = self.cache.get_full(path) {
                content = content.push(text::body(format!(
                    "Dimensions: {} x {}",
                    cached.width, cached.height
                )));
            }
        } else {
            content = content.push(text::body("No image loaded"));
        }

        content.into()
    }
}

fn key_press_handler(key: Key, modifiers: Modifiers) -> Option<Message> {
    let mut mods = Vec::new();

    if modifiers.control() {
        mods.push(Modifier::Ctrl);
    }

    if modifiers.shift() {
        mods.push(Modifier::Shift);
    }

    if modifiers.alt() {
        mods.push(Modifier::Alt);
    }

    if modifiers.logo() {
        mods.push(Modifier::Super);
    }

    let key_bind = KeyBind {
        modifiers: mods,
        key: key.clone(),
    };

    let bindings = key_binds::init_key_binds();
    bindings
        .get(&key_bind)
        .map(|action| Message::KeyBind(*action))
}
