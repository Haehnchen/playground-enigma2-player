use crate::support::client_with_responses;

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
