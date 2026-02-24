use anyhow::{Context, Result};
use reqwest::Client;

use crate::domain::models::Station;

pub struct RadioBrowserCatalog {
    client: Client,
    base_url: String,
}

impl RadioBrowserCatalog {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
        }
    }

    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<Station>> {
        let url = format!(
            "{}/json/stations/search?name={}&limit={}",
            self.base_url, query, limit
        );

        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("failed to request station catalog")?
            .error_for_status()
            .context("station catalog returned error status")?;

        #[derive(serde::Deserialize)]
        struct ApiStation {
            stationuuid: String,
            name: String,
            url_resolved: String,
            homepage: Option<String>,
            tags: String,
        }

        let api_stations: Vec<ApiStation> = response
            .json()
            .await
            .context("failed to deserialize station catalog response")?;

        Ok(api_stations
            .into_iter()
            .map(|s| Station {
                id: s.stationuuid,
                name: s.name,
                stream_url: s.url_resolved,
                homepage: s.homepage,
                tags: s
                    .tags
                    .split(',')
                    .map(str::trim)
                    .filter(|t| !t.is_empty())
                    .map(ToString::to_string)
                    .collect(),
            })
            .collect())
    }
}
