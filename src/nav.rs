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

/// Tracks current position in the image list
#[derive(Debug, Clone, Default)]
pub struct NavState {
    images: Vec<PathBuf>,
    cur_idx: Option<usize>,
}

impl NavState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the current image path, if any
    pub fn current(&self) -> Option<&PathBuf> {
        self.cur_idx.and_then(|idx| self.images.get(idx))
    }

    /// Get current index
    pub fn index(&self) -> Option<usize> {
        self.cur_idx
    }

    /// Check if an image is selected, causing the modal to be open
    pub fn is_selected(&self) -> bool {
        self.cur_idx.is_some()
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

    /// Set images list, optionally selecting a specific path
    pub fn set_images(&mut self, images: Vec<PathBuf>, select: Option<&Path>) {
        self.images = images;
        // Only set selection if explicitly requested and path exists
        self.cur_idx = select.and_then(|path| self.images.iter().position(|pos| pos == path));
    }

    /// Select an image
    pub fn select(&mut self, idx: usize) -> Option<&PathBuf> {
        if idx < self.images.len() {
            self.cur_idx = Some(idx);
            self.current()
        } else {
            None
        }
    }

    /// Deselect an image
    pub fn deselect(&mut self) {
        self.cur_idx = None;
    }

    /// Nav to next image, wrapping around
    pub fn go_next(&mut self) -> Option<&PathBuf> {
        if self.images.is_empty() {
            return None;
        }

        let current = self.cur_idx.unwrap_or_default();
        self.cur_idx = Some((current + 1) % self.images.len());
        self.current()
    }

    /// Nav to prev image, wrapping around
    pub fn go_prev(&mut self) -> Option<&PathBuf> {
        if self.images.is_empty() {
            return None;
        }

        let current = self.cur_idx.unwrap_or_default();

        self.cur_idx = Some(if self.cur_idx == Some(0) {
            self.images.len() - 1
        } else {
            current - 1
        });

        self.current()
    }

    /// Jump to first image
    pub fn first(&mut self) -> Option<&PathBuf> {
        if self.images.is_empty() {
            return None;
        }

        self.cur_idx = Some(0);
        self.current()
    }

    /// Jump to last image
    pub fn last(&mut self) -> Option<&PathBuf> {
        if self.images.is_empty() {
            return None;
        }

        self.cur_idx = Some(self.images.len() - 1);
        self.current()
    }

    /// Jump to specific index
    pub fn go_to(&mut self, idx: usize) -> Option<&PathBuf> {
        if idx < self.images.len() {
            self.cur_idx = Some(idx);
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
            if !include_hidden
                && let Some(name) = path.file_name().and_then(|name| name.to_str())
                && name.starts_with('.')
            {
                return false;
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
