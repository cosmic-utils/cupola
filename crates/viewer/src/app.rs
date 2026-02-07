//! Main app state

use crate::{
    fl,
    key_binds::{self, MenuAction},
    menu::menu_bar,
    message::{
        ContextPage, DeleteAction, DragHandle, EditMessage, ImageMessage, Message, NavMessage,
        SettingsMessage, ViewMessage,
    },
    views::{GalleryView, ImageViewState},
    watcher,
};
use ashpd::{
    desktop::wallpaper::{SetOn, WallpaperRequest},
    url::Url,
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
    widget::{
        Id, button, column, dropdown,
        menu::key_bind::{KeyBind, Modifier},
        radio, settings, slider, spin_button, text,
    },
};
use rfd::AsyncFileDialog;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use viewer_config::{AppTheme, ThumbnailSize, ViewerConfig, WallpaperBehavior};
use viewer_image::edit::Transform;
use viewer_image::{self as image, CachedImage, ImageCache, edit::EditState};
use viewer_nav::{self as nav, NavState};

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
    wallpaper_dialog: Option<PathBuf>,
    available_outputs: Vec<String>,
    delete_dialog: Option<PathBuf>,
    edit_state: EditState,
    _save_dialog: Option<PathBuf>,
    thumbnail_load_cursor: usize,
}

impl ImageViewer {
    pub const APP_ID: &'static str = "org.codeberg.bhh32.CosmicViewer";

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

    fn load_current_image(&mut self) -> Task<Action<Message>> {
        if let Some(path) = self.nav.current().cloned() {
            self.load_image(path)
        } else {
            Task::none()
        }
    }

    fn reload_thumbnail(&mut self, path: PathBuf) -> Task<Action<Message>> {
        // Remove from cache to force reload
        self.cache.remove_thumbnail(&path);
        self.cache.clear_pending_thumbnail(&path);

        // Load fresh thumbnail
        if self.cache.is_thumbnail_pending(&path) {
            return Task::none();
        }

        self.cache.set_thumbnail_pending(path.clone());
        let max_size = self.config.thumbnail_size.pixels();

        cosmic::task::future(async move {
            match image::load_thumbnail(path.clone(), max_size).await {
                Ok(img) => Message::Image(ImageMessage::ThumbnailReady {
                    path,
                    handle: img.handle,
                    width: img.width,
                    height: img.height,
                }),
                Err(_) => Message::Image(ImageMessage::LoadFailed {
                    path,
                    error: "Failed to load thumbnail".to_string(),
                }),
            }
        })
    }

    #[allow(dead_code)]
    fn thumbnails_remaining(&self) -> usize {
        self.nav
            .images()
            .iter()
            .filter(|path| {
                self.cache.get_thumbnail(path).is_none() && !self.cache.is_thumbnail_pending(path)
            })
            .count()
    }

