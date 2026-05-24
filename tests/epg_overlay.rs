use enigma2_player_core::enigma2::model::EpgEvent;
use enigma2_player_core::ui::epg_overlay::{
    epg_body_layout_width, epg_panel_width, events_from_current_onward, progress_is_visible,
    EPG_MAX_EVENTS, EPG_OVERLAY_MAX_WIDTH,
};

#[test]
fn events_from_current_onward_starts_with_current_event() {
    let events = events_from_current_onward(vec![
        event("Later", 240, 60, 150),
        event("Current", 100, 80, 150),
        event("Past", 10, 50, 150),
    ]);

    let titles = events
        .iter()
        .map(|event| event.title.as_str())
        .collect::<Vec<_>>();
    assert_eq!(titles, ["Current", "Later"]);
}

#[test]
fn events_from_current_onward_uses_next_event_when_current_is_missing() {
    let events = events_from_current_onward(vec![
        event("Past", 10, 50, 150),
        event("Next", 180, 60, 150),
        event("Later", 240, 60, 150),
    ]);

    let titles = events
        .iter()
        .map(|event| event.title.as_str())
        .collect::<Vec<_>>();
    assert_eq!(titles, ["Next", "Later"]);
}

#[test]
fn progress_is_visible_only_for_positive_fraction() {
    assert!(!progress_is_visible(0.0));
    assert!(!progress_is_visible(-0.1));
    assert!(progress_is_visible(0.01));
}

#[test]
fn epg_panel_width_caps_at_initial_overlay_width() {
    assert_eq!(epg_panel_width(1280), EPG_OVERLAY_MAX_WIDTH);
    assert_eq!(epg_panel_width(1920), EPG_OVERLAY_MAX_WIDTH);
}

#[test]
fn epg_panel_width_shrinks_with_smaller_windows() {
    assert_eq!(epg_panel_width(800), 764);
    assert_eq!(epg_panel_width(20), 1);
}

#[test]
fn epg_body_layout_width_uses_panel_width_before_body_allocation() {
    assert_eq!(epg_body_layout_width(0, 1920), EPG_OVERLAY_MAX_WIDTH);
    assert_eq!(epg_body_layout_width(0, 800), 764);
}

#[test]
fn events_from_current_onward_limits_events() {
    let events = (0..80)
        .map(|index| event(&format!("Event {index}"), index * 60, 60, 1))
        .collect::<Vec<_>>();

    assert_eq!(events_from_current_onward(events).len(), EPG_MAX_EVENTS);
}

fn event(title: &str, begin_timestamp: i64, duration_sec: i64, now_timestamp: i64) -> EpgEvent {
    EpgEvent {
        id: None,
        begin_timestamp,
        duration_sec,
        title: title.to_string(),
        shortdesc: String::new(),
        longdesc: String::new(),
        sref: "service-ref".to_string(),
        sname: "Das Erste HD".to_string(),
        now_timestamp,
        remaining: 0,
    }
}
