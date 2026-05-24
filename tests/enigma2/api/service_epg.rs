use crate::support::client_with_responses;

#[test]
fn enigma2_client_service_epg_loads_events_and_reuses_cache() {
    let epg = r#"{
        "events": [{
            "begin_timestamp": 100,
            "duration_sec": 100,
            "now_timestamp": 125,
            "title": "Tagesschau",
            "shortdesc": "Nachrichten",
            "longdesc": "Nachrichten &amp; Wetter",
            "sref": "service-ref",
            "sname": "Das Erste HD"
        }]
    }"#;
    let (client, requests) = client_with_responses(vec![(
        "http://receiver.local/api/epgservice?sRef=service-ref",
        epg,
    )]);

    let events = client.service_epg("service-ref").unwrap();
    let cached = client.service_epg("service-ref").unwrap();

    assert_eq!(events, cached);
    assert_eq!(events[0].title, "Tagesschau");
    assert_eq!(
        events[0].description(),
        "Nachrichten\n\nNachrichten & Wetter"
    );
    assert_eq!(
        requests.lock().unwrap().as_slice(),
        ["http://receiver.local/api/epgservice?sRef=service-ref"]
    );
}
