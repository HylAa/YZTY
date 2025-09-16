use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct CacheItem {
    value: String,
    expires_at: Instant,
}

impl CacheItem {
    fn new(value: String, ttl_seconds: u64) -> Self {
        Self {
            value,
            expires_at: Instant::now() + Duration::from_secs(ttl_seconds),
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }

    fn will_expire_soon(&self, seconds: u64) -> bool {
        Instant::now() + Duration::from_secs(seconds) >= self.expires_at
    }
}

pub struct WechatCache {
    store: Arc<RwLock<HashMap<String, CacheItem>>>,
}

impl WechatCache {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let store = self.store.read().await;
        if let Some(item) = store.get(key) {
            if !item.is_expired() {
                return Some(item.value.clone());
            }
        }
        None
    }

    pub async fn set(&self, key: String, value: String, ttl_seconds: u64) {
        let mut store = self.store.write().await;
        store.insert(key, CacheItem::new(value, ttl_seconds));
    }

    pub async fn should_refresh(&self, key: &str, advance_seconds: u64) -> bool {
        let store = self.store.read().await;
        match store.get(key) {
            Some(item) => item.will_expire_soon(advance_seconds),
            None => true,
        }
    }

    pub async fn clear(&self) {
        let mut store = self.store.write().await;
        store.clear();
    }

    pub async fn remove(&self, key: &str) {
        let mut store = self.store.write().await;
        store.remove(key);
    }
}

impl Clone for WechatCache {
    fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
        }
    }
}