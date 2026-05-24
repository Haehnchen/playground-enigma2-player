use crate::enigma2::api::Enigma2Client;
use crate::enigma2::model::{Channel, EpgEvent};
use gtk::glib;
use gtk::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const EPG_LIST_WIDTH_RATIO: f64 = 0.48;
#[doc(hidden)]
pub const EPG_MAX_EVENTS: usize = 50;
const EPG_OVERLAY_HORIZONTAL_MARGIN: i32 = 36;
#[doc(hidden)]
pub const EPG_OVERLAY_MAX_WIDTH: i32 = 1244;
const EPG_ROW_TIME_COLOR: &str = "#b6b6b6";
const EPG_DETAIL_TEXT_MAX_CHARS: i32 = 86;
#[doc(hidden)]
pub const EPG_DETAIL_TITLE_MAX_LINES: i32 = 2;
#[doc(hidden)]
pub const EPG_DETAIL_TITLE_REQUEST_CHARS: i32 = 1;
#[doc(hidden)]
pub const EPG_HOVER_DETAIL_DELAY_MS: u64 = 50;

pub struct EpgOverlay {
    root: gtk::Overlay,
    backdrop: gtk::Box,
    panel: gtk::Box,
    title: gtk::Label,
    list_box: gtk::Box,
    event_body: gtk::Box,
    event_list_scroll: gtk::ScrolledWindow,
    detail_pane: gtk::Box,
    detail_title: gtk::Label,
    detail_time: gtk::Label,
    detail_progress: gtk::ProgressBar,
    detail_description: gtk::Label,
    client: Enigma2Client,
    current_channel: RefCell<Option<Channel>>,
    hover_detail_source: RefCell<Option<glib::SourceId>>,
    load_generation: Cell<u64>,
    self_weak: RefCell<Weak<EpgOverlay>>,
}

struct LoadedEvents {
    channel: Channel,
    events: Vec<EpgEvent>,
}

enum LoadMessage {
    Events {
        generation: u64,
        result: Result<LoadedEvents, String>,
    },
}

