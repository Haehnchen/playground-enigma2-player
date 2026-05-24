use enigma2_player_core::enigma2::urls::extract_stream_url;

#[test]
fn extract_stream_url_returns_first_non_comment_playlist_entry() {
    let m3u = "#EXTM3U\n#EXTVLCOPT:http-reconnect=true\nhttp://receiver.local:8001/ref\n";

    assert_eq!(
        extract_stream_url(m3u).as_deref(),
        Some("http://receiver.local:8001/ref")
    );
}

#[test]
fn extract_stream_url_returns_none_for_comment_only_playlist() {
    let m3u = "#EXTM3U\n#EXTVLCOPT:http-reconnect=true\n\n";

    assert_eq!(extract_stream_url(m3u), None);
}
