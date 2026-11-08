use super::*;

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
        genre: "News".to_string(),
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

fn event(shortdesc: &str, longdesc: &str) -> EpgEvent {
    EpgEvent {
        id: None,
        begin_timestamp: 0,
        duration_sec: 0,
        title: String::new(),
        shortdesc: shortdesc.to_string(),
        longdesc: longdesc.to_string(),
        genre: String::new(),
        sref: String::new(),
        sname: String::new(),
        now_timestamp: 0,
        remaining: 0,
    }
}

#[test]
fn epg_event_description_decodes_html_entities_and_escaped_newlines() {
    let event = event(
        "Nicky &quot;Koch&quot;",
        "Regie: Holger Haase\n\nDarsteller: Katharina M&uuml;ller-Elmau",
    );

    assert_eq!(
        event.description(),
        "Nicky \"Koch\"\n\nRegie: Holger Haase\n\nDarsteller: Katharina M\u{00fc}ller-Elmau"
    );
}

#[test]
fn epg_event_description_deduplicates_equal_shortdesc_and_longdesc() {
    let event = event("Same &#38; Value", "Same &amp; Value");

    assert_eq!(event.description(), "Same & Value");
}

#[test]
fn epg_event_description_skips_title_duplicate() {
    let mut event = event("Tagesschau", "Die Nachrichten der ARD");
    event.title = "Tagesschau".to_string();

    assert_eq!(event.description(), "Die Nachrichten der ARD");
}

#[test]
fn epg_event_description_skips_leading_title_line() {
    let mut event = event(
        "Atomic Blonde\nAction, USA 2017\nAltersfreigabe: ab 16",
        "Actionthriller",
    );
    event.title = "Atomic Blonde".to_string();

    assert_eq!(
        event.description(),
        "Action, USA 2017\nAltersfreigabe: ab 16\n\nActionthriller"
    );
}

#[test]
fn normalize_epg_text_preserves_unknown_entities_and_decodes_numeric_entities() {
    assert_eq!(
        normalize_epg_text("Foo &unknown; &#x27;bar&#x27;"),
        format!("Foo &unknown; {}bar{}", char::from(39), char::from(39))
    );
}

fn event_with_times(begin_timestamp: i64, duration_sec: i64, now_timestamp: i64) -> EpgEvent {
    EpgEvent {
        id: None,
        begin_timestamp,
        duration_sec,
        title: String::new(),
        shortdesc: String::new(),
        longdesc: String::new(),
        genre: String::new(),
        sref: String::new(),
        sname: String::new(),
        now_timestamp,
        remaining: 0,
    }
}

#[test]
fn epg_event_progress_returns_fraction_inside_event_bounds() {
    assert_eq!(event_with_times(100, 100, 150).progress(), 0.5);
}

#[test]
fn epg_event_progress_clamps_before_start_after_end_and_invalid_duration() {
    assert_eq!(event_with_times(100, 100, 50).progress(), 0.0);
    assert_eq!(event_with_times(100, 100, 250).progress(), 1.0);
    assert_eq!(event_with_times(100, 0, 150).progress(), 0.0);
    assert_eq!(event_with_times(100, -10, 150).progress(), 0.0);
}

#[test]
fn formats_epg_times_with_summer_and_winter_time() {
    let berlin = glib::TimeZone::new(Some("Europe/Berlin"));

    for (timestamp, expected) in [
        (1_794_144_600, "14:30"), // 2026-11-08 13:30 UTC (winter).
        (1_783_513_800, "14:30"), // 2026-07-08 12:30 UTC (summer).
        (1_774_744_200, "01:30"), // Before the spring clock change.
        (1_774_747_800, "03:30"), // After the spring clock change.
        (1_792_888_200, "02:30"), // Before the autumn clock change.
        (1_792_891_800, "02:30"), // After the autumn clock change.
    ] {
        assert_eq!(format_time_in_timezone(timestamp, &berlin), expected);
    }
}

#[test]
fn missing_or_invalid_epg_times_use_placeholder() {
    for timestamp in [0, -1, i64::MIN, i64::MAX] {
        assert_eq!(format_time(timestamp), "--:--");
    }
}