impl EpgOverlay {
    pub fn new(root: &gtk::Overlay, client: Enigma2Client) -> Rc<Self> {
        let backdrop = gtk::Box::new(gtk::Orientation::Vertical, 0);
        backdrop.add_css_class("channel-overlay-backdrop");
        backdrop.set_hexpand(true);
        backdrop.set_vexpand(true);
        backdrop.set_visible(false);
        root.add_overlay(&backdrop);

        let panel = gtk::Box::new(gtk::Orientation::Vertical, 0);
        panel.add_css_class("channel-overlay-panel");
        panel.add_css_class("epg-overlay-panel");
        panel.set_halign(gtk::Align::Center);
        panel.set_valign(gtk::Align::Fill);
        panel.set_hexpand(false);
        panel.set_vexpand(true);
        set_epg_panel_width(&panel, root.width());
        panel.set_visible(false);
        root.add_overlay(&panel);

        let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        header.add_css_class("channel-overlay-header");
        panel.append(&header);

        let title = gtk::Label::new(Some("EPG"));
        title.add_css_class("channel-overlay-title");
        title.add_css_class("epg-overlay-title");
        title.set_xalign(0.0);
        title.set_hexpand(true);
        let close_button = icon_button("window-close-symbolic", "Close");

        header.append(&title);
        header.append(&close_button);

        let body = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        body.add_css_class("channel-overlay-body");
        body.add_css_class("epg-overlay-body");
        body.set_hexpand(true);
        body.set_vexpand(true);
        panel.append(&body);

        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .build();
        scroller.add_css_class("epg-list-scroll");
        scroller.set_hexpand(false);
        scroller.set_vexpand(true);
        body.append(&scroller);

        let list_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        list_box.add_css_class("epg-event-list");
        scroller.set_child(Some(&list_box));

        let detail = gtk::Box::new(gtk::Orientation::Vertical, 8);
        detail.add_css_class("detail-pane");
        detail.add_css_class("epg-detail-pane");
        detail.set_hexpand(true);
        detail.set_vexpand(true);
        body.append(&detail);

        {
            let scroller = scroller.clone();
            let detail = detail.clone();
            let root = root.clone();
            body.connect_notify_local(Some("width"), move |body, _| {
                position_epg_body(body, &scroller, &detail, &root);
            });
        }
        {
            let body_for_map = body.clone();
            let scroller = scroller.clone();
            let detail = detail.clone();
            let root = root.clone();
            body.connect_map(move |_| {
                let body = body_for_map.clone();
                let scroller = scroller.clone();
                let detail = detail.clone();
                let root = root.clone();
                glib::idle_add_local_once(move || {
                    position_epg_body(&body, &scroller, &detail, &root)
                });
            });
        }
        {
            let body = body.clone();
            let scroller = scroller.clone();
            let detail = detail.clone();
            let panel = panel.clone();
            let root_for_signal = root.clone();
            let root_for_position = root.clone();
            root_for_signal.connect_notify_local(Some("width"), move |root, _| {
                reset_epg_body_widths(&scroller, &detail);
                set_epg_panel_width(&panel, root.width());
                if panel.is_visible() {
                    position_epg_body(&body, &scroller, &detail, &root_for_position);
                }
            });
        }

        let detail_title = gtk::Label::new(Some("Select a programme"));
        detail_title.add_css_class("detail-title");
        detail_title.set_wrap(true);
        detail_title.set_wrap_mode(gtk::pango::WrapMode::Char);
        detail_title.set_natural_wrap_mode(gtk::NaturalWrapMode::Inherit);
        detail_title.set_ellipsize(gtk::pango::EllipsizeMode::End);
        detail_title.set_lines(EPG_DETAIL_TITLE_MAX_LINES);
        detail_title.set_width_chars(EPG_DETAIL_TITLE_REQUEST_CHARS);
        detail_title.set_max_width_chars(EPG_DETAIL_TITLE_REQUEST_CHARS);
        detail_title.set_xalign(0.0);
        detail_title.set_halign(gtk::Align::Fill);
        detail_title.set_overflow(gtk::Overflow::Hidden);
        detail_title.set_hexpand(true);
        detail.append(&detail_title);

        let detail_time = gtk::Label::new(None);
        detail_time.add_css_class("detail-time");
        detail_time.set_xalign(0.0);
        detail_time.set_ellipsize(gtk::pango::EllipsizeMode::End);
        detail_time.set_max_width_chars(EPG_DETAIL_TEXT_MAX_CHARS);
        detail.append(&detail_time);

        let detail_progress = gtk::ProgressBar::new();
        detail_progress.add_css_class("detail-progress");
        detail.append(&detail_progress);

        let detail_scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .build();
        detail_scroller.add_css_class("detail-scroll");
        detail_scroller.set_hexpand(true);
        detail_scroller.set_vexpand(true);
        detail_scroller.set_propagate_natural_height(false);
        detail.append(&detail_scroller);

        let detail_description = gtk::Label::new(None);
        detail_description.add_css_class("detail-description");
        detail_description.set_wrap(true);
        detail_description.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        detail_description.set_max_width_chars(EPG_DETAIL_TEXT_MAX_CHARS);
        detail_description.set_xalign(0.0);
        detail_description.set_yalign(0.0);
        detail_description.set_vexpand(false);
        detail_scroller.set_child(Some(&detail_description));

        let overlay = Rc::new(Self {
            root: root.clone(),
            backdrop,
            panel,
            title,
            list_box,
            event_body: body.clone(),
            event_list_scroll: scroller.clone(),
            detail_pane: detail.clone(),
            detail_title,
            detail_time,
            detail_progress,
            detail_description,
            client,
            current_channel: RefCell::new(None),
            hover_detail_source: RefCell::new(None),
            load_generation: Cell::new(0),
            self_weak: RefCell::new(Weak::new()),
        });
        *overlay.self_weak.borrow_mut() = Rc::downgrade(&overlay);

        {
            let click = gtk::GestureClick::new();
            let overlay_for_click = overlay.clone();
            click.connect_pressed(move |_, _, _, _| overlay_for_click.hide());
            overlay.backdrop.add_controller(click);
        }
        {
            let overlay = overlay.clone();
            close_button.connect_clicked(move |_| overlay.hide());
        }

        overlay
    }

