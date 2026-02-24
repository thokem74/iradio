use iradio::domain::commands::SlashCommand;

#[test]
fn parse_play_command() {
    let cmd = SlashCommand::parse("/play soma").expect("parse /play command");
    assert_eq!(cmd, SlashCommand::Play("soma".to_string()));
}

#[test]
fn parse_search_command() {
    let cmd = SlashCommand::parse("/search news radio").expect("parse /search command");
    assert_eq!(cmd, SlashCommand::Search("news radio".to_string()));
}

#[test]
fn reject_unknown_command() {
    let err = SlashCommand::parse("/does-not-exist").expect_err("unknown command should fail");
    assert!(err.to_string().contains("unknown command"));
}
