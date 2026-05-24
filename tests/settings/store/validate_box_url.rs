use enigma2_player_core::settings::store::{validate_box_url, BOX_URL_SCHEME_ERROR};

#[test]
fn validate_box_url_accepts_http_and_https_urls() {
    assert_eq!(
        validate_box_url(" http://receiver.local/ ").unwrap(),
        "http://receiver.local"
    );
    assert_eq!(
        validate_box_url("https://receiver.local/").unwrap(),
        "https://receiver.local"
    );
}

#[test]
fn validate_box_url_rejects_empty_and_scheme_less_urls() {
    assert_eq!(validate_box_url("").unwrap_err(), BOX_URL_SCHEME_ERROR);
    assert_eq!(
        validate_box_url("receiver.local").unwrap_err(),
        BOX_URL_SCHEME_ERROR
    );
}