    pub fn show_for_channel(self: &Rc<Self>, channel: Channel) {
        self.cancel_pending_hover_detail();
        self.reset_epg_body_widths();
        self.set_panel_width();
        self.backdrop.set_visible(true);
        self.panel.set_visible(true);
        self.schedule_epg_body_position();
        self.title
            .set_text(&format!("EPG - {}", channel.name.trim()));
        *self.current_channel.borrow_mut() = Some(channel.clone());

        if self.client.base_url().trim().is_empty() {
            self.invalidate_loads();
            self.clear_list();
            self.detail_title.set_text("No receiver configured");
            self.detail_time.set_text("");
            self.clear_detail_progress();
            self.detail_description
                .set_text("Open settings to enter the Dreambox / Enigma2 URL.");
            return;
        }

        self.load_channel_epg(channel);
    }

    pub fn hide(&self) {
        self.cancel_pending_hover_detail();
        self.invalidate_loads();
        self.backdrop.set_visible(false);
        self.panel.set_visible(false);
        self.clear_list();
        self.reset_epg_body_widths();
        *self.current_channel.borrow_mut() = None;
    }

    pub fn is_visible(&self) -> bool {
        self.panel.is_visible()
    }

    fn load_channel_epg(self: &Rc<Self>, channel: Channel) {
        let generation = self.next_load_generation();
        let client = self.client.clone();
        let (sender, receiver) = mpsc::channel();

        self.clear_list();
        self.detail_title.set_text("Loading EPG...");
        self.detail_time.set_text("");
        self.clear_detail_progress();
        self.detail_description.set_text("");

        thread::spawn(move || {
            let result = client
                .service_epg(&channel.service_ref)
                .map(|events| LoadedEvents { channel, events })
                .map_err(|err| err.to_string());
            let _ = sender.send(LoadMessage::Events { generation, result });
        });
        self.poll_load(receiver);
    }

    fn poll_load(self: &Rc<Self>, receiver: mpsc::Receiver<LoadMessage>) {
        let weak = Rc::downgrade(self);
        glib::timeout_add_local(Duration::from_millis(16), move || {
            match receiver.try_recv() {
                Ok(message) => {
                    if let Some(overlay) = weak.upgrade() {
                        overlay.apply_load_message(message);
                    }
                    glib::ControlFlow::Break
                }
                Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
            }
        });
    }

    fn apply_load_message(&self, message: LoadMessage) {
        match message {
            LoadMessage::Events { generation, result } => {
                if self.is_stale(generation) {
                    return;
                }
                match result {
                    Ok(loaded) => self.apply_loaded_events(loaded),
                    Err(err) => self.show_detail_message("EPG loading failed", &err),
                }
            }
        }
    }

    fn apply_loaded_events(&self, loaded: LoadedEvents) {
        self.title
            .set_text(&format!("EPG - {}", loaded.channel.name.trim()));
        *self.current_channel.borrow_mut() = Some(loaded.channel);

        let events = events_from_current_onward(loaded.events);
        if events.is_empty() {
            self.clear_list();
            self.show_detail_message("No EPG events available", "");
            return;
        }

        self.render_events(events);
        self.position_epg_body();
    }

    fn render_events(&self, events: Vec<EpgEvent>) {
        self.clear_list();
        let selected = events.first().cloned();

        for event in events {
            self.list_box.append(&self.create_event_row(event));
        }

        if let Some(event) = selected {
            self.show_detail(&event);
        }
    }

    fn create_event_row(&self, event: EpgEvent) -> gtk::Button {
        let button = gtk::Button::new();
        button.add_css_class("epg-event-row");
        button.set_has_frame(false);
        button.set_halign(gtk::Align::Fill);

        let row = gtk::Box::new(gtk::Orientation::Vertical, 2);
        row.set_valign(gtk::Align::Center);
        row.set_hexpand(true);

        let title = gtk::Label::new(None);
        title.add_css_class("epg-event-title");
        title.set_markup(&event_row_markup(&event));
        title.set_xalign(0.0);
        title.set_ellipsize(gtk::pango::EllipsizeMode::End);
        title.set_single_line_mode(true);
        title.set_hexpand(true);
        row.append(&title);

        let meta_text = event_row_meta_text(&event);
        let meta = gtk::Label::new(Some(&meta_text));
        meta.add_css_class("epg-event-meta");
        meta.set_xalign(0.0);
        meta.set_ellipsize(gtk::pango::EllipsizeMode::End);
        meta.set_single_line_mode(true);
        meta.set_hexpand(true);
        row.append(&meta);

        button.set_child(Some(&row));

        let hover_event = event.clone();
        let self_for_hover = self.self_weak.borrow().clone();
        let motion = gtk::EventControllerMotion::new();
        motion.connect_enter(move |_, _, _| {
            if let Some(overlay) = self_for_hover.upgrade() {
                overlay.schedule_hover_detail(hover_event.clone());
            }
        });
        let self_for_leave = self.self_weak.borrow().clone();
        motion.connect_leave(move |_| {
            if let Some(overlay) = self_for_leave.upgrade() {
                overlay.cancel_pending_hover_detail();
            }
        });
        button.add_controller(motion);

        button
    }

