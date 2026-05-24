use crate::support::client_with_responses;

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
