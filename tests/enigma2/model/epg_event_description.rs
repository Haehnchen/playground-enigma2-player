use enigma2_player_core::enigma2::model::{normalize_epg_text, EpgEvent};

fn event(shortdesc: &str, longdesc: &str) -> EpgEvent {
    EpgEvent {
        id: None,
        begin_timestamp: 0,
        duration_sec: 0,
        title: String::new(),
        shortdesc: shortdesc.to_string(),
        longdesc: longdesc.to_string(),
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
fn normalize_epg_text_preserves_unknown_entities_and_decodes_numeric_entities() {
    assert_eq!(
        normalize_epg_text("Foo &unknown; &#x27;bar&#x27;"),
        format!("Foo &unknown; {}bar{}", char::from(39), char::from(39))
    );
}