    // Load thumbnails in batches to avoid GPU memory exhaustion
    fn load_thumbnails(&mut self) -> Task<Action<Message>> {
        const MAX_PENDING: usize = 8;

        let pending = self.cache.pending_thumbnail_count();
        if pending >= MAX_PENDING {
            return Task::none();
        }

        let slots = MAX_PENDING - pending;
        let thumbnail_size = self.config.thumbnail_size.pixels();
        let images = self.nav.images();
        let total = images.len();
        let mut tasks = Vec::new();

        while tasks.len() < slots && self.thumbnail_load_cursor < total {
            let path = images[self.thumbnail_load_cursor].clone();
            self.thumbnail_load_cursor += 1;

            if self.cache.get_thumbnail(&path).is_some() || self.cache.is_thumbnail_pending(&path) {
                continue;
            }

            self.cache.set_thumbnail_pending(path.clone());

            tasks.push(cosmic::task::future(async move {
                match image::load_thumbnail(path.clone(), thumbnail_size).await {
                    Ok(img) => Message::Image(ImageMessage::ThumbnailReady {
                        path,
                        handle: img.handle,
                        width: img.width,
                        height: img.height,
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

    // Preload adjacent images for smooth navigation (not all images)
    fn preload_images(&mut self) -> Task<Action<Message>> {
        let current_idx = self.nav.index().unwrap_or(0);
        self.preload_images_at(current_idx)
    }

    fn preload_images_at(&mut self, current_idx: usize) -> Task<Action<Message>> {
        const PRELOAD_AHEAD: usize = 2;
        const PRELOAD_BEHIND: usize = 2;

        let images = self.nav.images();
        let total = images.len();
        if total == 0 {
            return Task::none();
        }
        let mut tasks = Vec::new();

        // Calculate range of images to preload (current + adjacent)
        let start = current_idx.saturating_sub(PRELOAD_BEHIND);
        let end = (current_idx + PRELOAD_AHEAD + 1).min(total);

        for img in images.iter().take(end).skip(start) {
            let path = img.clone();

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

    fn update_fit_zoom(&mut self) {
        // Use preview image dimensions if available, otherwise use cached
        if let Some(ref preview) = self.image_state.preview_image {
            self.image_state
                .calculate_fit_zoom(preview.width, preview.height);
        } else if let Some(path) = self.nav.current()
            && let Some(cached) = self.cache.get_full(path)
        {
            self.image_state
                .calculate_fit_zoom(cached.width, cached.height);
        }
    }

    fn scan_and_nav(&mut self, path: PathBuf) -> Task<Action<Message>> {
        let dir = nav::get_image_dir(&path);
        let include_hidden = self.config.show_hidden_files;
        let sort_mode = self.config.sort_mode;
        let sort_order = self.config.sort_order;
        let target = path.clone();

        // Track folder in recent folders
        if let Some(ref folder_dir) = dir
            && let Some(folder_str) = folder_dir.to_str()
        {
            self.config.add_recent_folder(folder_str.to_string());
            if let Some(ref handler) = self.config_handler {
                let _ = self.config.write_entry(handler);
            }
        }

        cosmic::task::future(async move {
            let images = if let Some(dir) = dir {
                nav::scan_dir(&dir, include_hidden, sort_mode, sort_order).await
            } else {
                Vec::new()
            };

            Message::Nav(NavMessage::DirectoryScanned { images, target })
        })
    }

    fn reload_image_list(&mut self) -> Task<Action<Message>> {
        let include_hidden = self.config.show_hidden_files;
        let sort_mode = self.config.sort_mode;
        let sort_order = self.config.sort_order;

        // If an image is selected, use its parent directory
        let dir_option: Option<PathBuf> = if let Some(current) = self.nav.current() {
            nav::get_image_dir(current)
        } else {
            self.config
                .last_dir
                .as_ref()
                .map(|dir_str| PathBuf::from(dir_str.clone()))
        };

        if let Some(dir) = dir_option {
            return cosmic::task::future(async move {
                let images = nav::scan_dir(&dir, include_hidden, sort_mode, sort_order).await;
                Message::Nav(NavMessage::DirectoryRefreshed { images })
            });
        }

        Task::none()
    }

    fn update_title(&mut self) -> Task<Action<Message>> {
        let title = if let Some(path) = self.nav.current()
            && let Some(name) = path.file_name().and_then(|name| name.to_str())
        {
            if self.edit_state.is_modified {
                format!("{} * - {}", name, fl!("app-title"))
            } else {
                format!("{} - {}", name, fl!("app-title"))
            }
        } else {
            fl!("app-title")
        };

        self.set_window_title(title, self.core.main_window_id().unwrap())
    }

    fn reload_with_edits(&self) -> Task<Message> {
        if let Some(original_path) = self.edit_state.original_path.as_ref() {
            let path = original_path.clone();
            let transforms = self.edit_state.transforms.clone();
            let crop = self.edit_state.crop;

            Task::perform(
                async move { viewer_image::edit::apply_edits_to_image(&path, &transforms, crop).await },
                |result| match result {
                    Ok((_, handle, width, height, path)) => {
                        Message::Image(ImageMessage::EditedPreview {
                            path,
                            handle,
                            width,
                            height,
                        })
                    }
                    Err(err) => {
                        Message::OpenError(Arc::new(format!("Failed to apply edits: {}", err)))
                    }
                },
            )
        } else {
            Task::none()
        }
    }

    fn save_edited_image(&mut self, save_path: PathBuf) -> Task<Message> {
        if let Some(original_path) = self.edit_state.original_path.as_ref() {
            let original = original_path.clone();
            let transforms = self.edit_state.transforms.clone();
            let crop = self.edit_state.crop;
            let result_path = save_path.clone();

            Task::perform(
                async move {
                    // Load and apply all edits
                    let result =
                        viewer_image::edit::apply_edits_to_image(&original, &transforms, crop)
                            .await;

                    match result {
                        Ok((img, _, _, _, _)) => {
                            viewer_image::edit::save_image(img, &save_path).await?;
                            Ok(result_path)
                        }
                        Err(err) => Err(err),
                    }
                },
                |result| match result {
                    Ok(path) => Message::Edit(EditMessage::SaveComplete(Ok(path))),
                    Err(err) => Message::Edit(EditMessage::SaveComplete(Err(err.to_string()))),
                },
            )
        } else {
            Task::none()
        }
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
        // Format hooks are registered automatically by viewer_image

        let mut tasks = vec![];

        let (config, config_handler) = match viewer_config::config() {
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
            wallpaper_dialog: None,
            available_outputs: Vec::new(),
            delete_dialog: None,
            edit_state: EditState::new(),
            _save_dialog: None,
            thumbnail_load_cursor: 0,
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
        vec![menu_bar(
            &self.core,
            &self.key_binds,
            self.is_slideshow_active,
            &self.config.recent_folders,
        )]
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let gallery = self.gallery_view.view(
            &self.nav,
            &self.cache,
            self.config.thumbnail_size.pixels(),
            &self.image_state,
            &self.edit_state,
        );

        // Overlay crop dialog if active (takes priority over other dialogs)
        if self.edit_state.is_cropping
            && let Some(path) = self.nav.current()
            && let Some(cached) = self.cache.get_full(path)
        {
            let dialog = self.crop_dialog_view(&cached);

            // Backdrop that doesn't close - crop requires explicit Apply/Cancel
            let backdrop = cosmic::widget::mouse_area(
                cosmic::widget::container(cosmic::widget::Space::new(
                    cosmic::iced::Length::Fill,
                    cosmic::iced::Length::Fill,
                ))
                .width(cosmic::iced::Length::Fill)
                .height(cosmic::iced::Length::Fill)
                .class(cosmic::theme::Container::Transparent),
            )
            .on_press(Message::View(ViewMessage::ImageEditEvent)); // No-op, just captures clicks

            return cosmic::iced_widget::stack![gallery, backdrop, dialog].into();
        }

        // Overlay wallpaper dialog if active
        if let Some(path) = &self.wallpaper_dialog {
            let dialog = self.wallpaper_dialog_view(path);

            let backdrop = cosmic::widget::mouse_area(
                cosmic::widget::container(cosmic::widget::Space::new(
                    cosmic::iced::Length::Fill,
                    cosmic::iced::Length::Fill,
                ))
                .width(cosmic::iced::Length::Fill)
                .height(cosmic::iced::Length::Fill)
                .class(cosmic::theme::Container::Transparent),
            )
            .on_press(Message::CloseWallpaperDialog);

            cosmic::iced_widget::stack![gallery, backdrop, dialog].into()
        } else if let Some(path) = &self.delete_dialog {
            let dialog = self.delete_dialog_view(path);

            let backdrop = cosmic::widget::mouse_area(
                cosmic::widget::container(cosmic::widget::Space::new(
                    cosmic::iced::Length::Fill,
                    cosmic::iced::Length::Fill,
                ))
                .width(cosmic::iced::Length::Fill)
                .height(cosmic::iced::Length::Fill)
                .class(cosmic::theme::Container::Transparent),
            )
            .on_press(Message::CloseDeleteDialog);

            cosmic::iced_widget::stack![gallery, backdrop, dialog].into()
        } else {
            gallery
        }
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
                    if self.nav.current() == Some(&path) {
                        self.image_state.calculate_fit_zoom(width, height);
                    }
                    tasks.push(self.update_title());
                }
                ImageMessage::EditedPreview {
                    path: _,
                    handle,
                    width,
                    height,
                } => {
                    // Display the edited preview without caching it
                    self.is_loading = false;
                    // Store in a special preview field, not in the cache
                    self.image_state.preview_image = Some(CachedImage {
                        handle,
                        width,
                        height,
                    });
                    self.image_state.calculate_fit_zoom(width, height);
                    tasks.push(self.update_title());
                }
                ImageMessage::LoadFailed { path, error } => {
                    self.is_loading = false;
                    self.cache.clear_pending(&path);
                    self.cache.clear_pending_thumbnail(&path);
                    tracing::error!("Failed to load {}: {error}", path.display());
                }
                ImageMessage::ThumbnailReady {
                    path,
                    handle,
                    width,
                    height,
                } => {
                    self.cache.insert_thumbnail(
                        path,
                        CachedImage {
                            handle,
                            width,
                            height,
                        },
                    );

                    // Load next batch of thumbnails
                    tasks.push(self.load_thumbnails());
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
                        self.edit_state.reset();
                        self.nav.go_next();
                        self.image_state.zoom_fit(); // Reset to fit mode for new image
                        self.update_fit_zoom();
                        tasks.push(self.load_current_image());
                        tasks.push(self.preload_images());
                        tasks.push(self.update_title().map(Action::from));
                    } else {
                        // Gallery view: move focus right
                        // GalleryGrid handles this internally, but update state for keybindings
                        let total = self.nav.total();
                        if total > 0 {
                            let new_idx = match self.gallery_view.focused_index {
                                Some(idx) if idx + 1 < total => idx + 1,
                                Some(idx) => idx,
                                None => 0,
                            };
                            self.gallery_view.focused_index = Some(new_idx);
                        }
                    }
                }
                NavMessage::Prev => {
                    self.is_slideshow_active = false;
                    if self.nav.is_selected() {
                        self.edit_state.reset();
                        self.nav.go_prev();
                        self.image_state.zoom_fit(); // Reset to fit mode for new image
                        self.update_fit_zoom();
                        tasks.push(self.load_current_image());
                        tasks.push(self.preload_images());
                        tasks.push(self.update_title().map(Action::from));
                    } else {
                        // Gallery view: move focus left
                        // GalleryGrid handles this internally, but update state for keybindings
                        let total = self.nav.total();
                        if total > 0 {
                            let new_idx = match self.gallery_view.focused_index {
                                Some(idx) if idx > 0 => idx - 1,
                                Some(idx) => idx,
                                None => 0,
                            };
                            self.gallery_view.focused_index = Some(new_idx);
                        }
                    }
                }
                NavMessage::First => {
                    self.is_slideshow_active = false;
                    self.edit_state.reset();
                    self.nav.first();
                    self.image_state.zoom_fit(); // Reset to fit mode for new image
                    self.update_fit_zoom();
                    tasks.push(self.load_current_image());
                    tasks.push(self.update_title().map(Action::from));
                }
                NavMessage::Last => {
                    self.is_slideshow_active = false;
                    self.edit_state.reset();
                    self.nav.last();
                    self.image_state.zoom_fit(); // Reset to fit mode for new image
                    self.update_fit_zoom();
                    tasks.push(self.load_current_image());
                    tasks.push(self.update_title().map(Action::from));
                }
                NavMessage::GoTo(idx) => {
                    self.is_slideshow_active = false;
                    self.edit_state.reset();
                    self.nav.go_to(idx);
                    self.image_state.zoom_fit(); // Reset to fit mode for new image
                    self.update_fit_zoom();
                    tasks.push(self.load_current_image());
                    tasks.push(self.update_title().map(Action::from));
                }
                NavMessage::GalleryFocus(idx) => {
                    self.gallery_view.focused_index = Some(idx);
                }
                NavMessage::GallerySelect(idx) => {
                    self.nav.select(idx);
                    self.image_state.zoom_fit();
                    self.update_fit_zoom();
                    tasks.push(self.load_current_image());
                    tasks.push(self.preload_images());
                }
                NavMessage::DirectoryScanned { images, target } => {
                    self.nav.set_images(images, Some(&target));
                    self.thumbnail_load_cursor = 0;
                    self.cache.resize_thumbnails(self.nav.total());
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
                        // Focus first image in gallery - FlexGrid handles scrolling
                        self.gallery_view.focused_index = Some(0);
                    }

                    tasks.push(self.load_thumbnails());
                    if target.is_file() {
                        tasks.push(self.load_current_image());
                        tasks.push(self.preload_images());
                    }
                }
                NavMessage::DirectoryRefreshed { images } => {
                    self.thumbnail_load_cursor = 0;
                    let was_selected = self.nav.is_selected();
                    let prev_path = self.nav.current().cloned();
                    let prev_idx = self.nav.index().unwrap_or(0);

                    // Update image list; clearing the selection
                    self.nav.set_images(images.clone(), None);
                    self.cache.resize_thumbnails(self.nav.total());

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
                    // Clear preview image so it doesn't persist
                    self.image_state.preview_image = None;
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
                    // When modal is open, up arrow = prev image
                    if self.nav.is_selected() {
                        return self.update(Message::Nav(NavMessage::Prev));
                    }

                    // GalleryGrid handles this internally via keyboard events
                    // This is kept for global key bindings when grid isn't focused
                    let total = self.nav.total();
                    if total == 0 {
                        return Task::none();
                    }

                    let cols = self.gallery_view.cols.max(1);
                    let new_idx = match self.gallery_view.focused_index {
                        Some(idx) if idx >= cols => idx - cols,
                        Some(idx) => idx,
                        None => 0,
                    };

                    self.gallery_view.focused_index = Some(new_idx);
                }
                ViewMessage::FocusDown => {
                    // When modal is open, down arrow = next image
                    if self.nav.is_selected() {
                        return self.update(Message::Nav(NavMessage::Next));
                    }

                    // GalleryGrid handles this internally via keyboard events
                    // This is kept for global key bindings when grid isn't focused
                    let total = self.nav.total();
                    if total == 0 {
                        return Task::none();
                    }

                    let cols = self.gallery_view.cols.max(1);
                    let new_idx = match self.gallery_view.focused_index {
                        Some(idx) if idx + cols < total => idx + cols,
                        Some(idx) => idx,
                        None => 0,
                    };

                    self.gallery_view.focused_index = Some(new_idx);
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
                ViewMessage::GalleryScrollTo(offset_y) => {
                    return scrollable::scroll_to(
                        Id::new(GalleryView::SCROLL_ID),
                        scrollable::AbsoluteOffset {
                            x: 0.0,
                            y: offset_y,
                        },
                    );
                }
                ViewMessage::ImageEditEvent => {
                    // TODO: Add the image edit events
                }
            },
            Message::Edit(edit_msg) => match edit_msg {
                EditMessage::Rotate90 => {
                    if let Some(current_path) = self.nav.current() {
                        if !self.edit_state.is_editing() {
                            self.edit_state.start_editing(current_path.clone());
                        }

                        self.edit_state.apply_transform(Transform::Rotate90);

                        tasks.push(self.reload_with_edits().map(Action::from));
                        tasks.push(self.update_title().map(Action::from));
                    }
                }
                EditMessage::Rotate180 => {
                    if let Some(current_path) = self.nav.current() {
                        if !self.edit_state.is_editing() {
                            self.edit_state.start_editing(current_path.clone());
                        }

                        self.edit_state.apply_transform(Transform::Rotate180);

                        tasks.push(self.reload_with_edits().map(Action::from));
                        tasks.push(self.update_title().map(Action::from));
                    }
                }
                EditMessage::FlipHorizontal => {
                    if let Some(current_path) = self.nav.current() {
                        if !self.edit_state.is_editing() {
                            self.edit_state.start_editing(current_path.clone());
                        }

                        self.edit_state.apply_transform(Transform::FlipHorizontal);
                        tasks.push(self.reload_with_edits().map(Action::from));
                        tasks.push(self.update_title().map(Action::from));
                    }
                }
                EditMessage::FlipVertical => {
                    if let Some(current_path) = self.nav.current() {
                        if !self.edit_state.is_editing() {
                            self.edit_state.start_editing(current_path.clone());
                        }

                        self.edit_state.apply_transform(Transform::FlipVertical);
                        tasks.push(self.reload_with_edits().map(Action::from));
                        tasks.push(self.update_title().map(Action::from));
                    }
                }
                EditMessage::Save => {
                    if self.edit_state.is_modified
                        && let Some(path) = self.edit_state.original_path.clone()
                    {
                        // Save and handle reload in SaveComplete
                        tasks.push(self.save_edited_image(path).map(Action::from));
                    }
                }
                EditMessage::SaveAs => {
                    // Allow SaveAs even without modifications - user may want to save in different format
                    if let Some(current_path) = self.nav.current() {
                        // Start editing if not already
                        if !self.edit_state.is_editing() {
                            self.edit_state.start_editing(current_path.clone());
                        }

                        tasks.push(
                            Task::perform(
                                async {
                                    AsyncFileDialog::new()
                                        .set_title("Save Image As")
                                        .save_file()
                                        .await
                                },
                                |result| {
                                    if let Some(file) = result {
                                        let path = file.path().to_path_buf();
                                        Message::Edit(EditMessage::SaveAsPathSelected(path))
                                    } else {
                                        Message::Cancelled
                                    }
                                },
                            )
                            .map(Action::from),
                        );
                    }
                }
                EditMessage::SaveAsPathSelected(path) => {
                    // Actually save the image to the selected path
                    tasks.push(self.save_edited_image(path).map(Action::from));
                }
                EditMessage::SaveComplete(result) => {
                    match result {
                        Ok(path) => {
                            if !path.as_os_str().is_empty() {
                                let path_clone = path.clone();

                                let is_save_as = self
                                    .edit_state
                                    .original_path
                                    .as_ref()
                                    .is_some_and(|p| path != *p);

                                self.cache.remove_full(&path_clone);
                                tasks.push(
                                    self.reload_thumbnail(path_clone.clone()).map(Action::from),
                                );
                                tasks.push(self.load_image(path_clone.clone()));

                                // Refresh gallery if SaveAs created a file in current directory
                                if is_save_as
                                    && let Some(current_path) = self.nav.current()
                                    && let (Some(saved_parent), Some(current_parent)) =
                                        (path_clone.parent(), current_path.parent())
                                    && saved_parent == current_parent
                                {
                                    tasks.push(self.reload_image_list());
                                }
                            }
                            self.edit_state.reset();
                            // Clear the preview image after successful save
                            self.image_state.preview_image = None;
                            tasks.push(self.update_title().map(Action::from));
                        }
                        Err(err) => {
                            tracing::error!("Save failed: {err}");
                        }
                    }
                }
                EditMessage::Undo => {
                    if self.edit_state.undo() {
                        tasks.push(self.reload_with_edits().map(Action::from));
                        tasks.push(self.update_title().map(Action::from));
                    }
                }
                EditMessage::StartCrop => {
                    if let Some(path) = self.nav.current() {
                        if !self.edit_state.is_editing() {
                            self.edit_state.start_editing(path.clone());
                        }
                        self.edit_state.start_crop();
                    }
                }
                EditMessage::CancelCrop => {
                    self.edit_state.cancel_crop();
                    // Clear the preview image so it doesn't persist
                    self.image_state.preview_image = None;
                }
                EditMessage::ApplyCrop => {
                    if let Some(region) = self.edit_state.crop_selection.to_crop_region() {
                        self.edit_state.set_crop(region);
                        self.edit_state.apply_crop();
                        // Regenerate preview with crop applied so user can see result
                        tasks.push(self.reload_with_edits().map(Action::from));
                        tasks.push(self.update_title().map(Action::from));
                    }
                }
                EditMessage::CropDragStart { x, y, handle } => {
                    if handle == DragHandle::None {
                        self.edit_state.crop_selection.start_new_selection(x, y);
                    } else {
                        self.edit_state
                            .crop_selection
                            .start_handle_drag(handle, x, y);
                    }
                }
                EditMessage::CropDragMove { x, y } => {
                    if let Some(cached) = self.nav.current().and_then(|p| self.cache.get_full(p)) {
                        self.edit_state.crop_selection.update_drag(
                            x,
                            y,
                            cached.width as f32,
                            cached.height as f32,
                        );
                    }
                }
                EditMessage::CropDragEnd => {
                    self.edit_state.crop_selection.end_drag();
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
                        self.thumbnail_load_cursor = 0;
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
                    SettingsMessage::WallpaperBehavior(behavior) => {
                        self.config.wallpaper_behavior = behavior
                    }
                    SettingsMessage::SortMode(mode) => {
                        self.config.sort_mode = mode;
                        // Reload the current directory with the new sort mode
                        tasks.push(self.reload_image_list());
                    }
                    SettingsMessage::SortOrder(order) => {
                        self.config.sort_order = order;
                        // Reload the current directory with the new sort order
                        tasks.push(self.reload_image_list());
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
                    let dialog = AsyncFileDialog::new().set_title(fl!("menu-open"));

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
            Message::OpenRecentFolder(idx) => {
                if let Some(folder) = self.config.recent_folders.get(idx).cloned() {
                    let path = PathBuf::from(folder);
                    if path.exists() {
                        tasks.push(self.scan_and_nav(path));
                    }
                }
            }
            Message::ClearRecentFolders => {
                self.config.recent_folders.clear();
                if let Some(ref handler) = self.config_handler {
                    let _ = self.config.write_entry(handler);
                }
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

                // Update fit_zoom for current image
                // Use preview image dimensions if available, otherwise use cached
                if let Some(ref preview) = self.image_state.preview_image {
                    self.image_state
                        .calculate_fit_zoom(preview.width, preview.height);
                } else if let Some(path) = self.nav.current()
                    && let Some(cached) = self.cache.get_full(path)
                {
                    self.image_state
                        .calculate_fit_zoom(cached.width, cached.height);
                }
            }
            Message::SlideshowTick => {
                if self.is_slideshow_active
                    && !self.nav.is_empty()
                    && let Some(path) = self.nav.go_next().cloned()
                {
                    self.update_fit_zoom();
                    tasks.push(self.load_image(path.clone()));
                }
            }
            Message::SetWallpaper => {
                // Try current selected image (modal view), then focused gallery thumbnail
                let path = self.nav.current().cloned().or_else(|| {
                    self.gallery_view
                        .focused_index
                        .and_then(|idx| self.nav.images().get(idx).cloned())
                });

                if let Some(path) = path {
                    // On COSMIC, check the wallpaper behavior setting
                    if is_cosmic_desktop() {
                        match self.config.wallpaper_behavior {
                            WallpaperBehavior::Ask => {
                                // Fetch available outputs and show dialog
                                self.available_outputs = get_cosmic_outputs();
                                self.wallpaper_dialog = Some(path);
                            }
                            WallpaperBehavior::AllDisplays => {
                                return cosmic::task::future(async move {
                                    let result = set_wallpaper_cosmic_on(&path, None).await;
                                    Message::WallpaperResult(result)
                                });
                            }
                            WallpaperBehavior::CurrentDisplay => {
                                // For "current display", we use the focused output
                                // Since we can't easily detect it, default to first output
                                let outputs = get_cosmic_outputs();
                                let output = outputs.first().cloned();
                                return cosmic::task::future(async move {
                                    let result =
                                        set_wallpaper_cosmic_on(&path, output.as_deref()).await;
                                    Message::WallpaperResult(result)
                                });
                            }
                        }
                    } else {
                        // Non-COSMIC: use XDG portal
                        return cosmic::task::future(async move {
                            let result = set_wallpaper(&path).await;
                            Message::WallpaperResult(result)
                        });
                    }
                }
            }
            Message::ShowWallpaperDialog(path) => {
                self.available_outputs = get_cosmic_outputs();
                self.wallpaper_dialog = Some(path);
            }
            Message::SetWallpaperOn(path, target) => {
                self.wallpaper_dialog = None;
                let output = match target {
                    crate::message::WallpaperTarget::All => None,
                    crate::message::WallpaperTarget::Output(name) => Some(name),
                };
                return cosmic::task::future(async move {
                    let result = set_wallpaper_cosmic_on(&path, output.as_deref()).await;
                    Message::WallpaperResult(result)
                });
            }
            Message::CloseWallpaperDialog => {
                self.wallpaper_dialog = None;
            }
            Message::WallpaperResult(result) => {
                if let Err(err) = result {
                    tracing::error!("Failed to set wallpaper: {}", err);
                }
            }
            Message::DeleteImage => {
                // Get current image path (modal view or focused gallery thumbnail)
                let path = self.nav.current().cloned().or_else(|| {
                    self.gallery_view
                        .focused_index
                        .and_then(|idx| self.nav.images().get(idx).cloned())
                });

                if let Some(path) = path {
                    self.delete_dialog = Some(path);
                }
            }
            Message::ShowDeleteDialog(path) => {
                self.delete_dialog = Some(path);
            }
            Message::ConfirmDelete(path, action) => {
                self.delete_dialog = None;
                return cosmic::task::future(async move {
                    let result = match action {
                        DeleteAction::Trash => trash::delete(&path)
                            .map_err(|e| format!("Failed to move to trash: {}", e)),
                        DeleteAction::Permanent => std::fs::remove_file(&path)
                            .map_err(|e| format!("Failed to delete file: {}", e)),
                    };
                    Message::DeleteResult(result)
                });
            }
            Message::CloseDeleteDialog => {
                self.delete_dialog = None;
            }
            Message::DeleteResult(result) => {
                if let Err(err) = result {
                    tracing::error!("Delete failed: {}", err);
                }
                // The file watcher will handle updating the gallery
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
            watcher::watch_directory(self.config.last_dir.as_ref().map(PathBuf::from))
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

    fn wallpaper_dialog_view(&self, path: &Path) -> Element<'_, Message> {
        use crate::message::WallpaperTarget;
        use cosmic::iced::Length;
        use cosmic::widget::{Space, container};

        let spacing = cosmic::theme::active().cosmic().spacing;

        // Build buttons for each output option
        let mut button_col = column().spacing(spacing.space_s);

        // "All Displays" button
        let all_btn = button::standard(fl!("wallpaper-all-displays")).on_press(
            Message::SetWallpaperOn(path.to_path_buf(), WallpaperTarget::All),
        );
        button_col = button_col.push(all_btn);

        // Individual output buttons
        for output in &self.available_outputs {
            let output_btn = button::standard(output.clone()).on_press(Message::SetWallpaperOn(
                path.to_path_buf(),
                WallpaperTarget::Output(output.clone()),
            ));
            button_col = button_col.push(output_btn);
        }

        // Cancel button
        let cancel_btn =
            button::text(fl!("wallpaper-cancel")).on_press(Message::CloseWallpaperDialog);

        let content = column()
            .push(text::title4(fl!("wallpaper-dialog-title")))
            .push(Space::with_height(Length::Fixed(spacing.space_s as f32)))
            .push(button_col)
            .push(Space::with_height(Length::Fixed(spacing.space_m as f32)))
            .push(cancel_btn)
            .spacing(spacing.space_xxs)
            .align_x(cosmic::iced::Alignment::Center);

        let dialog_container = container(content)
            .padding(spacing.space_m)
            .class(cosmic::theme::Container::Dialog);

        // Center the dialog on screen
        container(
            container(dialog_container)
                .width(Length::Shrink)
                .height(Length::Shrink),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(cosmic::iced::alignment::Horizontal::Center)
        .align_y(cosmic::iced::alignment::Vertical::Center)
        .into()
    }

    fn delete_dialog_view(&self, path: &Path) -> Element<'_, Message> {
        use cosmic::iced::Length;
        use cosmic::widget::{Space, container};

        let spacing = cosmic::theme::active().cosmic().spacing;

        // Get filename for display
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        // Trash button
        let trash_btn = button::suggested(fl!("delete-trash")).on_press(Message::ConfirmDelete(
            path.to_path_buf(),
            DeleteAction::Trash,
        ));

        // Delete permanently button
        let delete_btn = button::destructive(fl!("delete-permanent")).on_press(
            Message::ConfirmDelete(path.to_path_buf(), DeleteAction::Permanent),
        );

        // Cancel button
        let cancel_btn = button::text(fl!("delete-cancel")).on_press(Message::CloseDeleteDialog);

        let button_row = cosmic::widget::row()
            .push(trash_btn)
            .push(delete_btn)
            .spacing(spacing.space_s);

        let content = column()
            .push(text::title4(fl!("delete-dialog-title")))
            .push(Space::with_height(Length::Fixed(spacing.space_xs as f32)))
            .push(text::body(filename))
            .push(Space::with_height(Length::Fixed(spacing.space_m as f32)))
            .push(button_row)
            .push(Space::with_height(Length::Fixed(spacing.space_s as f32)))
            .push(cancel_btn)
            .spacing(spacing.space_xxs)
            .align_x(cosmic::iced::Alignment::Center);

        let dialog_container = container(content)
            .padding(spacing.space_m)
            .class(cosmic::theme::Container::Dialog);

        // Center the dialog on screen
        container(
            container(dialog_container)
                .width(Length::Shrink)
                .height(Length::Shrink),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(cosmic::iced::alignment::Horizontal::Center)
        .align_y(cosmic::iced::alignment::Vertical::Center)
        .into()
    }

    fn crop_dialog_view(&self, cached: &viewer_image::CachedImage) -> Element<'_, Message> {
        use crate::widgets::crop_widget;
        use cosmic::iced::Length;
        use cosmic::widget::icon;

        let spacing = cosmic::theme::active().cosmic().spacing;

        // Header with close button
        let close_btn = button::icon(icon::from_name("window-close-symbolic"))
            .on_press(Message::Edit(EditMessage::CancelCrop))
            .padding(spacing.space_xs)
            .class(cosmic::theme::Button::Destructive);

        let header = cosmic::widget::row()
            .push(cosmic::widget::horizontal_space())
            .push(close_btn)
            .width(Length::Fill)
            .padding(spacing.space_xs);

        // Use preview image if available (contains applied edits), otherwise use cached
        let (handle, width, height) = if let Some(ref preview) = self.image_state.preview_image {
            (preview.handle.clone(), preview.width, preview.height)
        } else {
            (cached.handle.clone(), cached.width, cached.height)
        };

        // Self-contained crop widget that handles image rendering and all crop UI
        let crop = crop_widget(handle, width, height, &self.edit_state.crop_selection);

        // Footer with Apply/Cancel buttons
        let cancel_btn =
            button::standard(fl!("crop-cancel")).on_press(Message::Edit(EditMessage::CancelCrop));

        let apply_btn = if self.edit_state.crop_selection.has_selection() {
            button::suggested(fl!("crop-apply")).on_press(Message::Edit(EditMessage::ApplyCrop))
        } else {
            button::suggested(fl!("crop-apply"))
        };

        let footer = cosmic::widget::row()
            .push(cosmic::widget::horizontal_space())
            .push(cancel_btn)
            .push(apply_btn)
            .push(cosmic::widget::horizontal_space())
            .spacing(spacing.space_s)
            .width(Length::Fill)
            .padding(spacing.space_xs);

        // Full-screen layout - crop widget fills the middle and handles its own centering
        cosmic::widget::container(
            column()
                .push(header)
                .push(cosmic::Element::from(crop))
                .push(footer)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .class(cosmic::theme::Container::Dialog)
        .into()
    }

    fn settings_page(&self) -> Element<'_, Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;

        let mut sections = vec![
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
        ];

        // Add COSMIC-specific wallpaper settings if on COSMIC desktop
        if is_cosmic_desktop() {
            sections.push(
                settings::section()
                    .title(fl!("settings-wallpaper"))
                    .add(settings::item(
                        fl!("settings-wallpaper-behavior"),
                        dropdown(
                            WallpaperBehavior::ALL
                                .iter()
                                .map(|b| b.to_string())
                                .collect::<Vec<_>>(),
                            WallpaperBehavior::ALL
                                .iter()
                                .position(|b| *b == self.config.wallpaper_behavior),
                            |idx| {
                                Message::Settings(SettingsMessage::WallpaperBehavior(
                                    WallpaperBehavior::ALL[idx],
                                ))
                            },
                        ),
                    ))
                    .into(),
            );
        }

        settings::view_column(sections).into()
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

            // Use preview image dimensions if available, otherwise use cached
            if let Some(ref preview) = self.image_state.preview_image {
                content = content.push(text::body(format!(
                    "Dimensions: {} x {} (edited)",
                    preview.width, preview.height
                )));
            } else if let Some(cached) = self.cache.get_full(path) {
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

async fn set_wallpaper(path: &std::path::Path) -> Result<(), String> {
    // Try XDG portal first (works on GNOME, KDE, and eventually COSMIC)
    let uri = Url::from_file_path(path).map_err(|()| "Invalid file path".to_string())?;

    let portal_result = WallpaperRequest::default()
        .set_on(SetOn::Both)
        .show_preview(true)
        .build_uri(&uri)
        .await;

    match portal_result {
        Ok(_) => Ok(()),
        Err(e) => {
            // Check if we're on COSMIC and the portal isn't available
            let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
            if desktop.to_uppercase().contains("COSMIC") {
                // Fall back to COSMIC-specific method
                set_wallpaper_cosmic(path).await
            } else {
                Err(e.to_string())
            }
        }
    }
}

// Set wallpaper directly via COSMIC's config system
async fn set_wallpaper_cosmic(path: &std::path::Path) -> Result<(), String> {
    use std::io::Write;

    // COSMIC stores the "all" wallpaper config in ~/.config/cosmic/com.system76.CosmicBackground/v1/all
    let config_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("cosmic/com.system76.CosmicBackground/v1");

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let all_path = config_dir.join("all");
    let path_str = path.to_string_lossy();

    // Build the Entry config in RON format
    let content = format!(
        r#"(
    output: "all",
    source: Path("{}"),
    filter_by_theme: false,
    rotation_frequency: 0,
    filter_method: Lanczos,
    scaling_mode: Zoom,
    sampling_method: Alphanumeric,
)
"#,
        path_str
    );

    let mut file = std::fs::File::create(&all_path)
        .map_err(|e| format!("Failed to create config file: {}", e))?;

    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

fn is_cosmic_desktop() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .map(|d| d.to_uppercase().contains("COSMIC"))
        .unwrap_or(false)
}

// Get output names from cosmic-randr (e.g., "eDP-1", "HDMI-A-1")
fn get_cosmic_outputs() -> Vec<String> {
    // Use cosmic-randr to get actual output names
    if let Ok(output) = std::process::Command::new("cosmic-randr")
        .arg("list")
        .output()
        && output.status.success()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Strip ANSI escape codes
        let stripped = strip_ansi_codes(&stdout);
        let outputs: Vec<String> = stripped
            .lines()
            .filter_map(|line| {
                // Lines like "HDMI-A-1 (enabled)" or "eDP-1 (enabled)"
                let line = line.trim();
                if line.contains("(enabled)") || line.contains("(disabled)") {
                    line.split_whitespace().next().map(String::from)
                } else {
                    None
                }
            })
            .collect();
        if !outputs.is_empty() {
            return outputs;
        }
    }

    Vec::new()
}

fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                // Skip until we hit a letter (end of escape sequence)
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

// Set wallpaper on COSMIC for a specific output (or all if None)
async fn set_wallpaper_cosmic_on(
    path: &std::path::Path,
    output: Option<&str>,
) -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("cosmic/com.system76.CosmicBackground/v1");

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let path_str = path.to_string_lossy();

    // Determine file name and output field value
    // For "all": file is "all", output field is "all"
    // For specific output: file is "output.<name>", output field is "<name>"
    let (config_filename, output_field) = match output {
        Some(name) => (format!("output.{}", name), name.to_string()),
        None => ("all".to_string(), "all".to_string()),
    };

    let config_path = config_dir.join(&config_filename);

    // If setting per-output wallpaper, ensure same-on-all is false and update backgrounds list
    if let Some(output_name) = output {
        let same_on_all_path = config_dir.join("same-on-all");
        std::fs::write(&same_on_all_path, "false\n")
            .map_err(|e| format!("Failed to write same-on-all: {}", e))?;

        // Update the backgrounds list to include this output
        update_backgrounds_list(&config_dir, output_name)?;
    }

    // Try to update existing config, or create new one with defaults
    let content = if config_path.exists() {
        // Read existing config and update only the source field
        let existing = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        update_source_in_config(&existing, &path_str)
    } else {
        // Create new config with defaults
        format!(
            r#"(
    output: "{}",
    source: Path("{}"),
    filter_by_theme: false,
    rotation_frequency: 300,
    filter_method: Lanczos,
    scaling_mode: Zoom,
    sampling_method: Alphanumeric,
)
"#,
            output_field, path_str
        )
    };

    std::fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    // Add to COSMIC Settings' custom-images list so it shows up in the wallpaper picker
    add_to_cosmic_settings_custom_images(path)?;

    Ok(())
}

// Add image to COSMIC Settings' custom-images so it shows up in the wallpaper picker
fn add_to_cosmic_settings_custom_images(path: &std::path::Path) -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("cosmic/com.system76.CosmicSettings.Wallpaper/v1");

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let custom_images_path = config_dir.join("custom-images");

    // Read existing custom images
    let mut custom_images: Vec<PathBuf> = std::fs::read_to_string(&custom_images_path)
        .ok()
        .and_then(|content| parse_path_list(&content))
        .unwrap_or_default();

    // Add this path if not already present
    let path_buf = path.to_path_buf();
    if !custom_images.contains(&path_buf) {
        custom_images.push(path_buf);

        // Write back in RON format
        let content = format!(
            "[\n    {},\n]",
            custom_images
                .iter()
                .map(|p| format!("\"{}\"", p.display()))
                .collect::<Vec<_>>()
                .join(",\n    ")
        );
        std::fs::write(&custom_images_path, content)
            .map_err(|e| format!("Failed to write custom-images: {}", e))?;
    }

    Ok(())
}

fn parse_path_list(content: &str) -> Option<Vec<PathBuf>> {
    let trimmed = content.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        Some(
            inner
                .split(',')
                .filter_map(|s| {
                    let s = s.trim().trim_matches('"');
                    if s.is_empty() {
                        None
                    } else {
                        Some(PathBuf::from(s))
                    }
                })
                .collect(),
        )
    } else {
        None
    }
}

fn update_backgrounds_list(config_dir: &std::path::Path, output_name: &str) -> Result<(), String> {
    let backgrounds_path = config_dir.join("backgrounds");
    let mut backgrounds: Vec<String> = std::fs::read_to_string(&backgrounds_path)
        .ok()
        .and_then(|content| parse_backgrounds_list(&content))
        .unwrap_or_default();

    if !backgrounds.contains(&output_name.to_string()) {
        backgrounds.push(output_name.to_string());
        let backgrounds_content = format!(
            "[\n    {},\n]",
            backgrounds
                .iter()
                .map(|s| format!("\"{}\"", s))
                .collect::<Vec<_>>()
                .join(",\n    ")
        );
        std::fs::write(&backgrounds_path, backgrounds_content)
            .map_err(|e| format!("Failed to write backgrounds: {}", e))?;
    }

    Ok(())
}

fn parse_backgrounds_list(content: &str) -> Option<Vec<String>> {
    let trimmed = content.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        Some(
            inner
                .split(',')
                .filter_map(|s| {
                    let s = s.trim().trim_matches('"');
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.to_string())
                    }
                })
                .collect(),
        )
    } else {
        None
    }
}

// Update only the source field, preserving other settings
fn update_source_in_config(existing: &str, new_path: &str) -> String {
    let mut result = String::new();
    let mut skip_until_comma_or_paren = false;

    for line in existing.lines() {
        let trimmed = line.trim();

        if skip_until_comma_or_paren {
            // Skip continuation of multi-line source value
            if trimmed.ends_with(',') || trimmed.ends_with(')') {
                skip_until_comma_or_paren = false;
            }
            continue;
        }

        if trimmed.starts_with("source:") {
            // Replace the source line
            result.push_str(&format!("    source: Path(\"{}\"),\n", new_path));
            // Check if this is a multi-line value
            if !trimmed.ends_with(',') && !trimmed.ends_with(')') {
                skip_until_comma_or_paren = true;
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    // Remove trailing newline if original didn't have one
    if !existing.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    result
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
