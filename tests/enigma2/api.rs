use super::*;

fn client_with_responses(responses: Vec<(&str, &str)>) -> (Enigma2Client, Arc<Mutex<Vec<String>>>) {
    let responses = Arc::new(Mutex::new(
        responses
            .into_iter()
            .map(|(url, body)| (url.to_string(), body.to_string()))
            .collect::<Vec<_>>(),
    ));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let requests_for_get = requests.clone();
    let responses_for_get = responses.clone();
    let client = Enigma2Client::with_http_getter("http://receiver.local", move |url| {
        requests_for_get.lock().unwrap().push(url.to_string());
        let mut responses = responses_for_get.lock().unwrap();
        if responses.is_empty() {
            return Err(Enigma2Error::Http(format!("unexpected request {url}")));
        }
        let (expected_url, body) = responses.remove(0);
        assert_eq!(url, expected_url);
        Ok(body)
    });
    (client, requests)
}

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

#[test]
fn enigma2_client_bouquets_parses_filters_normalizes_and_caches_services() {
    let services = r#"{
        "result": true,
        "services": [
            {
                "servicename": "TV &amp; Freies TV",
                "servicereference": "bouquet-ref",
                "subservices": [{
                    "pos": 1,
                    "servicename": "Das Erste &amp; HD",
                    "servicereference": "service-ref",
                    "program": 10
                }]
            },
            {
                "servicename": "",
                "servicereference": "ignored-empty-name",
                "subservices": [{"pos": 2, "servicename": "Ignored", "servicereference": "ignored"}]
            },
            {
                "servicename": "Ignored Empty Bouquet",
                "servicereference": "ignored-empty-bouquet",
                "subservices": []
            }
        ]
    }"#;
    let (client, requests) =
        client_with_responses(vec![("http://receiver.local/api/getallservices", services)]);

    let first = client.bouquets().unwrap();
    let second = client.bouquets().unwrap();

    assert_eq!(first, second);
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].name, "TV & Freies TV");
    assert_eq!(first[0].channels[0].name, "Das Erste & HD");
    assert_eq!(first[0].channels[0].program, Some(10));
    assert_eq!(
        requests.lock().unwrap().as_slice(),
        ["http://receiver.local/api/getallservices"]
    );
}

#[test]
fn enigma2_client_clones_share_services_cache() {
    let services = r#"{
        "result": true,
        "services": [{
            "servicename": "TV",
            "servicereference": "bouquet-ref",
            "subservices": [{
                "pos": 1,
                "servicename": "Das Erste HD",
                "servicereference": "service-ref"
            }]
        }]
    }"#;
    let (client, requests) =
        client_with_responses(vec![("http://receiver.local/api/getallservices", services)]);
    let clone = client.clone();

    assert_eq!(client.bouquets().unwrap(), clone.bouquets().unwrap());

    assert_eq!(
        requests.lock().unwrap().as_slice(),
        ["http://receiver.local/api/getallservices"]
    );
}

#[test]
fn enigma2_client_bouquets_reports_invalid_services_result() {
    let (client, requests) = client_with_responses(vec![(
        "http://receiver.local/api/getallservices",
        r#"{"result": false, "services": []}"#,
    )]);

    let err = client.bouquets().unwrap_err();

    assert_eq!(err.to_string(), "Dreambox did not accept getallservices");
    assert_eq!(
        requests.lock().unwrap().as_slice(),
        ["http://receiver.local/api/getallservices"]
    );
}

#[test]
fn enigma2_client_bouquets_requires_configured_base_url() {
    let client = Enigma2Client::new("");

    let err = client.bouquets().unwrap_err();

    assert_eq!(
        err.to_string(),
        "Dreambox URL is not configured in settings"
    );
    assert!(matches!(
        client.epg_now("ref"),
        Err(Enigma2Error::MissingSettings(_))
    ));
    assert!(matches!(
        client.service_epg("ref"),
        Err(Enigma2Error::MissingSettings(_))
    ));
    assert!(matches!(
        client.resolve_stream_url("ref"),
        Err(Enigma2Error::MissingSettings(_))
    ));
}

