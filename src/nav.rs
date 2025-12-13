//! Navigation state and directory scanning

use std::{
    fs,
    path::{Path, PathBuf},
};

use tokio::task::spawn_blocking;

/// Supported image file extensions
pub const EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "tiff", "tif", "ico", "avif", "raw", "cr2", "cr3",
    "nef", "arw", "dng", "orf", "rw2",
];

/// Navigation state for browsing images in a directory
#[derive(Debug, Clone, Default)]
pub struct NavState {
    images: Vec<PathBuf>,
    cur_idx: usize,
    cur_dir: Option<PathBuf>,
}

impl NavState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the current image path, if any
    pub fn current(&self) -> Option<&PathBuf> {
        self.images.get(self.cur_idx)
    }

    /// Get current index
    pub fn index(&self) -> usize {
        self.cur_idx
    }

    /// Get total number of images
    pub fn total(&self) -> usize {
        self.images.len()
    }

    /// Check if nav is empty (no images loaded)
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    pub fn images(&self) -> Vec<PathBuf> {
        self.images.clone()
    }

    /// Set images from a directory scan, optionally preserving selection
    pub fn set_images(&mut self, images: Vec<PathBuf>, select: Option<&Path>) {
        self.images = images;
        if let Some(path) = select {
            self.cur_idx = self.images.iter().position(|pos| pos == path).unwrap_or(0);
        } else {
            self.cur_idx = 0;
        }
    }

    /// Nav to next image, wrapping around
    pub fn next(&mut self) -> Option<&PathBuf> {
        if self.images.is_empty() {
            return None;
        }

        self.cur_idx = (self.cur_idx + 1) % self.images.len();
        self.current()
    }

    /// Nav to prev image, wrapping around
    pub fn prev(&mut self) -> Option<&PathBuf> {
        if self.images.is_empty() {
            return None;
        }

        self.cur_idx = if self.cur_idx == 0 {
            self.images.len() - 1
        } else {
            self.cur_idx - 1
        };

        self.current()
    }

    /// Jump to first image
    pub fn first(&mut self) -> Option<&PathBuf> {
        if self.images.is_empty() {
            return None;
        }

        self.cur_idx = 0;
        self.current()
    }

    /// Jump to last image
    pub fn last(&mut self) -> Option<&PathBuf> {
        if self.images.is_empty() {
            return None;
        }

        self.cur_idx = self.images.len() - 1;
        self.current()
    }

    /// Jump to specific index
    pub fn go_to(&mut self, idx: usize) -> Option<&PathBuf> {
        if idx < self.images.len() {
            self.cur_idx = idx;
            self.current()
        } else {
            None
        }
    }
}

/// Get the dir containing an image file
pub fn get_image_dir(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        path.parent().map(|par| par.to_path_buf())
    } else if path.is_dir() {
        Some(path.to_path_buf())
    } else {
        None
    }
}

/// Async scan a dir for image files
pub async fn scan_dir(dir: &Path, include_hidden: bool) -> Vec<PathBuf> {
    let dir = dir.to_path_buf();

    spawn_blocking(move || scan_dir_sync(&dir, include_hidden))
        .await
        .unwrap_or_default()
}

/// Sync dir scanning
fn scan_dir_sync(dir: &Path, include_hidden: bool) -> Vec<PathBuf> {
    let mut images: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            if !include_hidden {
                if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                    if name.starts_with('.') {
                        return false;
                    }
                }
            }
            is_supported_image(path)
        })
        .collect();

    images.sort_by(|a, b| {
        let a_name = a.file_name().and_then(|name| name.to_str()).unwrap_or("");
        let b_name = b.file_name().and_then(|name| name.to_str()).unwrap_or("");
        human_sort(a_name, b_name)
    });

    images
}

/// Check if a path is a supported image format
pub fn is_supported_image(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Human-friendly sorting that handles numbers properly
fn human_sort(a: &str, b: &str) -> std::cmp::Ordering {
    let mut a_chars = a.chars().peekable();
    let mut b_chars = b.chars().peekable();

    loop {
        match (a_chars.peek(), b_chars.peek()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(ac), Some(bc)) => {
                if ac.is_ascii_digit() && bc.is_ascii_digit() {
                    let a_num: String = std::iter::from_fn(|| {
                        a_chars
                            .clone()
                            .peek()
                            .filter(|c| c.is_ascii_digit())
                            .map(|_| a_chars.next().unwrap())
                    })
                    .collect();
                    let b_num: String = std::iter::from_fn(|| {
                        b_chars
                            .clone()
                            .peek()
                            .filter(|c| c.is_ascii_digit())
                            .map(|_| b_chars.next().unwrap())
                    })
                    .collect();

                    let a_val: u64 = a_num.parse().unwrap_or(0);
                    let b_val: u64 = b_num.parse().unwrap_or(0);

                    match a_val.cmp(&b_val) {
                        std::cmp::Ordering::Equal => continue,
                        other => return other,
                    }
                } else {
                    let ac = a_chars.next().unwrap().to_lowercase().next().unwrap();
                    let bc = b_chars.next().unwrap().to_lowercase().next().unwrap();

                    match ac.cmp(&bc) {
                        std::cmp::Ordering::Equal => continue,
                        other => return other,
                    }
                }
            }
        }
    }
}
