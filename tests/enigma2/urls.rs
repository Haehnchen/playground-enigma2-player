use super::*;

#[test]
fn epg_now_url_percent_encodes_bouquet_reference() {
    let url = epg_now_url("http://receiver.local/", "1:7:1:FROM BOUQUET \"tv\"");

    assert_eq!(
        url,
        "http://receiver.local/api/epgnow?bRef=1%3A7%3A1%3AFROM+BOUQUET+%22tv%22"
    );
}

#[test]
fn epg_service_url_percent_encodes_service_reference() {
    let url = epg_service_url("http://receiver.local/", "1:0:19:283D:3FB:1:C00000:0:0:0:");

    assert_eq!(
        url,
        "http://receiver.local/api/epgservice?sRef=1%3A0%3A19%3A283D%3A3FB%3A1%3AC00000%3A0%3A0%3A0%3A"
    );
}

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

#[test]
fn normalize_base_url_trims_whitespace_and_trailing_slash_without_adding_scheme() {
    assert_eq!(
        normalize_base_url("  http://receiver.local/  "),
        "http://receiver.local"
    );
    assert_eq!(normalize_base_url("receiver.local/"), "receiver.local");
    assert_eq!(normalize_base_url("   "), "");
}

#[test]
fn has_supported_url_scheme_accepts_only_http_and_https() {
    assert!(has_supported_url_scheme("http://receiver.local"));
    assert!(has_supported_url_scheme("https://receiver.local"));
    assert!(!has_supported_url_scheme("receiver.local"));
}

#[test]
fn stream_m3u_url_percent_encodes_service_reference() {
    let url = stream_m3u_url("http://receiver.local/", "1:0:19:2B66:3F3:1:C00000:0:0:0:");

    assert_eq!(
        url,
        "http://receiver.local/web/stream.m3u?ref=1%3A0%3A19%3A2B66%3A3F3%3A1%3AC00000%3A0%3A0%3A0%3A"
    );
}