#[test]
fn current_and_service_epg_keep_separate_cache_entries_for_the_same_reference() {
    let (client, requests) = client_with_responses(vec![
        (
            "http://receiver.local/api/epgnow?bRef=ref",
            r#"{"events":[{"title":"Current"}]}"#,
        ),
        (
            "http://receiver.local/api/epgservice?sRef=ref",
            r#"{"events":[{"title":"Schedule"}]}"#,
        ),
    ]);

    for _ in 0..2 {
        assert_eq!(client.epg_now("ref").unwrap()[0].title, "Current");
        assert_eq!(client.service_epg("ref").unwrap()[0].title, "Schedule");
    }
    assert_eq!(requests.lock().unwrap().len(), 2);
}

#[test]
fn clearing_cache_or_changing_receiver_invalidates_services_and_both_epg_endpoints() {
    for change_receiver in [false, true] {
        let next_base = if change_receiver {
            "http://other.local"
        } else {
            "http://receiver.local"
        };
        let next_services = format!("{next_base}/api/getallservices");
        let next_now = format!("{next_base}/api/epgnow?bRef=ref");
        let next_service = format!("{next_base}/api/epgservice?sRef=ref");
        let services = r#"{"result":true,"services":[]}"#;
        let epg = r#"{"events":[]}"#;
        let (client, requests) = client_with_responses(vec![
            ("http://receiver.local/api/getallservices", services),
            ("http://receiver.local/api/epgnow?bRef=ref", epg),
            ("http://receiver.local/api/epgservice?sRef=ref", epg),
            (&next_services, services),
            (&next_now, epg),
            (&next_service, epg),
        ]);
        let load_all = || {
            client.bouquets().unwrap();
            client.epg_now("ref").unwrap();
            client.service_epg("ref").unwrap();
        };

        load_all();
        client.set_base_url(" http://receiver.local/ ");
        load_all();
        assert_eq!(requests.lock().unwrap().len(), 3);

        if change_receiver {
            client.set_base_url(next_base);
        } else {
            client.clear_cache();
        }
        load_all();
        assert_eq!(requests.lock().unwrap().len(), 6);
    }
}

#[test]
fn expired_caches_are_refreshed() {
    let services = r#"{"result":true,"services":[]}"#;
    let epg = r#"{"events":[]}"#;
    let responses = [
        ("http://receiver.local/api/getallservices", services),
        ("http://receiver.local/api/epgnow?bRef=ref", epg),
        ("http://receiver.local/api/epgservice?sRef=ref", epg),
    ];
    let (client, requests) = client_with_responses(responses.repeat(2));
    let load_all = || {
        client.bouquets().unwrap();
        client.epg_now("ref").unwrap();
        client.service_epg("ref").unwrap();
    };
    load_all();
    {
        let mut state = client.state.lock().unwrap();
        state.services_cache.as_mut().unwrap().fetched_at = Instant::now() - SERVICES_CACHE_TTL;
        for cache in state.epg_cache.values_mut() {
            cache.fetched_at = Instant::now() - EPG_CACHE_TTL;
        }
    }
    load_all();
    assert_eq!(requests.lock().unwrap().len(), 6);
}

#[test]
fn enigma2_client_resolve_stream_url_reads_first_playable_playlist_entry() {
    let playlist = "#EXTM3U\n#EXTINF:-1,Test\nhttp://receiver.local:8001/service-ref\n";
    let (client, requests) = client_with_responses(vec![(
        "http://receiver.local/web/stream.m3u?ref=service-ref",
        playlist,
    )]);

    let stream = client.resolve_stream_url("service-ref").unwrap();

    assert_eq!(stream, "http://receiver.local:8001/service-ref");
    assert_eq!(
        requests.lock().unwrap().as_slice(),
        ["http://receiver.local/web/stream.m3u?ref=service-ref"]
    );
}

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
