use enigma2_player_core::enigma2::urls::stream_m3u_url;

#[test]
fn stream_m3u_url_percent_encodes_service_reference() {
    let url = stream_m3u_url("http://receiver.local/", "1:0:19:2B66:3F3:1:C00000:0:0:0:");

    assert_eq!(
        url,
        "http://receiver.local/web/stream.m3u?ref=1%3A0%3A19%3A2B66%3A3F3%3A1%3AC00000%3A0%3A0%3A0%3A"
    );
}
