use enigma2_player_core::enigma2::urls::epg_service_url;

#[test]
fn epg_service_url_percent_encodes_service_reference() {
    let url = epg_service_url("http://receiver.local/", "1:0:19:283D:3FB:1:C00000:0:0:0:");

    assert_eq!(
        url,
        "http://receiver.local/api/epgservice?sRef=1%3A0%3A19%3A283D%3A3FB%3A1%3AC00000%3A0%3A0%3A0%3A"
    );
}
