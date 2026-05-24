const CSS: &str = r#"
window {
  background: #050505;
  font-family: Inter, Cantarell, Sans;
}

.video-shell {
  background: #050505;
}

.video-area {
  background: #050505;
}

.top-controls,
.top-overlay-controls {
  margin: 6px;
}

.icon-button,
.overlay-icon-button {
  min-width: 30px;
  min-height: 28px;
  padding: 3px 7px;
  color: #ffffff;
  background: rgba(0, 0, 0, 0.58);
  border: 1px solid transparent;
  border-radius: 4px;
  box-shadow: none;
  outline-color: transparent;
}

.icon-button:hover,
.overlay-icon-button:hover {
  background: rgba(54, 54, 54, 0.90);
}

.settings-overlay-button {
  background: rgba(0, 0, 0, 0.30);
}

.settings-overlay-button:hover {
  background: rgba(38, 38, 38, 0.62);
}

.close-button:hover {
  background: rgba(170, 36, 36, 0.90);
}

.resize-handle {
  background: transparent;
}

.empty-select-button {
  min-width: 52px;
  min-height: 52px;
  padding: 0;
  border-radius: 4px;
  opacity: 0.60;
  color: rgba(255, 255, 255, 0.62);
  background: transparent;
  border: 1px solid transparent;
  box-shadow: none;
}

.empty-select-button:hover {
  color: rgba(255, 255, 255, 0.78);
  background: rgba(255, 255, 255, 0.06);
}

.player-footer,
.video-footer {
  margin: 0;
  padding: 4px 6px;
  background: rgba(0, 0, 0, 0.58);
  border-radius: 0;
}

.video-footer button,
.video-footer scale {
  color: white;
}

.video-footer button {
  background: rgba(30, 30, 30, 0.82);
  color: white;
  border-color: transparent;
  outline-color: transparent;
  box-shadow: none;
  min-height: 0;
}

.video-footer button:hover {
  background: rgba(54, 54, 54, 0.90);
}

.player-footer .icon-button,
.player-footer button.icon-button,
.player-footer .overlay-icon-button,
.player-footer button.overlay-icon-button {
  min-width: 26px;
  min-height: 24px;
  padding: 4px 5px;
  color: rgba(255, 255, 255, 0.88);
  background: transparent;
  background-image: none;
  border: none;
  border-color: transparent;
  box-shadow: none;
  outline-color: transparent;
}

.player-footer .icon-button:hover,
.player-footer button.icon-button:hover,
.player-footer .overlay-icon-button:hover,
.player-footer button.overlay-icon-button:hover {
  color: white;
  background: rgba(255, 255, 255, 0.14);
  background-image: none;
}

.player-footer .player-refresh-button,
.player-footer button.player-refresh-button {
  min-width: 20px;
  min-height: 22px;
  padding: 3px;
}

.player-footer .stream-settings-button,
.player-footer button.stream-settings-button {
  min-width: 30px;
  min-height: 28px;
  padding: 2px 4px;
}

.stream-settings-popover contents {
  background: rgba(28, 28, 28, 0.98);
  padding: 0;
  margin: 0;
  border: none;
  border-radius: 4px;
  box-shadow: none;
}

.stream-settings-menu {
  background: rgba(28, 28, 28, 0.98);
  padding: 6px;
  min-width: 0;
  margin: 0;
}

.stream-settings-menu-with-audio {
  min-width: 150px;
}

.stream-settings-heading {
  color: rgba(255, 255, 255, 0.66);
  font-size: 11px;
  font-weight: 700;
  margin-top: 1px;
  margin-bottom: 0;
}

.stream-settings-divider {
  background: rgba(255, 255, 255, 0.18);
  min-height: 1px;
  margin: 1px 0;
}

.stream-settings-item {
  background: transparent;
  color: white;
  font-size: 12px;
  border-color: transparent;
  outline-color: transparent;
  box-shadow: none;
  border-radius: 4px;
  margin: 0;
  min-height: 0;
  padding: 3px 6px;
}

.stream-settings-item label {
  margin: 0;
  padding: 0;
}

.stream-settings-item:hover {
  background: rgba(74, 74, 74, 0.98);
}

.stream-settings-item-selected {
  background: rgba(255, 255, 255, 0.16);
}

