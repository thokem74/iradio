use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;

use crate::domain::models::{Station, StationSearchQuery};

pub trait StationCatalog: Send {
    fn search(&self, query: &StationSearchQuery) -> Result<Vec<Station>>;
}

pub struct RadioBrowserCatalog {
    client: Client,
    base_url: String,
    timeout: Duration,
    max_retries: usize,
}

impl RadioBrowserCatalog {
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        Self::new_with_config(base_url, Duration::from_secs(3), 2)
    }

    pub fn new_with_config(
        base_url: impl Into<String>,
        timeout: Duration,
        max_retries: usize,
    ) -> Result<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .context("failed to build radio browser client")?;
        Ok(Self {
            client,
            base_url: base_url.into(),
            timeout,
            max_retries,
        })
    }

    fn build_params(&self, query: &StationSearchQuery) -> Vec<(String, String)> {
        let mut params = vec![
            ("hidebroken".to_string(), "true".to_string()),
            ("limit".to_string(), query.limit.to_string()),
            ("order".to_string(), query.sort.as_api_order().to_string()),
            (
                "reverse".to_string(),
                if query.sort.is_descending() {
                    "true".to_string()
                } else {
                    "false".to_string()
                },
            ),
        ];

        if !query.query.trim().is_empty() {
            params.push(("name".to_string(), query.query.trim().to_string()));
        }

        if let Some(country) = &query.filters.country {
            params.push(("country".to_string(), country.clone()));
        }
        if let Some(language) = &query.filters.language {
            params.push(("language".to_string(), language.clone()));
        }
        if let Some(tag) = &query.filters.tag {
            params.push(("tag".to_string(), tag.clone()));
        }
        if let Some(codec) = &query.filters.codec {
            params.push(("codec".to_string(), codec.clone()));
        }
        if let Some(min_bitrate) = query.filters.min_bitrate {
            params.push(("bitrateMin".to_string(), min_bitrate.to_string()));
        }

        params
    }
}

impl StationCatalog for RadioBrowserCatalog {
    fn search(&self, query: &StationSearchQuery) -> Result<Vec<Station>> {
        let url = format!("{}/json/stations/search", self.base_url);
        let params = self.build_params(query);

        #[derive(serde::Deserialize)]
        struct ApiStation {
            stationuuid: String,
            name: Option<String>,
            url: Option<String>,
            url_resolved: Option<String>,
            homepage: Option<String>,
            tags: Option<String>,
            country: Option<String>,
            language: Option<String>,
            codec: Option<String>,
            bitrate: Option<u32>,
            votes: Option<u32>,
            clickcount: Option<u32>,
        }

        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            let response = self.client.get(&url).query(&params).send();
            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_server_error() {
                        last_error = Some(anyhow!("station catalog server error: HTTP {status}"));
                    } else {
                        let api_stations: Vec<ApiStation> = resp
                            .error_for_status()
                            .context("station catalog returned error status")?
                            .json()
                            .context("failed to deserialize station catalog response")?;

                        let stations = api_stations
                            .into_iter()
                            .map(|s| {
                                let stream_url =
                                    s.url_resolved.or(s.url).unwrap_or_else(|| "".to_string());

                                Station {
                                    id: s.stationuuid,
                                    name: s
                                        .name
                                        .filter(|n| !n.trim().is_empty())
                                        .unwrap_or_else(|| "(unnamed station)".to_string()),
                                    stream_url,
                                    homepage: s.homepage,
                                    tags: s
                                        .tags
                                        .unwrap_or_default()
                                        .split(',')
                                        .map(str::trim)
                                        .filter(|t| !t.is_empty())
                                        .map(ToString::to_string)
                                        .collect(),
                                    country: s.country.filter(|v| !v.trim().is_empty()),
                                    language: s.language.filter(|v| !v.trim().is_empty()),
                                    codec: s.codec.filter(|v| !v.trim().is_empty()),
                                    bitrate: s.bitrate,
                                    votes: s.votes,
                                    clicks: s.clickcount,
                                }
                            })
                            .filter(|s| !s.stream_url.trim().is_empty())
                            .collect();

                        return Ok(stations);
                    }
                }
                Err(err) => {
                    last_error = Some(anyhow!(
                        "station catalog request failed (timeout={}ms): {err}",
                        self.timeout.as_millis()
                    ));
                }
            }

            if attempt < self.max_retries {
                let backoff = Duration::from_millis(150 * (attempt as u64 + 1));
                thread::sleep(backoff);
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("station catalog request failed")))
    }
}

pub struct StaticCatalog {
    stations: Vec<Station>,
}

