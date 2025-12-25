//! Image caching

use cosmic::widget::image::Handle;
use lru::LruCache;
use std::{
    collections::HashSet,
    num::NonZeroUsize,
    path::PathBuf,
    sync::{Arc, Mutex},
};

/// Cached image data with dimensions
#[derive(Clone)]
pub struct CachedImage {
    pub handle: Handle,
    pub width: u32,
    pub height: u32,
}

/// Thread-safe image cache with LRU eviction
#[derive(Clone)]
pub struct ImageCache {
    full_images: Arc<Mutex<LruCache<PathBuf, CachedImage>>>,
    thumbnails: Arc<Mutex<LruCache<PathBuf, Handle>>>,
    pending: Arc<Mutex<HashSet<PathBuf>>>,
}

impl ImageCache {
    /// Create a new cache with specified capacity
    pub fn new(full_capacity: usize, thumbnail_capacity: usize) -> Self {
        Self {
            full_images: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(full_capacity.max(1)).unwrap(),
            ))),
            thumbnails: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(thumbnail_capacity.max(1)).unwrap(),
            ))),
            pending: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Create with default capacities
    pub fn with_defaults() -> Self {
        Self::new(20, 100)
    }

    /// Get a full-resolution cached image
    pub fn get_full(&self, path: &PathBuf) -> Option<CachedImage> {
        self.full_images.lock().ok()?.get(path).cloned()
    }

    /// Insert a full-resolution image
    pub fn insert_full(&self, path: PathBuf, image: CachedImage) {
        if let Ok(mut cache) = self.full_images.lock() {
            cache.put(path.clone(), image);
        }

        self.clear_pending(&path);
    }

    /// Get a cached thumbnail
    pub fn get_thumbnail(&self, path: &PathBuf) -> Option<Handle> {
        self.thumbnails.lock().ok()?.get(path).cloned()
    }

    /// Insert a thumbnail
    pub fn insert_thumbnail(&self, path: PathBuf, handle: Handle) {
        if let Ok(mut cache) = self.thumbnails.lock() {
            cache.put(path, handle);
        }
    }

    /// Check if a path is pending load
    pub fn is_pending(&self, path: &PathBuf) -> bool {
        self.pending
            .lock()
            .map(|set| set.contains(path))
            .unwrap_or(false)
    }

    /// Mark a path as pending load
    pub fn set_pending(&self, path: PathBuf) {
        if let Ok(mut set) = self.pending.lock() {
            set.insert(path);
        }
    }

    /// Clear pending status for a path
    pub fn clear_pending(&self, path: &PathBuf) {
        if let Ok(mut set) = self.pending.lock() {
            set.remove(path);
        }
    }

    /// Clear all caches
    pub fn clear(&self) {
        if let Ok(mut cache) = self.full_images.lock() {
            cache.clear();
        }

        if let Ok(mut cache) = self.thumbnails.lock() {
            cache.clear();
        }

        if let Ok(mut set) = self.pending.lock() {
            set.clear();
        }
    }
}
