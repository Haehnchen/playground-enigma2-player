use enigma2_player_core::enigma2::urls::{has_supported_url_scheme, normalize_base_url};

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
