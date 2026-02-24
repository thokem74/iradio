use iradio::domain::models::Station;
use iradio::storage::favorites::FavoritesStore;

#[test]
fn save_and_load_favorites_round_trip() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("favorites.json");
    let store = FavoritesStore::new(&path);

    let stations = vec![Station {
        id: "id-1".to_string(),
        name: "Station 1".to_string(),
        stream_url: "http://example.com/1".to_string(),
        homepage: None,
        tags: vec!["tag".to_string()],
    }];

    store.save(&stations).expect("save favorites");
    let loaded = store.load().expect("load favorites");
    assert_eq!(stations, loaded);
}
