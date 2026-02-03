use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type CacheKey = String;
pub type CacheValue = Vec<u8>;

const CACHE_SIZE_MB: usize = 100; // 100MB cache limit
const AVG_THUMBNAIL_SIZE: usize = 50 * 1024; // 50KB average thumbnail size
const MAX_CACHE_ENTRIES: usize = (CACHE_SIZE_MB * 1024 * 1024) / AVG_THUMBNAIL_SIZE;

#[derive(Clone, Debug)]
pub struct ThumbnailCache {
    cache: Arc<Mutex<LruCache<CacheKey, CacheValue>>>,
}

impl ThumbnailCache {
    pub fn new() -> Self {
        let cache =
            LruCache::new(NonZeroUsize::new(MAX_CACHE_ENTRIES).expect("Cache size must be > 0"));

        Self {
            cache: Arc::new(Mutex::new(cache)),
        }
    }

    pub async fn get(&self, key: &CacheKey) -> Option<CacheValue> {
        let mut cache = self.cache.lock().await;
        cache.get(key).cloned()
    }

    pub async fn put(&self, key: CacheKey, value: CacheValue) -> Option<CacheValue> {
        let mut cache = self.cache.lock().await;
        cache.put(key, value)
    }

    pub async fn remove(&self, key: &CacheKey) -> Option<CacheValue> {
        let mut cache = self.cache.lock().await;
        cache.pop(key)
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }

    pub async fn len(&self) -> usize {
        let cache = self.cache.lock().await;
        cache.len()
    }

    pub async fn is_empty(&self) -> bool {
        let cache = self.cache.lock().await;
        cache.is_empty()
    }

    pub fn generate_cache_key(file_path: &PathBuf) -> CacheKey {
        use std::time::SystemTime;

        let modified = std::fs::metadata(file_path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let modified_epoch = modified
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        format!("{}:{}", file_path.display(), modified_epoch)
    }

    pub async fn get_memory_usage_bytes(&self) -> usize {
        let cache = self.cache.lock().await;
        cache.len() * AVG_THUMBNAIL_SIZE
    }

    pub async fn get_memory_usage_mb(&self) -> f64 {
        self.get_memory_usage_bytes().await as f64 / (1024.0 * 1024.0)
    }
}

impl Default for ThumbnailCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let cache = ThumbnailCache::new();
        let key = "test_key".to_string();
        let value = vec![1, 2, 3, 4];

        assert!(cache.get(&key).await.is_none());

        let old_value = cache.put(key.clone(), value.clone()).await;
        assert!(old_value.is_none());

        let retrieved = cache.get(&key).await;
        assert_eq!(retrieved, Some(value));

        let removed = cache.remove(&key).await;
        assert_eq!(removed, Some(vec![1, 2, 3, 4]));
        assert!(cache.get(&key).await.is_none());
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = ThumbnailCache::new();

        cache.put("key1".to_string(), vec![1]).await;
        cache.put("key2".to_string(), vec![2]).await;

        assert_eq!(cache.len().await, 2);

        cache.clear().await;

        assert_eq!(cache.len().await, 0);
        assert!(cache.is_empty().await);
    }
}
