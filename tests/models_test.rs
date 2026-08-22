use apple_tui::api::models::{RawCatalogResponse, SearchResults, Song};

#[test]
fn test_song_deserialization() {
    let json = r#"{
        "id": "1440857781",
        "type": "songs",
        "href": "/v1/catalog/us/songs/1440857781",
        "attributes": {
            "name": "Blinding Lights",
            "artistName": "The Weeknd",
            "albumName": "After Hours",
            "durationInMillis": 200040,
            "trackNumber": 9,
            "releaseDate": "2019-11-29",
            "isrc": "USUG11904206",
            "url": "https://music.apple.com/us/album/blinding-lights/1440857781?i=1440857781"
        }
    }"#;

    let song: Song = serde_json::from_str(json).unwrap();
    assert_eq!(song.id, "1440857781");
    assert_eq!(song.name, "Blinding Lights");
    assert_eq!(song.artist_name, "The Weeknd");
    assert_eq!(song.album_name.as_deref(), Some("After Hours"));
    assert_eq!(song.duration_in_millis, 200040);
    assert_eq!(song.formatted_duration(), "3:20");
    assert_eq!(song.playback_id(), "1440857781");
}

#[test]
fn test_library_song_play_params() {
    let json = r#"{
        "id": "i.RBrxWXmHYbA5BbA",
        "type": "library-songs",
        "attributes": {
            "name": "A&W",
            "artistName": "Lana Del Rey",
            "durationInMillis": 433916,
            "playParams": {
                "id": "i.RBrxWXmHYbA5BbA",
                "kind": "song",
                "isLibrary": true,
                "catalogId": "1655349125"
            }
        }
    }"#;

    let song: Song = serde_json::from_str(json).unwrap();
    assert_eq!(song.id, "i.RBrxWXmHYbA5BbA");
    assert_eq!(song.catalog_id.as_deref(), Some("1655349125"));
    assert_eq!(song.playback_id(), "1655349125");
}

#[test]
fn test_catalog_search_response_parsing() {
    let json = r#"{
        "results": {
            "songs": {
                "data": [
                    {
                        "id": "1",
                        "type": "songs",
                        "attributes": {
                            "name": "Starboy",
                            "artistName": "The Weeknd",
                            "albumName": "Starboy",
                            "durationInMillis": 230453
                        }
                    }
                ]
            },
            "albums": {
                "data": [
                    {
                        "id": "100",
                        "type": "albums",
                        "attributes": {
                            "name": "Starboy",
                            "artistName": "The Weeknd",
                            "trackCount": 18
                        }
                    }
                ]
            }
        }
    }"#;

    let raw: RawCatalogResponse = serde_json::from_str(json).unwrap();
    let search_results = SearchResults::from(raw);
    assert_eq!(search_results.songs.len(), 1);
    assert_eq!(search_results.songs[0].name, "Starboy");
    assert_eq!(search_results.albums.len(), 1);
    assert_eq!(search_results.albums[0].name, "Starboy");
}
