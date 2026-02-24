use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Station {
    #[serde(alias = "id")]
    pub station_uuid: String,
    pub name: String,
    #[serde(alias = "stream_url")]
    pub url_resolved: String,
    pub homepage: Option<String>,
    pub favicon: Option<String>,
    pub tags: Vec<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub language: Option<String>,
    pub codec: Option<String>,
    pub bitrate: Option<u32>,
    pub votes: Option<u32>,
    #[serde(alias = "clicks")]
    pub click_count: Option<u32>,
}

impl Station {
    pub fn matches_query(&self, query: &str) -> bool {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return true;
        }

        self.name.to_lowercase().contains(&q)
            || self.tags.iter().any(|t| t.to_lowercase().contains(&q))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StationFilters {
    pub country: Option<String>,
    pub language: Option<String>,
    pub tag: Option<String>,
    pub codec: Option<String>,
    pub min_bitrate: Option<u32>,
}

impl StationFilters {
    pub fn is_empty(&self) -> bool {
        self.country.is_none()
            && self.language.is_none()
            && self.tag.is_none()
            && self.codec.is_none()
            && self.min_bitrate.is_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StationSort {
    Name,
    #[default]
    Votes,
    Clicks,
    Bitrate,
}

impl StationSort {
    pub fn as_api_order(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Votes => "votes",
            Self::Clicks => "clickcount",
            Self::Bitrate => "bitrate",
        }
    }

    pub fn is_descending(self) -> bool {
        !matches!(self, Self::Name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StationSearchQuery {
    pub query: String,
    pub filters: StationFilters,
    pub sort: StationSort,
    pub limit: usize,
}

impl Default for StationSearchQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            filters: StationFilters::default(),
            sort: StationSort::default(),
            limit: 50,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_legacy_station_fields() {
        let json = r#"{
            "id":"legacy-id",
            "name":"Legacy Station",
            "stream_url":"https://example.com/legacy",
            "homepage":null,
            "tags":["news"],
            "country":"US",
            "language":"english",
            "codec":"mp3",
            "bitrate":128,
            "votes":10,
            "clicks":42
        }"#;

        let station: Station = serde_json::from_str(json).expect("deserialize legacy station");
        assert_eq!(station.station_uuid, "legacy-id");
        assert_eq!(station.url_resolved, "https://example.com/legacy");
        assert_eq!(station.click_count, Some(42));
    }

    #[test]
    fn deserialize_new_station_fields() {
        let json = r#"{
            "station_uuid":"new-id",
            "name":"New Station",
            "url_resolved":"https://example.com/new",
            "homepage":null,
            "favicon":"https://example.com/favicon.png",
            "tags":["jazz"],
            "country":"US",
            "country_code":"US",
            "language":"english",
            "codec":"mp3",
            "bitrate":192,
            "votes":12,
            "click_count":99
        }"#;

        let station: Station = serde_json::from_str(json).expect("deserialize new station");
        assert_eq!(station.station_uuid, "new-id");
        assert_eq!(station.url_resolved, "https://example.com/new");
        assert_eq!(station.country_code.as_deref(), Some("US"));
        assert_eq!(station.click_count, Some(99));
    }
}
