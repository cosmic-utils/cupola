#[cfg(test)]
mod tests {
    use shared::cache::ThumbnailCache;

    #[tokio::test]
    async fn test_cache_insert_and_retrieve() {
        let cache = ThumbnailCache::new();
        let key = "test_image_1".to_string();
        let value = vec![1, 2, 3, 4, 5]; // Mock thumbnail data

        // Initially cache should be empty
        assert!(cache.get(&key).await.is_none());

        // Insert data
        cache.put(key.clone(), value.clone()).await;

        // Verify we can retrieve it
        let retrieved = cache.get(&key).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), value);
    }

    #[tokio::test]
    async fn test_cache_remove() {
        let cache = ThumbnailCache::new();
        let key = "test_remove".to_string();
        let value = vec![1, 2, 3];

        // Insert and verify
        cache.put(key.clone(), value.clone()).await;
        assert!(cache.get(&key).await.is_some());

        // Remove and verify it's gone
        let removed = cache.remove(&key).await;
        assert!(removed.is_some());
        assert_eq!(removed.unwrap(), value);
        assert!(cache.get(&key).await.is_none());
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = ThumbnailCache::new();
        let key1 = "test1".to_string();
        let key2 = "test2".to_string();

        // Insert multiple items
        cache.put(key1.clone(), vec![1, 2, 3]).await;
        cache.put(key2.clone(), vec![4, 5, 6]).await;

        assert_eq!(cache.len().await, 2);

        // Clear should remove all items
        cache.clear().await;
        assert_eq!(cache.len().await, 0);
        assert!(cache.get(&key1).await.is_none());
        assert!(cache.get(&key2).await.is_none());
    }
}
