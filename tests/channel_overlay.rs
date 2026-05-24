use enigma2_player_core::ui::channel_overlay::{
    channel_body_layout_width, CHANNEL_OVERLAY_PANEL_HORIZONTAL_MARGIN,
};

#[test]
fn channel_body_layout_width_uses_current_body_width() {
    let root_width = 1280;
    let body_width = root_width - CHANNEL_OVERLAY_PANEL_HORIZONTAL_MARGIN;

    assert_eq!(
        channel_body_layout_width(body_width, root_width),
        body_width
    );
}

#[test]
fn channel_body_layout_width_clamps_stale_fullscreen_width() {
    let root_width = 1280;
    let fullscreen_body_width = 1920 - CHANNEL_OVERLAY_PANEL_HORIZONTAL_MARGIN;

    assert_eq!(
        channel_body_layout_width(fullscreen_body_width, root_width),
        root_width - CHANNEL_OVERLAY_PANEL_HORIZONTAL_MARGIN
    );
}

#[test]
fn channel_body_layout_width_uses_root_width_before_body_allocation() {
    let root_width = 1280;

    assert_eq!(
        channel_body_layout_width(0, root_width),
        root_width - CHANNEL_OVERLAY_PANEL_HORIZONTAL_MARGIN
    );
}
