use std::{
    fs,
    path::{Path, PathBuf},
};
use viewer_config::{SortMode, SortOrder};

use tokio::task::spawn_blocking;

pub const EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "tiff", "tif", "ico", "avif", "raw", "cr2", "cr3",
    "nef", "arw", "dng", "orf", "rw2",
];

#[derive(Debug, Clone, Default)]
pub struct NavState {
    images: Vec<PathBuf>,
    cur_idx: Option<usize>,
}

impl NavState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current(&self) -> Option<&PathBuf> {
        self.cur_idx.and_then(|idx| self.images.get(idx))
    }

    pub fn index(&self) -> Option<usize> {
        self.cur_idx
    }

    pub fn is_selected(&self) -> bool {
        self.cur_idx.is_some()
    }

    pub fn total(&self) -> usize {
        self.images.len()
    }

    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    pub fn images(&self) -> Vec<PathBuf> {
        self.images.clone()
    }

    pub fn set_images(&mut self, images: Vec<PathBuf>, select: Option<&Path>) {
        self.images = images;
        // Only set selection if explicitly requested and path exists
        self.cur_idx = select.and_then(|path| self.images.iter().position(|pos| pos == path));
    }

    pub fn select(&mut self, idx: usize) -> Option<&PathBuf> {
        if idx < self.images.len() {
            self.cur_idx = Some(idx);
            self.current()
        } else {
            None
        }
    }

    pub fn deselect(&mut self) {
        self.cur_idx = None;
    }

    pub fn go_next(&mut self) -> Option<&PathBuf> {
        if self.images.is_empty() {
            return None;
        }

        let current = self.cur_idx.unwrap_or_default();
        self.cur_idx = Some((current + 1) % self.images.len());
        self.current()
    }

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

    pub fn first(&mut self) -> Option<&PathBuf> {
        if self.images.is_empty() {
            return None;
        }

        self.cur_idx = Some(0);
        self.current()
    }

    pub fn last(&mut self) -> Option<&PathBuf> {
        if self.images.is_empty() {
            return None;
        }

        self.cur_idx = Some(self.images.len() - 1);
        self.current()
    }

    pub fn go_to(&mut self, idx: usize) -> Option<&PathBuf> {
        if idx < self.images.len() {
            self.cur_idx = Some(idx);
            self.current()
        } else {
            None
        }
    }
}

pub fn get_image_dir(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        path.parent().map(|par| par.to_path_buf())
    } else if path.is_dir() {
        Some(path.to_path_buf())
    } else {
        None
    }
}

pub async fn scan_dir(
    dir: &Path,
    include_hidden: bool,
    sort_mode: SortMode,
    sort_order: SortOrder,
) -> Vec<PathBuf> {
    let dir = dir.to_path_buf();

    spawn_blocking(move || scan_dir_sync(&dir, include_hidden, sort_mode, sort_order))
        .await
        .unwrap_or_default()
}

fn scan_dir_sync(
    dir: &Path,
    include_hidden: bool,
    sort_mode: SortMode,
    sort_order: SortOrder,
) -> Vec<PathBuf> {
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
        let ordering = match sort_mode {
            SortMode::Name => {
                let a_name = a.file_name().and_then(|name| name.to_str()).unwrap_or("");
                let b_name = b.file_name().and_then(|name| name.to_str()).unwrap_or("");
                human_sort(a_name, b_name)
            }
            SortMode::Date => {
                let a_time = fs::metadata(a)
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let b_time = fs::metadata(b)
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                a_time.cmp(&b_time)
            }
            SortMode::Size => {
                let a_size = fs::metadata(a).map(|m| m.len()).unwrap_or(0);
                let b_size = fs::metadata(b).map(|m| m.len()).unwrap_or(0);
                a_size.cmp(&b_size)
            }
        };

        match sort_order {
            SortOrder::Ascending => ordering,
            SortOrder::Descending => ordering.reverse(),
        }
    });

    images
}

pub fn is_supported_image(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

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
