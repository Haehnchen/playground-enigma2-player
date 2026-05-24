use enigma2_player_core::common::player::session::AudioTrack;
use enigma2_player_core::common::player::stream_settings::audio_tracks_in_menu_order;

fn track(id: i64, selected: bool) -> AudioTrack {
    AudioTrack {
        id,
        label: format!("Audio {id}"),
        selected,
    }
}

#[test]
fn audio_tracks_in_menu_order_moves_selected_track_to_bottom() {
    let tracks = [track(1, false), track(2, true), track(3, false)];

    let ordered: Vec<i64> = audio_tracks_in_menu_order(&tracks)
        .into_iter()
        .map(|track| track.id)
        .collect();

    assert_eq!(ordered, [1, 3, 2]);
}

#[test]
fn audio_tracks_in_menu_order_preserves_non_selected_order() {
    let tracks = [
        track(10, false),
        track(20, false),
        track(30, false),
        track(40, true),
    ];

    let ordered: Vec<i64> = audio_tracks_in_menu_order(&tracks)
        .into_iter()
        .map(|track| track.id)
        .collect();

    assert_eq!(ordered, [10, 20, 30, 40]);
}
