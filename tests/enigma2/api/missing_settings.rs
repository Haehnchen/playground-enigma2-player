use enigma2_player_core::enigma2::api::Enigma2Client;

#[test]
fn enigma2_client_bouquets_requires_configured_base_url() {
    let client = Enigma2Client::new("");

    let err = client.bouquets().unwrap_err();

    assert_eq!(
        err.to_string(),
        "Dreambox URL is not configured in settings"
    );
}
