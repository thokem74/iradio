use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlashCommand {
    Play(String),
    Stop,
    Pause,
    Resume,
    Search(String),
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
                let query = parts.collect::<Vec<_>>().join(" ");
                if query.is_empty() {
                    Ok(Self::Play("selected".to_string()))
                } else {
                    Ok(Self::Play(query))
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
            "fav" | "favorite" => Ok(Self::Favorite),
            "unfav" | "unfavorite" => Ok(Self::Unfavorite),
            "quit" | "q" => Ok(Self::Quit),
            "help" => Ok(Self::Help),
            _ => Err(anyhow!("unknown command: {cmd}")),
        }
    }
}
