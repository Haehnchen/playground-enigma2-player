use enigma2_player_core::enigma2::model::EpgEvent;

fn event_with_times(begin_timestamp: i64, duration_sec: i64, now_timestamp: i64) -> EpgEvent {
    EpgEvent {
        id: None,
        begin_timestamp,
        duration_sec,
        title: String::new(),
        shortdesc: String::new(),
        longdesc: String::new(),
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
