//! Main app state

use crate::{
    config::{AppTheme, ThumbnailSize, ViewerConfig},
    fl,
    image::{self, CachedImage, ImageCache},
    key_binds::{self, MenuAction},
    message::{ContextPage, ImageMessage, Message, NavMessage, SettingsMessage, ViewMessage},
    nav::{self, EXTENSIONS, NavState},
    views::{GalleryView, ImageViewState},
    watcher,
};
use cosmic::{
    Action, Application, ApplicationExt, Core, Element, Task,
    app::context_drawer,
    cosmic_config::{Config, CosmicConfigEntry},
    iced::{
        keyboard::{Key, Modifiers},
        window,
    },
    iced_widget::{scrollable, toggler},
    task::future,
    theme,
    widget::{
        Id, button, column, dropdown,
        menu::key_bind::{KeyBind, Modifier},
        radio, settings, slider, spin_button, text,
    },
};
use rfd::AsyncFileDialog;
use std::{collections::HashMap, path::PathBuf, time::Duration};

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
    is_fullscreen: bool,
    is_slideshow_active: bool,
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

    /// Count how many thumbnails still need to be loaded
    fn thumbnails_remaining(&self) -> usize {
        self.nav
            .images()
            .iter()
            .filter(|path| {
                self.cache.get_thumbnail(path).is_none() && !self.cache.is_thumbnail_pending(path)
            })
            .count()
    }

    /// Load the thumbnails for gallery view (chunked to avoid overwhelming the system)
    fn load_thumbnails(&mut self) -> Task<Action<Message>> {
        let thumbnail_size = self.config.thumbnail_size.pixels();
        let mut tasks = Vec::new();

        const BATCH_SIZE: usize = 100;

        for path in self.nav.images().iter().cloned() {
            // Skip if already cached or already loading
            if self.cache.get_thumbnail(&path).is_some() || self.cache.is_thumbnail_pending(&path) {
                continue;
            }

            if tasks.len() >= BATCH_SIZE {
                break;
            }

            // Mark as pending before spawning task
            self.cache.set_thumbnail_pending(path.clone());

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

    /// Preload single view images (currently unused, kept for future smart preloading)
    #[allow(dead_code)]
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

    /// Snap gallery scroll to the row containing the index
    fn snap_to_thumbnail(&self, index: usize) -> Task<Message> {
        let cols = self.gallery_view.cols;
        if cols == 0 {
            return Task::none();
        }

        let Some(viewport) = self.gallery_view.viewport else {
            // Fallback to relative snap if no viewport yet (e.g. initial load)
            let row = index / cols;
            let total = self.nav.total();
            let total_rows = (total + cols - 1) / cols;

            if total_rows <= 1 {
                return Task::none();
            }

            let y = row as f32 / (total_rows - 1) as f32;

            return scrollable::snap_to(
                Id::new(GalleryView::SCROLL_ID),
                scrollable::RelativeOffset { x: 0.0, y },
            );
        };

        let row = index / cols;
        let spacing = theme::active().cosmic().spacing;
        let thumbnail_size = self.config.thumbnail_size.pixels();

        // Gallery layout calculations
        let padding_top = spacing.space_s as f32;
        let row_spacing = spacing.space_xs as f32;
        // Account for button padding (space_xxs) around content
        let cell_padding = spacing.space_xxs as f32 * 2.0;
        let row_height = thumbnail_size as f32 + cell_padding;

        let item_top = padding_top + (row as f32) * (row_height + row_spacing);
        let item_bottom = item_top + row_height;

        let view_top = viewport.absolute_offset().y;
        let view_height = viewport.bounds().height;
        let view_bottom = view_top + view_height;

        if item_top < view_top {
            scrollable::scroll_to(
                Id::new(GalleryView::SCROLL_ID),
                scrollable::AbsoluteOffset {
                    x: 0.0,
                    y: item_top - padding_top,
                },
            )
        } else if item_bottom > view_bottom {
            // Scroll so bottom is visible, adding a small buffer
            let target_y = item_bottom - view_height + row_spacing;
            scrollable::scroll_to(
                Id::new(GalleryView::SCROLL_ID),
                scrollable::AbsoluteOffset {
                    x: 0.0,
                    y: target_y,
                },
            )
        } else {
            Task::none()
        }
    }

    /// Recalculate fit_zoom for the current image
    fn update_fit_zoom(&mut self) {
        if let Some(path) = self.nav.current()
            && let Some(cached) = self.cache.get_full(path)
        {
            self.image_state
                .calculate_fit_zoom(cached.width, cached.height);
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

    /// Reload the image list from the current directory
    fn reload_image_list(&mut self) -> Task<Action<Message>> {
        let include_hidden = self.config.show_hidden_files;

        // If an image is selected, use its parent directory
        let dir_option: Option<PathBuf> = if let Some(current) = self.nav.current() {
            nav::get_image_dir(current)
        } else if let Some(dir_str) = self.config.last_dir.as_ref() {
            Some(PathBuf::from(dir_str.clone()))
        } else {
            None
        };

        if let Some(dir) = dir_option {
            return cosmic::task::future(async move {
                let images = nav::scan_dir(&dir, include_hidden).await;
                Message::Nav(NavMessage::DirectoryRefreshed { images })
            });
        }

        Task::none()
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
            is_fullscreen: false,
            is_slideshow_active: false,
        };

        let startup_path = if let Some(path) = flags {
            Some(path)
        } else if app.config.remember_last_dir {
            app.config.last_dir.as_ref().map(PathBuf::from)
        } else {
            None
        };

        let startup_path = startup_path.or_else(dirs::picture_dir);

        // Apply saved theme on startup
        tasks.push(cosmic::command::set_theme(
            app.config.app_theme.to_cosmic_theme(),
        ));

        tasks.push(app.set_window_title(fl!("app-title"), app.core.main_window_id().unwrap()));
        if let Some(path) = startup_path {
            tasks.push(app.scan_and_nav(path));
        }

        (app, Task::batch(tasks))
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        vec![crate::menu::menu_bar(&self.core, &self.key_binds, self.is_slideshow_active).into()]
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
                    self.cache.clear_pending_thumbnail(&path);
                    tracing::error!("Failed to load {}: {error}", path.display());
                    // Continue loading more thumbnails if there are more to load
                    if self.thumbnails_remaining() > 0 {
                        tasks.push(self.load_thumbnails());
                    }
                }
                ImageMessage::ThumbnailReady { path, handle } => {
                    self.cache.insert_thumbnail(path, handle);
                    // Continue loading more thumbnails if there are more to load
                    if self.thumbnails_remaining() > 0 {
                        tasks.push(self.load_thumbnails());
                    }
                }
                ImageMessage::Clear => {
                    self.nav = NavState::new();
                    self.cache.clear();
                    tasks.push(self.update_title());
                }
            },
            Message::Nav(nav_msg) => match nav_msg {
                NavMessage::Next => {
                    self.is_slideshow_active = false;
                    if self.nav.is_selected() {
                        // Modal open: navigate images
                        self.nav.go_next();
                        self.image_state.zoom_fit(); // Reset to fit mode for new image
                        self.update_fit_zoom();
                        tasks.push(self.load_current_image());
                    } else {
                        // Gallery view: move focus right
                        let total = self.nav.total();
                        if total > 0 {
                            let new_idx = match self.gallery_view.focused_index {
                                Some(idx) if idx + 1 < total => idx + 1,
                                Some(idx) => idx,
                                None => 0,
                            };

                            self.gallery_view.focused_index = Some(new_idx);

                            // Focus the thumbnail button
                            let button_id = Id::new(format!("thumbnail-{new_idx}"));
                            return Task::batch(vec![
                                button::focus(button_id).map(|m: Message| Action::from(m)),
                                self.snap_to_thumbnail(new_idx)
                                    .map(|m: Message| Action::from(m)),
                            ]);
                        }
                    }
                }
                NavMessage::Prev => {
                    self.is_slideshow_active = false;
                    if self.nav.is_selected() {
                        self.nav.go_prev();
                        self.image_state.zoom_fit(); // Reset to fit mode for new image
                        self.update_fit_zoom();
                        tasks.push(self.load_current_image());
                    } else {
                        // Gallery view: move focus left
                        let total = self.nav.total();
                        if total > 0 {
                            let new_idx = match self.gallery_view.focused_index {
                                Some(idx) if idx > 0 => idx - 1,
                                Some(idx) => idx,
                                None => 0,
                            };

                            self.gallery_view.focused_index = Some(new_idx);

                            // Focus the thumbnail button
                            let button_id = Id::new(format!("thumbnail-{new_idx}"));
                            return Task::batch(vec![
                                button::focus(button_id).map(|m: Message| Action::from(m)),
                                self.snap_to_thumbnail(new_idx)
                                    .map(|m: Message| Action::from(m)),
                            ]);
                        }
                    }
                }
                NavMessage::First => {
                    self.is_slideshow_active = false;
                    self.nav.first();
                    self.image_state.zoom_fit(); // Reset to fit mode for new image
                    self.update_fit_zoom();
                    tasks.push(self.load_current_image());
                }
                NavMessage::Last => {
                    self.is_slideshow_active = false;
                    self.nav.last();
                    self.image_state.zoom_fit(); // Reset to fit mode for new image
                    self.update_fit_zoom();
                    tasks.push(self.load_current_image());
                }
                NavMessage::GoTo(idx) => {
                    self.is_slideshow_active = false;
                    self.nav.go_to(idx);
                    self.image_state.zoom_fit(); // Reset to fit mode for new image
                    self.update_fit_zoom();
                    tasks.push(self.load_current_image());
                }
                NavMessage::GallerySelect(idx) => {
                    self.nav.select(idx);
                    self.image_state.zoom_fit();
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

                    // Open modal only if a specific image file was requested
                    if target.is_file() && self.nav.total() > 0 {
                        self.nav.select(self.nav.index().unwrap_or(0));
                    } else if self.nav.total() > 0 {
                        // Focus first image in gallery
                        self.gallery_view.focused_index = Some(0);
                        tasks.push(self.snap_to_thumbnail(0).map(|m: Message| Action::from(m)));
                    }

                    tasks.push(self.load_thumbnails());
                    tasks.push(self.load_current_image());
                    // Don't preload all images upfront - load on demand instead
                }
                NavMessage::DirectoryRefreshed { images } => {
                    let was_selected = self.nav.is_selected();
                    let prev_path = self.nav.current().cloned();
                    let prev_idx = self.nav.index().unwrap_or(0);

                    // Update image list; clearing the selection
                    self.nav.set_images(images.clone(), None);

                    if was_selected {
                        if self.nav.total() > 0 {
                            // Try to restore selection to same image or nearest neighbor
                            let new_idx = if let Some(ref path) = prev_path {
                                // Find the image in the new list
                                images.iter().position(|pos| pos == path)
                            } else {
                                None
                            };

                            let idx = new_idx.unwrap_or_else(|| {
                                // Image was deleted, use prev_idx clamped to valid range
                                prev_idx.min(self.nav.total() - 1)
                            });

                            self.nav.select(idx);

                            // Reset zoom if showing different image
                            if new_idx.is_none() {
                                self.image_state.zoom_fit();
                            }

                            self.update_fit_zoom();
                            tasks.push(self.load_current_image());
                        }
                        // If no images are left, selection stays cleared
                        // and the modal is closed.
                    } else {
                        // Background update: refresh thumbnails
                        tasks.push(self.load_thumbnails());
                    }
                }
            },
            Message::View(view_msg) => match view_msg {
                ViewMessage::ZoomIn => tasks.push(self.image_state.zoom_in().map(Action::from)),
                ViewMessage::ZoomOut => tasks.push(self.image_state.zoom_out().map(Action::from)),
                ViewMessage::ZoomReset => {
                    tasks.push(self.image_state.zoom_reset().map(Action::from))
                }
                ViewMessage::ZoomFit => self.image_state.zoom_fit(),
                ViewMessage::ToggleFullScreen => {
                    self.is_fullscreen = !self.is_fullscreen;

                    let mode = if self.is_fullscreen {
                        window::Mode::Fullscreen
                    } else {
                        window::Mode::Windowed
                    };

                    let window_id = self
                        .core
                        .main_window_id()
                        .expect("Main window ID should be present");
                    return window::change_mode::<Message>(window_id, mode).map(Action::from);
                }
                ViewMessage::CloseModal => {
                    // Close the modal
                    self.nav.deselect();
                    // Reset zoom state
                    if self.image_state.zoom_level != 1.0 {
                        self.image_state.zoom_level = 1.0;
                        self.image_state.fit_to_window = true;
                    }

                    // If a slideshow was playing, stop it
                    if self.is_slideshow_active {
                        self.is_slideshow_active = false;
                    }
                }
                ViewMessage::FocusUp => {
                    let total = self.nav.total();
                    if total == 0 {
                        return Task::none();
                    }

                    let cols = self.gallery_view.cols;
                    let new_idx = match self.gallery_view.focused_index {
                        Some(idx) if idx >= cols => idx - cols,
                        Some(idx) => idx, // Already on the top row
                        None => 0,        // Init to first image
                    };

                    self.gallery_view.focused_index = Some(new_idx);

                    // Focus thumbnail button
                    let button_id = Id::new(format!("thumbnail-{new_idx}"));
                    return Task::batch(vec![
                        button::focus(button_id).map(|m: Message| Action::from(m)),
                        self.snap_to_thumbnail(new_idx)
                            .map(|m: Message| Action::from(m)),
                    ]);
                }
                ViewMessage::FocusDown => {
                    let total = self.nav.total();
                    if total == 0 {
                        return Task::none();
                    }

                    let cols = self.gallery_view.cols;
                    let new_idx = match self.gallery_view.focused_index {
                        Some(idx) if idx + cols < total => idx + cols,
                        Some(idx) => idx, // Already on the bottom row
                        None => 0,        // Init to first image
                    };

                    self.gallery_view.focused_index = Some(new_idx);

                    // Focus thumbnail button
                    let button_id = Id::new(format!("thumbnail-{new_idx}"));
                    return Task::batch(vec![
                        button::focus(button_id).map(|m: Message| Action::from(m)),
                        self.snap_to_thumbnail(new_idx)
                            .map(|m: Message| Action::from(m)),
                    ]);
                }
                ViewMessage::SelectFocused => {
                    if let Some(idx) = self.gallery_view.focused_index {
                        self.nav.select(idx);
                        self.image_state.zoom_fit();
                        self.update_fit_zoom();
                        tasks.push(self.load_current_image());
                    }
                }
                ViewMessage::StartSlideshow => {
                    if self.nav.total() > 0 {
                        self.is_slideshow_active = true;
                        if !self.nav.is_selected()
                            && let Some(path) = self.nav.go_to(0).cloned()
                        {
                            self.update_fit_zoom();
                            tasks.push(self.load_image(path.clone()));
                        }
                    }
                }
                ViewMessage::StopSlideshow => self.is_slideshow_active = false,
                ViewMessage::ToggleSlideshow => {
                    if self.is_slideshow_active {
                        tasks.push(self.update(Message::View(ViewMessage::StopSlideshow)));
                    } else {
                        tasks.push(self.update(Message::View(ViewMessage::StartSlideshow)));
                    }
                }
                ViewMessage::GalleryScroll(viewport) => {
                    self.gallery_view.viewport = Some(viewport);
                }
                ViewMessage::ImageEditEvent => {
                    // TODO: Add the image edit events
                }
            },
            Message::Settings(msg) => {
                match msg {
                    SettingsMessage::AppTheme(theme) => {
                        self.config.app_theme = theme;
                        // Save config and apply theme
                        if let Some(ref handler) = self.config_handler {
                            let _ = self.config.write_entry(handler);
                        }
                        return cosmic::command::set_theme(theme.to_cosmic_theme());
                    }
                    SettingsMessage::DefaultZoom(zoom) => self.config.default_zoom = zoom,
                    SettingsMessage::FitToWindow(fit) => self.config.fit_to_window = fit,
                    SettingsMessage::SmoothScaling(smooth) => self.config.smooth_scaling = smooth,
                    SettingsMessage::ThumbnailSize(size) => {
                        self.config.thumbnail_size = size;
                        // Clear thumbnail cache and for regeneration
                        self.cache.clear_thumbnails();
                        tasks.push(self.load_thumbnails());
                    }
                    SettingsMessage::ShowHiddenFiles(show) => {
                        self.config.show_hidden_files = show;
                        // Reload the current directory with the setting
                        tasks.push(self.reload_image_list());
                    }
                    SettingsMessage::SlideshowInterval(interval) => {
                        self.config.slideshow_interval = interval
                    }
                    SettingsMessage::CacheSize(size) => {
                        self.config.cache_size = size;
                        self.cache.resize(size);
                    }
                    SettingsMessage::RememberLastDir(remem) => {
                        self.config.remember_last_dir = remem
                    }
                }

                // Save config changes
                if let Some(ref handler) = self.config_handler {
                    let _ = self.config.write_entry(handler);
                }
            }
            Message::KeyBind(action) => tasks.push(self.update(action.message())),
            Message::Surface(action) => {
                return cosmic::task::message(Action::Cosmic(cosmic::app::Action::Surface(action)));
            }
            Message::ToggleContextPage(page) => {
                if self.context_page == Some(page) {
                    self.context_page = None;
                } else {
                    self.context_page = Some(page);
                }
            }
            Message::OpenFileDialog => {
                return future(async {
                    let mut dialog = AsyncFileDialog::new()
                        .set_title(fl!("menu-open"))
                        .add_filter("All", &["*"]);

                    for ext in EXTENSIONS {
                        dialog = dialog.add_filter(&format!("*.{ext}"), &[*ext]);
                    }

                    match dialog.pick_file().await {
                        Some(handle) => {
                            let path = handle.path().to_path_buf();
                            Message::FilesSelected(vec![path])
                        }
                        None => Message::Cancelled,
                    }
                });
            }
            Message::OpenFolderDialog => {
                return future(async {
                    let dialog = AsyncFileDialog::new().set_title(fl!("menu-open-folder"));

                    match dialog.pick_folder().await {
                        Some(handle) => {
                            let dir = handle.path().to_path_buf();
                            Message::OpenPath(dir)
                        }
                        None => Message::Cancelled,
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
            Message::WatcherEvent(evt) => {
                tracing::info!("WatcherEvent recieved: {evt:?}");
                match evt {
                    watcher::WatcherEvent::Created(_) => {
                        tasks.push(self.reload_image_list());
                        // TODO: Do this more elegantly at some point
                    }
                    watcher::WatcherEvent::Modified(path) => {
                        // On some systems, external deletion reports as Modified
                        if !path.exists() {
                            self.cache.clear_pending(&path);
                            if self.nav.current() == Some(&path) {
                                self.nav.deselect();
                            }

                            tasks.push(self.reload_image_list());
                        }
                    }
                    watcher::WatcherEvent::Removed(path) => {
                        self.cache.clear_pending(&path);
                        // If the deleted image is the one in the modal, deselect it
                        // so reload_image_list falls back to last_dir
                        if self.nav.current() == Some(&path) {
                            self.nav.deselect();
                        }

                        // Let DirectoryRefresh handle modal transition
                        tasks.push(self.reload_image_list());
                    }
                    watcher::WatcherEvent::Error(err) => tracing::warn!("watcher error: {err}"),
                }
            }
            Message::WindowResized { width, height } => {
                self.image_state.set_window_size(width, height);

                // Update gallery column count for keyboard nav
                let spacing = theme::active().cosmic().spacing;
                let thumbnail_size = self.config.thumbnail_size.pixels();
                let padding = (spacing.space_xxs * 2) as u32; // button padding (left + right)
                let col_spacing = spacing.space_xs as u32;
                let cell_width = thumbnail_size + padding;
                let available = (width as u32).saturating_sub(padding);
                self.gallery_view.cols =
                    ((available + col_spacing) / (cell_width + col_spacing)).max(1) as usize;

                // Update fit_zoom for current image
                if let Some(path) = self.nav.current()
                    && let Some(cached) = self.cache.get_full(path)
                {
                    self.image_state
                        .calculate_fit_zoom(cached.width, cached.height);
                }
            }
            Message::SlideshowTick => {
                if self.is_slideshow_active && !self.nav.is_empty() {
                    if let Some(path) = self.nav.go_next().cloned() {
                        self.update_fit_zoom();
                        tasks.push(self.load_image(path.clone()));
                    }
                }
            }
            Message::Quit => {
                std::process::exit(0);
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
        // Setup the subscription to watch the current directory
        let watcher_sub =
            watcher::watch_directory(self.config.last_dir.as_ref().map(|dir| PathBuf::from(dir)))
                .map(Message::WatcherEvent);

        // Slideshow timer
        let slideshow_sub = if self.is_slideshow_active {
            cosmic::iced::time::every(Duration::from_secs(self.config.slideshow_interval as u64))
                .map(|_| Message::SlideshowTick)
        } else {
            cosmic::iced::Subscription::none()
        };

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
            watcher_sub,
            slideshow_sub,
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
        let spacing = cosmic::theme::active().cosmic().spacing;

        settings::view_column(vec![
            // Appearance section
            settings::section()
                .title(fl!("settings-appearance"))
                .add(settings::item(
                    fl!("settings-theme"),
                    dropdown(
                        AppTheme::ALL
                            .iter()
                            .map(|t| t.to_string())
                            .collect::<Vec<_>>(),
                        AppTheme::ALL
                            .iter()
                            .position(|t| *t == self.config.app_theme),
                        |idx| Message::Settings(SettingsMessage::AppTheme(AppTheme::ALL[idx])),
                    ),
                ))
                .into(),
            // View settings section
            settings::section()
                .title(fl!("settings-view"))
                .add(settings::item(
                    fl!("settings-default-zoom"),
                    slider(0.1..=5.0, self.config.default_zoom, |zoom| {
                        Message::Settings(SettingsMessage::DefaultZoom(zoom))
                    })
                    .step(0.1),
                ))
                .add(settings::item(
                    fl!("settings-fit-to-window"),
                    toggler(self.config.fit_to_window)
                        .on_toggle(|fit| Message::Settings(SettingsMessage::FitToWindow(fit))),
                ))
                .add(settings::item(
                    fl!("settings-smooth-scaling"),
                    toggler(self.config.smooth_scaling).on_toggle(|smooth| {
                        Message::Settings(SettingsMessage::SmoothScaling(smooth))
                    }),
                ))
                .into(),
            // Gallery settings section
            settings::section()
                .title(fl!("settings-gallery"))
                .add(settings::item(
                    fl!("settings-thumbnail-size"),
                    column()
                        .push(radio(
                            text::body(fl!("settings-thumbnail-small")),
                            ThumbnailSize::Small,
                            Some(self.config.thumbnail_size),
                            |size| Message::Settings(SettingsMessage::ThumbnailSize(size)),
                        ))
                        .push(radio(
                            text::body(fl!("settings-thumbnail-medium")),
                            ThumbnailSize::Medium,
                            Some(self.config.thumbnail_size),
                            |size| Message::Settings(SettingsMessage::ThumbnailSize(size)),
                        ))
                        .push(radio(
                            text::body(fl!("settings-thumbnail-large")),
                            ThumbnailSize::Large,
                            Some(self.config.thumbnail_size),
                            |size| Message::Settings(SettingsMessage::ThumbnailSize(size)),
                        ))
                        .push(radio(
                            text::body(fl!("settings-thumbnail-xlarge")),
                            ThumbnailSize::XLarge,
                            Some(self.config.thumbnail_size),
                            |size| Message::Settings(SettingsMessage::ThumbnailSize(size)),
                        ))
                        .spacing(spacing.space_xxs),
                ))
                .add(settings::item(
                    fl!("settings-show-hidden"),
                    toggler(self.config.show_hidden_files).on_toggle(|show| {
                        Message::Settings(SettingsMessage::ShowHiddenFiles(show))
                    }),
                ))
                .into(),
            // Slideshow settings section
            settings::section()
                .title(fl!("settings-slideshow"))
                .add(settings::item(
                    fl!("settings-slideshow-interval"),
                    spin_button(
                        format!("{}", self.config.slideshow_interval),
                        fl!("settings-slideshow-interval"),
                        self.config.slideshow_interval,
                        1,
                        1,
                        60,
                        |inter| Message::Settings(SettingsMessage::SlideshowInterval(inter)),
                    ),
                ))
                .into(),
            // Performance section
            settings::section()
                .title(fl!("settings-performance"))
                .add(settings::item(
                    fl!("settings-cache-size"),
                    spin_button(
                        format!("{}", self.config.cache_size),
                        fl!("settings-cache-size"),
                        self.config.cache_size,
                        5,
                        5,
                        100,
                        |size| Message::Settings(SettingsMessage::CacheSize(size)),
                    ),
                ))
                .into(),
            // Directory settings section
            settings::section()
                .title(fl!("settings-directory"))
                .add(settings::item(
                    fl!("settings-remember-dir"),
                    toggler(self.config.remember_last_dir).on_toggle(|remem| {
                        Message::Settings(SettingsMessage::RememberLastDir(remem))
                    }),
                ))
                .into(),
        ])
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
