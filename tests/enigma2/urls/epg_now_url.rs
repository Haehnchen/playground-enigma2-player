use enigma2_player_core::enigma2::urls::epg_now_url;

#[test]
fn epg_now_url_percent_encodes_bouquet_reference() {
    let url = epg_now_url("http://receiver.local/", "1:7:1:FROM BOUQUET \"tv\"");

    assert_eq!(
        url,
        "http://receiver.local/api/epgnow?bRef=1%3A7%3A1%3AFROM+BOUQUET+%22tv%22"
    );
}
