use iradio::domain::palette::{fuzzy_filter, PaletteItem};

#[test]
fn fuzzy_filter_ranks_matches() {
    let items = vec![
        PaletteItem {
            label: "Play selected station".to_string(),
            action: "play".to_string(),
        },
        PaletteItem {
            label: "Pause playback".to_string(),
            action: "pause".to_string(),
        },
    ];

    let result = fuzzy_filter(&items, "pla");
    assert!(!result.is_empty());
    assert_eq!(result[0].action, "play");
}
