use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NidusLock {
    pub version: String,
    pub generated_at: String,
    pub dependencies: HashMap<String, LockEntry>,
}

impl Default for NidusLock {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockEntry {
    pub branch: String,
    pub commit: String,
    pub locked_at: String,
}

impl NidusLock {
    pub fn new() -> Self {
        Self {
            version: "1".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            dependencies: HashMap::new(),
        }
    }

    pub fn load() -> Option<Self> {
        let content = std::fs::read_to_string("nidus.lock").ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        std::fs::write("nidus.lock", json)
    }

    pub fn add_entry(&mut self, url: &str, branch: &str, commit: &str) {
        self.dependencies.insert(
            url.to_string(),
            LockEntry {
                branch: branch.to_string(),
                commit: commit.to_string(),
                locked_at: chrono::Utc::now().to_rfc3339(),
            },
        );
    }
}
