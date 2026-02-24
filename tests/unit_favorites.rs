use iradio::storage::favorites::FavoritesStore;

#[test]
fn save_and_load_favorites_round_trip() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("favorites.json");
    let store = FavoritesStore::new(&path);

    let station_ids = vec!["id-1".to_string(), "id-2".to_string()];

    store.save(&station_ids).expect("save favorites");
    let loaded = store.load().expect("load favorites");
    assert_eq!(station_ids, loaded);
}

#[test]
fn load_legacy_station_array_and_save_new_id_array() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("favorites.json");
    let store = FavoritesStore::new(&path);

    let legacy = r#"[
      {"id":"legacy-1","name":"Station 1","stream_url":"http://example.com/1","tags":[]},
      {"station_uuid":"legacy-2","name":"Station 2","url_resolved":"http://example.com/2","tags":[]}
    ]"#;
    std::fs::write(&path, legacy).expect("write legacy favorites");

    let loaded = store.load().expect("load legacy favorites");
    assert_eq!(loaded, vec!["legacy-1".to_string(), "legacy-2".to_string()]);

    store.save(&loaded).expect("save migrated favorites");
    let rewritten = std::fs::read_to_string(&path).expect("read rewritten favorites");
    assert!(rewritten.contains("legacy-1"));
    assert!(rewritten.contains("legacy-2"));
    assert!(!rewritten.contains("stream_url"));
}
