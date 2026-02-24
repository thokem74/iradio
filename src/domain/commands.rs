use anyhow::{anyhow, Result};

use crate::domain::models::{StationFilters, StationSort};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayTarget {
    Selected,
    Index(usize),
    Query(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlashCommand {
    Play(PlayTarget),
    Stop,
    Pause,
    Resume,
    Search(String),
    Filter(StationFilters),
    ClearFilters,
    Sort(StationSort),
    Favorites,
    Favorite,
    Unfavorite,
    Quit,
    Help,
}

impl SlashCommand {
    pub fn parse(input: &str) -> Result<Self> {
        let trimmed = input.trim();
        if !trimmed.starts_with('/') {
            return Err(anyhow!("slash commands must start with '/'"));
        }

        let mut parts = trimmed[1..].split_whitespace();
        let cmd = parts.next().ok_or_else(|| anyhow!("empty command"))?;

        match cmd {
            "play" => {
                let value = parts.collect::<Vec<_>>().join(" ");
                if value.is_empty() || value.eq_ignore_ascii_case("selected") {
                    Ok(Self::Play(PlayTarget::Selected))
                } else if let Ok(index) = value.parse::<usize>() {
                    if index == 0 {
                        Err(anyhow!("play index must be >= 1"))
                    } else {
                        Ok(Self::Play(PlayTarget::Index(index)))
                    }
                } else {
                    Ok(Self::Play(PlayTarget::Query(value)))
                }
            }
            "stop" => Ok(Self::Stop),
            "pause" => Ok(Self::Pause),
            "resume" => Ok(Self::Resume),
            "search" => {
                let query = parts.collect::<Vec<_>>().join(" ");
                if query.is_empty() {
                    Err(anyhow!("usage: /search <query>"))
                } else {
                    Ok(Self::Search(query))
                }
            }
            "filter" => {
                let args = parts.collect::<Vec<_>>();
                if args.is_empty() {
                    return Err(anyhow!(
                        "usage: /filter country=<x> language=<y> tag=<z> codec=<c> min_bitrate=<n>"
                    ));
                }
                Ok(Self::Filter(parse_filter_args(&args)?))
            }
            "clear-filters" => Ok(Self::ClearFilters),
            "sort" => {
                let value = parts
                    .next()
                    .ok_or_else(|| anyhow!("usage: /sort <name|votes|clicks|bitrate>"))?;
                if parts.next().is_some() {
                    return Err(anyhow!("usage: /sort <name|votes|clicks|bitrate>"));
                }
                let sort = match value.to_ascii_lowercase().as_str() {
                    "name" => StationSort::Name,
                    "votes" => StationSort::Votes,
                    "clicks" => StationSort::Clicks,
                    "bitrate" => StationSort::Bitrate,
                    _ => return Err(anyhow!("invalid sort field: {value}")),
                };
                Ok(Self::Sort(sort))
            }
            "favorites" => Ok(Self::Favorites),
            "fav" | "favorite" => Ok(Self::Favorite),
            "unfav" | "unfavorite" => Ok(Self::Unfavorite),
            "quit" | "q" => Ok(Self::Quit),
            "help" => Ok(Self::Help),
            _ => Err(anyhow!("unknown command: {cmd}")),
        }
    }
}

fn parse_filter_args(args: &[&str]) -> Result<StationFilters> {
    let mut filters = StationFilters::default();

    for arg in args {
        let (key, value) = arg
            .split_once('=')
            .ok_or_else(|| anyhow!("invalid filter syntax: {arg} (expected key=value)"))?;
        if value.trim().is_empty() {
            return Err(anyhow!("filter value cannot be empty for key: {key}"));
        }

        match key.to_ascii_lowercase().as_str() {
            "country" => filters.country = Some(value.to_string()),
            "language" => filters.language = Some(value.to_string()),
            "tag" => filters.tag = Some(value.to_string()),
            "codec" => filters.codec = Some(value.to_string()),
            "min_bitrate" => {
                let bitrate = value
                    .parse::<u32>()
                    .map_err(|_| anyhow!("min_bitrate must be an integer"))?;
                filters.min_bitrate = Some(bitrate);
            }
            _ => return Err(anyhow!("unknown filter key: {key}")),
        }
    }

    Ok(filters)
}
