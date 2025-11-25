use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use super::scryfall::Card;

const CACHE_DURATION_HOURS: u64 = 24;

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    card: Card,
    timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct CacheData {
    entries: HashMap<String, CacheEntry>,
}

pub struct CardCache {
    cache_path: PathBuf,
    data: CacheData,
}

impl CardCache {
    pub fn new() -> Self {
        let cache_path = Self::get_cache_path();
        let data = Self::load_cache(&cache_path).unwrap_or_default();

        Self { cache_path, data }
    }

    fn get_cache_path() -> PathBuf {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("mtg");

        fs::create_dir_all(&cache_dir).ok();
        cache_dir.join("card_cache.json")
    }

    fn load_cache(path: &PathBuf) -> Option<CacheData> {
        let contents = fs::read_to_string(path).ok()?;
        serde_json::from_str(&contents).ok()
    }

    fn save_cache(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.data) {
            fs::write(&self.cache_path, json).ok();
        }
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs()
    }

    fn is_expired(timestamp: u64) -> bool {
        let now = Self::current_timestamp();
        let age = now.saturating_sub(timestamp);
        age > CACHE_DURATION_HOURS * 3600
    }

    pub fn get(&self, key: &str) -> Option<Card> {
        let normalized_key = key.to_lowercase();

        self.data.entries.get(&normalized_key).and_then(|entry| {
            if Self::is_expired(entry.timestamp) {
                None
            } else {
                Some(entry.card.clone())
            }
        })
    }

    pub fn set(&self, key: &str, card: &Card) {
        let normalized_key = key.to_lowercase();
        let entry = CacheEntry {
            card: card.clone(),
            timestamp: Self::current_timestamp(),
        };

        // Use interior mutability pattern or just save directly
        // For simplicity, we'll use a new instance each time
        let mut data = Self::load_cache(&self.cache_path).unwrap_or_default();
        data.entries.insert(normalized_key, entry);

        if let Ok(json) = serde_json::to_string_pretty(&data) {
            fs::write(&self.cache_path, json).ok();
        }
    }

    pub fn clear(&self) {
        fs::remove_file(&self.cache_path).ok();
    }
}

impl Default for CardCache {
    fn default() -> Self {
        Self::new()
    }
}
