use enigma2_player_core::enigma2::model::{attach_epg, Bouquet, Channel, EpgEvent};

fn channel(service_ref: &str) -> Channel {
    Channel {
        position: 82,
        name: "ZDF HD".to_string(),
        service_ref: service_ref.to_string(),
        program: Some(11110),
        epg: None,
    }
}

#[test]
fn attach_epg_matches_events_by_service_reference() {
    let service_ref = "1:0:19:2B66:3F3:1:C00000:0:0:0:";
    let bouquet = Bouquet {
        name: "Freie Sender".to_string(),
        service_ref: "bouquet-ref".to_string(),
        channels: vec![channel(service_ref)],
    };
    let event = EpgEvent {
        id: Some(1),
        begin_timestamp: 100,
        duration_sec: 100,
        title: "Heute".to_string(),
        shortdesc: "Kurz".to_string(),
        longdesc: "Lang".to_string(),
        sref: service_ref.to_string(),
        sname: "ZDF HD".to_string(),
        now_timestamp: 150,
        remaining: 50,
    };

    let bouquet = attach_epg(bouquet, &[event]);
    let attached = bouquet.channels[0].epg.as_ref().unwrap();

    assert_eq!(attached.title, "Heute");
    assert_eq!(attached.progress(), 0.5);
    assert_eq!(attached.description(), "Kurz\n\nLang");
}
