use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub id: String,
    pub text: String,
    pub raw_text: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub transcribe_ms: u64,
    #[serde(default)]
    pub cleanup_ms: u64,
}

pub struct HistoryStore {
    items: Vec<HistoryItem>,
    path: PathBuf,
}

impl HistoryStore {
    pub fn load(data_dir: &Path) -> Self {
        let path = data_dir.join("history.json");
        let items = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Self { items, path }
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.items) {
            std::fs::write(&self.path, json).ok();
        }
    }

    pub fn add(&mut self, text: String, raw_text: String, transcribe_ms: u64, cleanup_ms: u64) {
        let item = HistoryItem {
            id: uuid::Uuid::new_v4().to_string(),
            text,
            raw_text,
            created_at: Utc::now(),
            transcribe_ms,
            cleanup_ms,
        };
        self.items.insert(0, item);
        if self.items.len() > 100 {
            self.items.truncate(100);
        }
        self.save();
    }

    pub fn items(&self) -> Vec<HistoryItem> {
        self.items.clone()
    }

    pub fn get(&self, id: &str) -> Option<&HistoryItem> {
        self.items.iter().find(|item| item.id == id)
    }

    pub fn delete(&mut self, id: &str) {
        self.items.retain(|item| item.id != id);
        self.save();
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.save();
    }
}
