use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const MAX_HISTORY: usize = 20;
const HISTORY_FILE: &str = ".prunner_history.json";

#[derive(Serialize, Deserialize, Default)]
pub struct History {
    entries: Vec<String>,
}

impl History {
    pub fn load() -> Self {
        let path = Self::path();
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(h) = serde_json::from_str::<History>(&data) {
                return h;
            }
        }
        History::default()
    }

    pub fn save(&self) {
        if let Some(path) = dirs::home_dir().map(|d| d.join(HISTORY_FILE)) {
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(path, json);
            }
        }
    }

    /// Push a new command. Deduplicates (moves to top if exists).
    pub fn push(&mut self, cmd: String) {
        // Remove existing occurrence
        self.entries.retain(|e| e != &cmd);
        // Insert at front
        self.entries.insert(0, cmd);
        // Cap at max
        self.entries.truncate(MAX_HISTORY);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    fn path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(HISTORY_FILE)
    }
}
