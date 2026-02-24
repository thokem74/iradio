use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::domain::models::Station;

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

    pub fn load(&self) -> Result<Vec<Station>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.path)
            .with_context(|| format!("failed to read favorites file: {}", self.path.display()))?;

        let favorites = serde_json::from_str::<Vec<Station>>(&content)
            .with_context(|| format!("failed to parse favorites file: {}", self.path.display()))?;

        Ok(favorites)
    }

    pub fn save(&self, stations: &[Station]) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create favorites directory: {}", parent.display())
            })?;
        }

        let body =
            serde_json::to_string_pretty(stations).context("failed to serialize favorites")?;
        fs::write(&self.path, body)
            .with_context(|| format!("failed to write favorites file: {}", self.path.display()))?;

        Ok(())
    }
}