.stream-selector {
  min-width: 140px;
}

.channel-button,
.stream-dropdown {
  min-width: 140px;
  min-height: 24px;
  padding: 2px 8px;
  color: #ffffff;
  background: rgba(30, 30, 30, 0.82);
  border-color: transparent;
  border-radius: 4px;
  box-shadow: none;
  outline-color: transparent;
}

.channel-button-label,
.stream-button-label {
  color: #ffffff;
  font-size: 13px;
  padding: 0;
}

.stream-info,
.stream-info-labels {
  margin-left: 2px;
  margin-right: 4px;
}

.stream-title,
.stream-title-label {
  color: rgba(255, 255, 255, 0.92);
  font-size: 12px;
  font-weight: 700;
}

.stream-meta,
.stream-metadata-label {
  color: rgba(255, 255, 255, 0.78);
  font-size: 11px;
}

.volume-scale {
  min-width: 112px;
}

.epg-overlay-button {
  margin-left: 2px;
}

.channel-overlay-backdrop {
  background: rgba(0, 0, 0, 0.55);
}

.channel-overlay-panel {
  margin: 42px 18px 48px 18px;
  background: rgba(7, 7, 8, 0.90);
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.18);
}

.channel-overlay-header {
  padding: 7px 9px;
  background: rgba(0, 0, 0, 0.40);
}

.channel-overlay-nav-button {
  color: rgba(255, 255, 255, 0.92);
  background: rgba(255, 255, 255, 0.04);
}

.channel-overlay-nav-button:hover {
  color: #ffffff;
  background: rgba(255, 255, 255, 0.12);
}

.channel-overlay-title {
  color: rgba(255, 255, 255, 0.90);
  font-size: 20px;
  font-weight: 500;
}

.channel-list-scroll {
  border-right: none;
}

.channel-list {
  padding: 6px 4px 10px 8px;
}

.channel-row {
  min-height: 28px;
  padding: 2px 6px;
  border-radius: 2px;
  background: transparent;
  color: rgba(255, 255, 255, 0.86);
}

.channel-row:hover {
  background: rgba(50, 84, 171, 0.56);
  color: white;
}

.channel-row-selected {
  background: rgba(50, 84, 171, 0.92);
  color: white;
}

.channel-row-selected:hover {
  background: rgba(58, 94, 186, 0.96);
}

.channel-row-title {
  font-size: 16px;
}

.channel-progress {
  min-width: 150px;
  min-height: 10px;
  margin-top: 3px;
  margin-bottom: 0;
}

.channel-progress trough,
.detail-progress trough {
  min-height: 4px;
  background: rgba(255, 255, 255, 0.82);
  background-image: none;
}

.channel-progress progress,
.detail-progress progress {
  min-height: 4px;
  background: #f05a28;
  background-image: none;
}

.detail-pane {
  padding: 0 20px 18px 12px;
  background: transparent;
  border-left: none;
  border-radius: 0;
}

.detail-scroll {
  background: transparent;
  border: none;
  box-shadow: none;
}

.detail-scroll viewport {
  background: transparent;
}

.detail-pane label,
.detail-title,
.detail-time,
.detail-description {
  color: rgba(255, 255, 255, 0.88);
}

.detail-title {
  font-size: 24px;
  font-weight: 500;
}

.detail-time {
  font-size: 16px;
}

.detail-progress {
  margin-top: 0;
  margin-bottom: 0;
}

.detail-description {
  font-size: 15px;
  line-height: 1.16;
}

.epg-overlay-panel {
  margin: 46px 18px 48px 18px;
}

.epg-event-list {
  padding: 6px 4px 10px 8px;
}

.epg-event-row {
  min-height: 40px;
  min-width: 0;
  margin: 0;
  padding: 4px 7px;
  border: none;
  border-width: 0;
  border-color: transparent;
  border-radius: 2px;
  background: transparent;
  background-image: none;
  box-shadow: none;
  outline: none;
  outline-width: 0;
  outline-offset: 0;
  outline-color: transparent;
  color: rgba(255, 255, 255, 0.86);
}

.epg-event-row:hover,
.epg-event-row-hover {
  background: rgba(50, 84, 171, 0.56);
  background-image: none;
  border: none;
  border-width: 0;
  box-shadow: none;
  outline: none;
  outline-width: 0;
  outline-offset: 0;
  color: white;
}