impl StaticCatalog {
    pub fn new(stations: Vec<Station>) -> Self {
        Self { stations }
    }
}

impl StationCatalog for StaticCatalog {
    fn search(&self, query: &StationSearchQuery) -> Result<Vec<Station>> {
        let text = query.query.trim().to_ascii_lowercase();
        let mut stations: Vec<Station> = self
            .stations
            .iter()
            .filter(|station| text.is_empty() || station.matches_query(&text))
            .cloned()
            .collect();

        match query.sort {
            crate::domain::models::StationSort::Name => {
                stations.sort_by(|a, b| a.name.cmp(&b.name));
            }
            crate::domain::models::StationSort::Votes => {
                stations.sort_by(|a, b| b.votes.cmp(&a.votes).then_with(|| a.name.cmp(&b.name)));
            }
            crate::domain::models::StationSort::Clicks => {
                stations.sort_by(|a, b| b.clicks.cmp(&a.clicks).then_with(|| a.name.cmp(&b.name)));
            }
            crate::domain::models::StationSort::Bitrate => {
                stations
                    .sort_by(|a, b| b.bitrate.cmp(&a.bitrate).then_with(|| a.name.cmp(&b.name)));
            }
        }

        if stations.len() > query.limit {
            stations.truncate(query.limit);
        }

        Ok(stations)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    use super::*;
    use crate::domain::models::{StationFilters, StationSort};

    #[test]
    fn search_builds_filter_and_sort_params() {
        let listener = match TcpListener::bind("127.0.0.1:0") {
            Ok(listener) => listener,
            Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => return,
            Err(err) => panic!("bind listener: {err}"),
        };
        let addr = listener.local_addr().expect("local addr");

        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut buf = [0_u8; 8192];
            let n = stream.read(&mut buf).expect("read request");
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            assert!(req.contains("name=jazz"));
            assert!(req.contains("country=US"));
            assert!(req.contains("language=english"));
            assert!(req.contains("tag=smooth"));
            assert!(req.contains("codec=mp3"));
            assert!(req.contains("bitrateMin=128"));
            assert!(req.contains("order=clickcount"));
            assert!(req.contains("reverse=true"));

            let body = r#"[{"stationuuid":"id1","name":"Jazz FM","url_resolved":"https://example.com/stream","tags":"jazz,smooth","country":"US","language":"english","codec":"mp3","bitrate":128,"votes":10,"clickcount":20}]"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .expect("write response");
        });

        let catalog = RadioBrowserCatalog::new_with_config(
            format!("http://{addr}"),
            Duration::from_secs(1),
            0,
        )
        .expect("create catalog");
        let stations = catalog
            .search(&StationSearchQuery {
                query: "jazz".to_string(),
                filters: StationFilters {
                    country: Some("US".to_string()),
                    language: Some("english".to_string()),
                    tag: Some("smooth".to_string()),
                    codec: Some("mp3".to_string()),
                    min_bitrate: Some(128),
                },
                sort: StationSort::Clicks,
                limit: 25,
            })
            .expect("search stations");

        handle.join().expect("join server");
        assert_eq!(stations.len(), 1);
        assert_eq!(stations[0].name, "Jazz FM");
        assert_eq!(stations[0].clicks, Some(20));
    }

    #[test]
    fn search_retries_after_server_error() {
        let listener = match TcpListener::bind("127.0.0.1:0") {
            Ok(listener) => listener,
            Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => return,
            Err(err) => panic!("bind listener: {err}"),
        };
        let addr = listener.local_addr().expect("local addr");

        let handle = std::thread::spawn(move || {
            for idx in 0..2 {
                let (mut stream, _) = listener.accept().expect("accept request");
                let mut buf = [0_u8; 2048];
                let _ = stream.read(&mut buf).expect("read request");

                if idx == 0 {
                    stream
                        .write_all(
                            b"HTTP/1.1 500 Internal Server Error\r\ncontent-length: 0\r\n\r\n",
                        )
                        .expect("write error response");
                } else {
                    let body = r#"[{"stationuuid":"id2","name":"Retry FM","url_resolved":"https://example.com/retry","tags":""}]"#;
                    let response = format!(
                        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    stream
                        .write_all(response.as_bytes())
                        .expect("write success response");
                }
            }
        });

        let catalog = RadioBrowserCatalog::new_with_config(
            format!("http://{addr}"),
            Duration::from_secs(1),
            1,
        )
        .expect("create catalog");

        let stations = catalog
            .search(&StationSearchQuery::default())
            .expect("search should retry and succeed");

        handle.join().expect("join server");
        assert_eq!(stations.len(), 1);
        assert_eq!(stations[0].name, "Retry FM");
    }
}