    fn show_detail(&self, event: &EpgEvent) {
        let title = event
            .title
            .trim()
            .is_empty()
            .then_some("Untitled EPG event")
            .unwrap_or(event.title.trim());
        self.detail_title.set_text(title);

        let channel_name = if event.sname.trim().is_empty() {
            self.current_channel
                .borrow()
                .as_ref()
                .map(|channel| channel.name.clone())
                .unwrap_or_default()
        } else {
            event.sname.clone()
        };
        if channel_name.trim().is_empty() {
            self.detail_time.set_text(&event.time_range());
        } else {
            self.detail_time
                .set_text(&format!("{}  {}", channel_name.trim(), event.time_range()));
        }
        self.set_detail_progress(event.progress());
        self.detail_description.set_text(&event.description());
    }

    fn schedule_hover_detail(&self, event: EpgEvent) {
        self.cancel_pending_hover_detail();

        let self_weak = self.self_weak.borrow().clone();
        let source = glib::timeout_add_local_once(
            Duration::from_millis(EPG_HOVER_DETAIL_DELAY_MS),
            move || {
                if let Some(overlay) = self_weak.upgrade() {
                    overlay.hover_detail_source.borrow_mut().take();
                    overlay.show_detail(&event);
                }
            },
        );
        *self.hover_detail_source.borrow_mut() = Some(source);
    }

    fn cancel_pending_hover_detail(&self) {
        if let Some(source) = self.hover_detail_source.borrow_mut().take() {
            source.remove();
        }
    }

    fn show_detail_message(&self, title: &str, description: &str) {
        self.detail_title.set_text(title);
        self.detail_time.set_text("");
        self.clear_detail_progress();
        self.detail_description.set_text(description);
    }

    fn set_detail_progress(&self, fraction: f64) {
        self.detail_progress.set_fraction(fraction);
        self.detail_progress
            .set_visible(progress_is_visible(fraction));
    }

    fn clear_detail_progress(&self) {
        self.detail_progress.set_fraction(0.0);
        self.detail_progress.set_visible(false);
    }

    fn clear_list(&self) {
        self.cancel_pending_hover_detail();
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
    }

    fn next_load_generation(&self) -> u64 {
        let next = self.load_generation.get().wrapping_add(1);
        self.load_generation.set(next);
        next
    }

    fn invalidate_loads(&self) {
        self.load_generation
            .set(self.load_generation.get().wrapping_add(1));
    }

    fn is_stale(&self, generation: u64) -> bool {
        generation != self.load_generation.get() || !self.panel.is_visible()
    }

    fn schedule_epg_body_position(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        glib::idle_add_local_once(move || {
            if let Some(overlay) = weak.upgrade() {
                overlay.position_epg_body();
            }
        });
    }

    fn position_epg_body(&self) {
        position_epg_body_for_width(
            &self.event_list_scroll,
            &self.detail_pane,
            epg_body_layout_width(self.event_body.width(), self.root.width()),
        );
    }

    fn reset_epg_body_widths(&self) {
        reset_epg_body_widths(&self.event_list_scroll, &self.detail_pane);
    }

    fn set_panel_width(&self) {
        set_epg_panel_width(&self.panel, self.root.width());
    }
}

