use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct FavoritesStore {
    path: PathBuf,
}

impl FavoritesStore {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn load(&self) -> Result<Vec<String>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.path)
            .with_context(|| format!("failed to read favorites file: {}", self.path.display()))?;

        if let Ok(ids) = serde_json::from_str::<Vec<String>>(&content) {
            return Ok(ids);
        }

        #[derive(serde::Deserialize)]
        struct LegacyFavoriteStation {
            station_uuid: Option<String>,
            id: Option<String>,
        }
        let legacy = serde_json::from_str::<Vec<LegacyFavoriteStation>>(&content)
            .with_context(|| format!("failed to parse favorites file: {}", self.path.display()))?;
        let ids = legacy
            .into_iter()
            .filter_map(|entry| entry.station_uuid.or(entry.id))
            .collect();

        Ok(ids)
    }

    pub fn save(&self, ids: &[String]) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create favorites directory: {}", parent.display())
            })?;
        }

        let body = serde_json::to_string_pretty(ids).context("failed to serialize favorites")?;
        fs::write(&self.path, body)
            .with_context(|| format!("failed to write favorites file: {}", self.path.display()))?;

        Ok(())
    }
}