.epg-event-row-selected {
  background: rgba(50, 84, 171, 0.92);
  background-image: none;
  border: none;
  border-width: 0;
  box-shadow: none;
  outline: none;
  outline-width: 0;
  outline-offset: 0;
  color: white;
}

.epg-event-row-selected:hover,
.epg-event-row-selected.epg-event-row-hover {
  background: rgba(58, 94, 186, 0.96);
  background-image: none;
  border: none;
  border-width: 0;
  box-shadow: none;
  outline: none;
  outline-width: 0;
  outline-offset: 0;
}

.epg-event-row label {
  min-width: 0;
}

.epg-event-title {
  font-size: 15px;
  min-width: 0;
}

.epg-event-meta {
  color: rgba(255, 255, 255, 0.62);
  font-size: 12px;
  min-width: 0;
}

entry.overlay-search-entry,
searchentry.overlay-search-entry,
.overlay-search-entry {
  min-height: 30px;
  min-width: 250px;
  padding: 4px 8px;
  color: rgba(255, 255, 255, 0.88);
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.14);
  border-radius: 4px;
  box-shadow: none;
}

entry.overlay-search-entry:focus,
searchentry.overlay-search-entry:focus,
.overlay-search-entry:focus {
  background: rgba(255, 255, 255, 0.12);
  border-color: rgba(255, 255, 255, 0.22);
}

entry.overlay-search-entry text,
searchentry.overlay-search-entry text,
.overlay-search-entry text {
  color: rgba(255, 255, 255, 0.88);
  background: transparent;
}


.settings-root {
  background: #141417;
  color: #efeff1;
}

.settings-body,
.settings-content {
  background: #141417;
}

.settings-sidebar {
  padding: 8px;
  background: #1f1f23;
  border-right: 1px solid rgba(255, 255, 255, 0.10);
}

.settings-sidebar-button {
  min-height: 0;
  background: transparent;
  border: none;
  border-radius: 6px;
  padding: 10px 12px;
  color: #efeff1;
  font-size: 14px;
  box-shadow: none;
  outline-color: transparent;
}

.settings-sidebar-button:hover,
.settings-sidebar-button-selected {
  background: #2f2f35;
  color: #ffffff;
}

.settings-page {
  padding: 18px;
  background: #141417;
}

.settings-page-title {
  color: #efeff1;
  font-size: 20px;
  font-weight: 700;
}

.settings-field-label {
  color: rgba(239, 239, 241, 0.70);
  font-size: 13px;
  font-weight: 700;
}

.settings-hint-label,
.settings-status {
  color: rgba(239, 239, 241, 0.64);
  font-size: 13px;
  margin-top: -4px;
}

entry.settings-entry {
  min-height: 28px;
  padding: 3px 8px;
  color: #ffffff;
  background: #222226;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 5px;
  box-shadow: none;
  font-size: 13px;
}

entry.settings-entry:focus {
  color: #ffffff;
  background: #26262c;
  border-color: rgba(255, 255, 255, 0.22);
}

entry.settings-entry text,
entry.settings-entry text selection {
  color: #ffffff;
  background: transparent;
}

.settings-check-row {
  margin-top: 8px;
}

.settings-check {
  color: #efeff1;
  font-size: 14px;
  margin-top: 0;
  margin-bottom: 0;
}

checkbutton.settings-check check {
  min-width: 16px;
  min-height: 16px;
  background: #2b2b2d;
  border: 1px solid rgba(255, 255, 255, 0.24);
  border-radius: 4px;
  box-shadow: none;
}

checkbutton.settings-check check:checked {
  background: #e26a3a;
  border-color: #e26a3a;
  color: #ffffff;
}

.settings-action-row,
.settings-footer {
  background: #141417;
  padding: 0 18px 18px 18px;
}

.settings-primary-button {
  min-width: 72px;
  min-height: 28px;
  background: #3a2b52;
  color: white;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 5px;
  padding: 4px 14px;
  box-shadow: none;
  font-size: 13px;
}

.settings-primary-button:hover {
  background: #4b3670;
}

.settings-about-row label {
  font-size: 13px;
}

"#;

pub fn install() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(CSS);
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
