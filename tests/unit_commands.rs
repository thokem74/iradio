use iradio::domain::commands::{PlayTarget, SlashCommand};
use iradio::domain::models::{StationFilters, StationSort};

#[test]
fn parse_play_command() {
    let cmd = SlashCommand::parse("/play soma").expect("parse /play command");
    assert_eq!(
        cmd,
        SlashCommand::Play(PlayTarget::Query("soma".to_string()))
    );
}

#[test]
fn parse_play_without_args_uses_selected_station() {
    let cmd = SlashCommand::parse("/play").expect("parse /play command without args");
    assert_eq!(cmd, SlashCommand::Play(PlayTarget::Selected));
}

#[test]
fn parse_play_selected_alias() {
    let cmd = SlashCommand::parse("/play selected").expect("parse /play selected");
    assert_eq!(cmd, SlashCommand::Play(PlayTarget::Selected));
}

#[test]
fn parse_play_index() {
    let cmd = SlashCommand::parse("/play 1").expect("parse /play 1");
    assert_eq!(cmd, SlashCommand::Play(PlayTarget::Index(1)));
}

#[test]
fn parse_favorites_command() {
    let cmd = SlashCommand::parse("/favorites").expect("parse /favorites");
    assert_eq!(cmd, SlashCommand::Favorites);
}

#[test]
fn parse_volume_command_bounds() {
    let low = SlashCommand::parse("/volume 0").expect("parse /volume 0");
    let high = SlashCommand::parse("/volume 100").expect("parse /volume 100");
    assert_eq!(low, SlashCommand::Volume(0));
    assert_eq!(high, SlashCommand::Volume(100));
}

#[test]
fn parse_search_command() {
    let cmd = SlashCommand::parse("/search news radio").expect("parse /search command");
    assert_eq!(cmd, SlashCommand::Search("news radio".to_string()));
}

#[test]
fn parse_filter_command() {
    let cmd = SlashCommand::parse(
        "/filter country=US language=english tag=jazz codec=mp3 min_bitrate=128",
    )
    .expect("parse /filter command");
    assert_eq!(
        cmd,
        SlashCommand::Filter(StationFilters {
            country: Some("US".to_string()),
            language: Some("english".to_string()),
            tag: Some("jazz".to_string()),
            codec: Some("mp3".to_string()),
            min_bitrate: Some(128),
        })
    );
}

#[test]
fn parse_sort_command() {
    let cmd = SlashCommand::parse("/sort clicks").expect("parse /sort command");
    assert_eq!(cmd, SlashCommand::Sort(StationSort::Clicks));
}

#[test]
fn reject_invalid_filter_value() {
    let err = SlashCommand::parse("/filter min_bitrate=abc").expect_err("invalid should fail");
    assert!(err.to_string().contains("min_bitrate must be an integer"));
}

#[test]
fn reject_unknown_filter_key() {
    let err = SlashCommand::parse("/filter foo=bar").expect_err("invalid should fail");
    assert!(err.to_string().contains("unknown filter key"));
}

#[test]
fn reject_unknown_sort_field() {
    let err = SlashCommand::parse("/sort listeners").expect_err("invalid should fail");
    assert!(err.to_string().contains("invalid sort field"));
}

#[test]
fn reject_unknown_command() {
    let err = SlashCommand::parse("/does-not-exist").expect_err("unknown command should fail");
    assert!(err.to_string().contains("unknown command"));
}

#[test]
fn reject_zero_play_index() {
    let err = SlashCommand::parse("/play 0").expect_err("index 0 should fail");
    assert!(err.to_string().contains(">= 1"));
}

#[test]
fn reject_volume_without_value() {
    let err = SlashCommand::parse("/volume").expect_err("missing volume should fail");
    assert!(err.to_string().contains("usage: /volume <0-100>"));
}

#[test]
fn reject_volume_with_non_integer() {
    let err = SlashCommand::parse("/volume loud").expect_err("non-integer should fail");
    assert!(err.to_string().contains("volume must be an integer"));
}

#[test]
fn reject_volume_with_negative_integer() {
    let err = SlashCommand::parse("/volume -1").expect_err("negative should fail");
    assert!(err.to_string().contains("volume must be an integer"));
}

#[test]
fn reject_volume_out_of_range() {
    let err = SlashCommand::parse("/volume 101").expect_err("out of range should fail");
    assert!(err.to_string().contains("between 0 and 100"));
}

#[test]
fn reject_volume_with_extra_args() {
    let err = SlashCommand::parse("/volume 50 extra").expect_err("extra args should fail");
    assert!(err.to_string().contains("usage: /volume <0-100>"));
}
