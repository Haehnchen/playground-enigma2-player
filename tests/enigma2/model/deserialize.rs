use enigma2_player_core::enigma2::model::{EpgNowResponse, ServicesResponse};

#[test]
fn services_response_deserializes_normalized_bouquet_and_channel_names() {
    let services: ServicesResponse = serde_json::from_str(
        r#"{
            "result": true,
            "services": [{
                "servicename": "TV &amp; Freies TV",
                "servicereference": "bouquet-ref",
                "subservices": [{
                    "pos": 7,
                    "servicename": "ORF 1 HD Snow White &amp; The Huntsman",
                    "servicereference": "service-ref",
                    "program": 1
                }]
            }]
        }"#,
    )
    .unwrap();

    assert_eq!(services.services[0].servicename, "TV & Freies TV");
    assert_eq!(
        services.services[0].subservices[0].servicename,
        "ORF 1 HD Snow White & The Huntsman"
    );
}

#[test]
fn epg_now_response_deserializes_normalized_event_text() {
    let epg: EpgNowResponse = serde_json::from_str(
        r#"{
            "events": [{
                "title": "Radsport: Giro d&#x27;Italia",
                "shortdesc": "Kurz",
                "longdesc": "Lang &amp; sauber",
                "genre": "Sport &amp; Freizeit",
                "sref": "service-ref",
                "sname": "Eurosport &amp; Co"
            }]
        }"#,
    )
    .unwrap();

    let event = &epg.events[0];
    assert_eq!(
        event.title,
        format!("Radsport: Giro d{}Italia", char::from(39))
    );
    assert_eq!(event.sname, "Eurosport & Co");
    assert_eq!(event.genre, "Sport & Freizeit");
    assert_eq!(event.description(), "Kurz\n\nLang & sauber");
}