#[doc(hidden)]
pub fn events_from_current_onward(mut events: Vec<EpgEvent>) -> Vec<EpgEvent> {
    events.sort_by_key(|event| (event.begin_timestamp.max(0), event.id.unwrap_or(0)));
    let now = events
        .iter()
        .find_map(|event| (event.now_timestamp > 0).then_some(event.now_timestamp))
        .unwrap_or_else(current_unix_timestamp);

    if let Some(index) = events.iter().position(|event| event_is_current(event, now)) {
        events.drain(..index);
        events.truncate(EPG_MAX_EVENTS);
        return events;
    }

    let mut upcoming = events
        .iter()
        .filter(|event| event.begin_timestamp <= 0 || event.end_timestamp() >= now)
        .cloned()
        .collect::<Vec<_>>();
    if upcoming.is_empty() {
        events.truncate(EPG_MAX_EVENTS);
        events
    } else {
        upcoming.truncate(EPG_MAX_EVENTS);
        upcoming
    }
}

fn event_is_current(event: &EpgEvent, now: i64) -> bool {
    event.begin_timestamp > 0
        && event.duration_sec > 0
        && event.begin_timestamp <= now
        && event.end_timestamp() > now
}

#[doc(hidden)]
pub fn progress_is_visible(fraction: f64) -> bool {
    fraction > 0.0
}

fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn event_row_markup(event: &EpgEvent) -> String {
    let time_text = glib::markup_escape_text(&event.time_range());
    let title = event
        .title
        .trim()
        .is_empty()
        .then_some("Untitled EPG event")
        .unwrap_or(event.title.trim());
    let title_text = glib::markup_escape_text(title);
    format!(
        "<span foreground=\"{}\">{}</span>  {}",
        EPG_ROW_TIME_COLOR, time_text, title_text
    )
}

#[doc(hidden)]
pub fn event_row_meta_text(event: &EpgEvent) -> String {
    let genre = event.genre.trim();
    if !genre.is_empty() {
        return genre.to_string();
    }

    let title = event.title.trim();
    first_meaningful_line(&event.description(), title)
        .map(str::to_string)
        .unwrap_or_else(|| "-".to_string())
}

fn first_meaningful_line<'a>(value: &'a str, title: &str) -> Option<&'a str> {
    value
        .lines()
        .map(str::trim)
        .filter(|line| title.is_empty() || *line != title)
        .find(|line| !line.is_empty())
}

fn icon_button(icon: &str, tooltip: &str) -> gtk::Button {
    let button = gtk::Button::new();
    button.add_css_class("icon-button");
    button.set_tooltip_text(Some(tooltip));
    button.set_child(Some(&gtk::Image::from_icon_name(icon)));
    button
}

fn position_epg_body(
    body: &gtk::Box,
    list_scroll: &gtk::ScrolledWindow,
    detail_pane: &gtk::Box,
    root: &gtk::Overlay,
) {
    position_epg_body_for_width(
        list_scroll,
        detail_pane,
        epg_body_layout_width(body.width(), root.width()),
    );
}

fn set_epg_panel_width(panel: &gtk::Box, root_width: i32) {
    panel.set_size_request(epg_panel_width(root_width), -1);
}

#[doc(hidden)]
pub fn epg_panel_width(root_width: i32) -> i32 {
    let available_width = if root_width > 0 {
        root_width.saturating_sub(EPG_OVERLAY_HORIZONTAL_MARGIN)
    } else {
        EPG_OVERLAY_MAX_WIDTH
    };
    available_width.clamp(1, EPG_OVERLAY_MAX_WIDTH)
}

fn reset_epg_body_widths(list_scroll: &gtk::ScrolledWindow, detail_pane: &gtk::Box) {
    list_scroll.set_size_request(-1, -1);
    detail_pane.set_size_request(-1, -1);
}

fn position_epg_body_for_width(
    list_scroll: &gtk::ScrolledWindow,
    detail_pane: &gtk::Box,
    width: i32,
) {
    if width > 0 {
        let list_width = (width as f64 * EPG_LIST_WIDTH_RATIO).round() as i32;
        list_scroll.set_size_request(list_width, -1);
        detail_pane.set_size_request((width - list_width).max(1), -1);
    }
}

#[doc(hidden)]
pub fn epg_body_layout_width(body_width: i32, root_width: i32) -> i32 {
    let panel_width = epg_panel_width(root_width);
    if body_width <= 0 {
        panel_width
    } else {
        body_width.min(panel_width)
    }
}
