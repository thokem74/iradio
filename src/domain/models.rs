use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub stream_url: String,
    pub homepage: Option<String>,
    pub tags: Vec<String>,
    pub country: Option<String>,
    pub language: Option<String>,
    pub codec: Option<String>,
    pub bitrate: Option<u32>,
    pub votes: Option<u32>,
    pub clicks: Option<u32>,
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
