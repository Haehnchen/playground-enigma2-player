use crate::support::client_with_responses;

#[test]
fn enigma2_client_bouquet_with_epg_attaches_normalized_events_and_reuses_epg_cache() {
    let services = r#"{
        "result": true,
        "services": [{
            "servicename": "TV",
            "servicereference": "bouquet-ref",
            "subservices": [{
                "pos": 1,
                "servicename": "Eurosport 1 HD",
                "servicereference": "service-ref"
            }]
        }]
    }"#;
    let epg = r#"{
        "events": [{
            "begin_timestamp": 100,
            "duration_sec": 100,
            "now_timestamp": 125,
            "title": "Giro d&#x27;Italia",
            "shortdesc": "Etappe &amp; Analyse",
            "longdesc": "Etappe &amp; Analyse",
            "sref": "service-ref",
            "sname": "Eurosport 1 HD"
        }]
    }"#;
    let (client, requests) = client_with_responses(vec![
        ("http://receiver.local/api/getallservices", services),
        ("http://receiver.local/api/epgnow?bRef=bouquet-ref", epg),
    ]);

    let bouquet = client.bouquet_with_epg(0).unwrap();
    let event = bouquet.channels[0].epg.as_ref().unwrap();
    assert_eq!(event.title, format!("Giro d{}Italia", char::from(39)));
    assert_eq!(event.description(), "Etappe & Analyse");
    assert_eq!(event.progress(), 0.25);

    let cached = client.epg_now("bouquet-ref").unwrap();
    assert_eq!(cached.len(), 1);
    assert_eq!(
        requests.lock().unwrap().as_slice(),
        [
            "http://receiver.local/api/getallservices",
            "http://receiver.local/api/epgnow?bRef=bouquet-ref",
        ]
    );
}

#[test]
fn enigma2_client_clones_share_epg_cache() {
    let epg = r#"{
        "events": [{
            "begin_timestamp": 100,
            "duration_sec": 100,
            "now_timestamp": 125,
            "title": "Tagesschau",
            "sref": "service-ref",
            "sname": "Das Erste HD"
        }]
    }"#;
    let (client, requests) = client_with_responses(vec![(
        "http://receiver.local/api/epgnow?bRef=bouquet-ref",
        epg,
    )]);
    let clone = client.clone();

    assert_eq!(
        client.epg_now("bouquet-ref").unwrap(),
        clone.epg_now("bouquet-ref").unwrap()
    );

    assert_eq!(
        requests.lock().unwrap().as_slice(),
        ["http://receiver.local/api/epgnow?bRef=bouquet-ref"]
    );
}
