use crate::enigma2::api::Enigma2Client;
use crate::enigma2::model::{attach_epg, Bouquet, Channel, EpgEvent};
use gtk::glib;
use gtk::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const CHANNEL_LIST_WIDTH_RATIO: f64 = 0.60;
#[doc(hidden)]
pub const CHANNEL_OVERLAY_PANEL_HORIZONTAL_MARGIN: i32 = 36;
const CHANNEL_ROW_EVENT_COLOR: &str = "#b6b6b6";

pub struct ChannelOverlay {
    root: gtk::Overlay,
    backdrop: gtk::Box,
    panel: gtk::Box,
    bouquet_title: gtk::Label,
    search_entry: gtk::SearchEntry,
    list_box: gtk::Box,
    channel_body: gtk::Box,
    channel_list_scroll: gtk::ScrolledWindow,
    detail_pane: gtk::Box,
    detail_title: gtk::Label,
    detail_time: gtk::Label,
    detail_progress: gtk::ProgressBar,
    detail_description: gtk::Label,
    client: Enigma2Client,
    current_index: Cell<usize>,
    bouquet_count: Cell<usize>,
    current_bouquet: RefCell<Option<Bouquet>>,
    current_service_ref: RefCell<Option<String>>,
    load_generation: Cell<u64>,
    on_activate: Box<dyn Fn(Channel)>,
    self_weak: RefCell<Weak<ChannelOverlay>>,
}

struct LoadedBouquet {
    index: usize,
    bouquet_count: usize,
    bouquet: Bouquet,
    epg_error: Option<String>,
}

enum LoadMessage {
    Bouquet {
        generation: u64,
        result: Result<LoadedBouquet, String>,
    },
}

impl ChannelOverlay {
    pub fn new(
        root: &gtk::Overlay,
        client: Enigma2Client,
        on_activate: impl Fn(Channel) + 'static,
    ) -> Rc<Self> {
        let backdrop = gtk::Box::new(gtk::Orientation::Vertical, 0);
        backdrop.add_css_class("channel-overlay-backdrop");
        backdrop.set_hexpand(true);
        backdrop.set_vexpand(true);
        backdrop.set_visible(false);
        root.add_overlay(&backdrop);

        let panel = gtk::Box::new(gtk::Orientation::Vertical, 0);
        panel.add_css_class("channel-overlay-panel");
        panel.set_halign(gtk::Align::Fill);
        panel.set_valign(gtk::Align::Fill);
        panel.set_hexpand(true);
        panel.set_vexpand(true);
        panel.set_visible(false);
        root.add_overlay(&panel);

        let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        header.add_css_class("channel-overlay-header");
        panel.append(&header);

        let prev_button = icon_button("go-previous-symbolic", "Previous bouquet");
        prev_button.add_css_class("channel-overlay-nav-button");
        let next_button = icon_button("go-next-symbolic", "Next bouquet");
        next_button.add_css_class("channel-overlay-nav-button");
        let bouquet_title = gtk::Label::new(Some("TV"));
        bouquet_title.add_css_class("channel-overlay-title");
        bouquet_title.set_xalign(0.0);
        bouquet_title.set_hexpand(true);
        let search_entry = gtk::SearchEntry::new();
        search_entry.add_css_class("overlay-search-entry");
        search_entry.set_placeholder_text(Some("Filter channels"));
        search_entry.set_width_chars(24);
        let close_button = icon_button("window-close-symbolic", "Close");

        header.append(&prev_button);
        header.append(&bouquet_title);
        header.append(&next_button);
        header.append(&search_entry);
        header.append(&close_button);

        let body = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        body.add_css_class("channel-overlay-body");
        body.set_hexpand(true);
        body.set_vexpand(true);
        panel.append(&body);

        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .build();
        scroller.add_css_class("channel-list-scroll");
        scroller.set_hexpand(false);
        scroller.set_vexpand(true);
        body.append(&scroller);

        let list_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        list_box.add_css_class("channel-list");
        scroller.set_child(Some(&list_box));

        let detail = gtk::Box::new(gtk::Orientation::Vertical, 8);
        detail.add_css_class("detail-pane");
        detail.set_hexpand(true);
        detail.set_vexpand(true);
        body.append(&detail);

        {
            let scroller = scroller.clone();
            let detail = detail.clone();
            let root = root.clone();
            body.connect_notify_local(Some("width"), move |body, _| {
                position_channel_body(body, &scroller, &detail, &root);
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
                    position_channel_body(&body, &scroller, &detail, &root)
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
            root_for_signal.connect_notify_local(Some("width"), move |_, _| {
                reset_channel_body_widths(&scroller, &detail);
                if panel.is_visible() {
                    position_channel_body(&body, &scroller, &detail, &root_for_position);
                }
            });
        }

        let detail_title = gtk::Label::new(Some("Select a channel"));
        detail_title.add_css_class("detail-title");
        detail_title.set_wrap(true);
        detail_title.set_xalign(0.0);
        detail.append(&detail_title);

        let detail_time = gtk::Label::new(None);
        detail_time.add_css_class("detail-time");
        detail_time.set_xalign(0.0);
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
        detail_description.set_xalign(0.0);
        detail_description.set_yalign(0.0);
        detail_description.set_vexpand(false);
        detail_scroller.set_child(Some(&detail_description));

        let overlay = Rc::new(Self {
            root: root.clone(),
            backdrop,
            panel,
            bouquet_title,
            search_entry,
            list_box,
            channel_body: body.clone(),
            channel_list_scroll: scroller.clone(),
            detail_pane: detail.clone(),
            detail_title,
            detail_time,
            detail_progress,
            detail_description,
            client,
            current_index: Cell::new(0),
            bouquet_count: Cell::new(0),
            current_bouquet: RefCell::new(None),
            current_service_ref: RefCell::new(None),
            load_generation: Cell::new(0),
            on_activate: Box::new(on_activate),
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
        {
            let overlay = overlay.clone();
            prev_button.connect_clicked(move |_| overlay.shift_bouquet(-1));
        }
        {
            let overlay = overlay.clone();
            next_button.connect_clicked(move |_| overlay.shift_bouquet(1));
        }
        {
            let overlay_for_changed = overlay.clone();
            overlay
                .search_entry
                .connect_search_changed(move |_| overlay_for_changed.render_channels());
        }

        overlay
    }

    pub fn set_current_service_ref(&self, service_ref: Option<&str>) {
        *self.current_service_ref.borrow_mut() = service_ref
            .map(str::trim)
            .filter(|service_ref| !service_ref.is_empty())
            .map(str::to_string);

        if self.panel.is_visible() {
            self.render_channels();
        }
    }

    pub fn show(self: &Rc<Self>) {
        self.reset_channel_body_widths();
        self.backdrop.set_visible(true);
        self.panel.set_visible(true);
        self.schedule_channel_body_position();
        self.search_entry.grab_focus();

        if self.client.base_url().trim().is_empty() {
            self.invalidate_loads();
            self.clear_list();
            self.bouquet_title.set_text("TV");
            self.detail_title.set_text("No receiver configured");
            self.detail_time.set_text("");
            self.detail_progress.set_fraction(0.0);
            self.detail_description
                .set_text("Open settings to enter the Dreambox / Enigma2 URL.");
            return;
        }

        self.load_current_bouquet();
    }

    pub fn hide(&self) {
        self.invalidate_loads();
        self.backdrop.set_visible(false);
        self.panel.set_visible(false);
        self.clear_list();
        self.reset_channel_body_widths();
        *self.current_bouquet.borrow_mut() = None;
    }

    pub fn is_visible(&self) -> bool {
        self.panel.is_visible()
    }

    fn shift_bouquet(self: &Rc<Self>, direction: isize) {
        let count = self.bouquet_count.get();
        if count == 0 {
            return;
        }

        let current = self.current_index.get() as isize;
        let next = (current + direction).rem_euclid(count as isize) as usize;
        self.current_index.set(next);
        self.load_current_bouquet();
    }

    fn load_current_bouquet(self: &Rc<Self>) {
        let generation = self.next_load_generation();
        let client = self.client.clone();
        let requested_index = self.current_index.get();
        let (sender, receiver) = mpsc::channel();

        self.clear_list();
        *self.current_bouquet.borrow_mut() = None;
        self.bouquet_title.set_text("TV");
        self.detail_title.set_text("Loading channels...");
        self.detail_time.set_text("");
        self.detail_progress.set_fraction(0.0);
        self.detail_description.set_text("");

        thread::spawn(move || {
            let result = load_bouquet_by_index(&client, requested_index);
            let _ = sender.send(LoadMessage::Bouquet { generation, result });
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
            LoadMessage::Bouquet { generation, result } => {
                if self.is_stale(generation) {
                    return;
                }
                match result {
                    Ok(loaded) => self.apply_loaded_bouquet(loaded),
                    Err(err) => self.show_detail_message("Loading failed", &err),
                }
            }
        }
    }

    fn apply_loaded_bouquet(&self, loaded: LoadedBouquet) {
        self.current_index.set(loaded.index);
        self.bouquet_count.set(loaded.bouquet_count);
        self.bouquet_title
            .set_text(&format!("TV - {}", loaded.bouquet.name.trim()));
        *self.current_bouquet.borrow_mut() = Some(loaded.bouquet);
        self.position_channel_body();
        self.render_channels();
        self.position_channel_body();
        if let Some(err) = loaded.epg_error {
            self.show_detail_message("EPG loading failed", &err);
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

    fn render_channels(&self) {
        self.clear_list();
        let Some(bouquet) = self.current_bouquet.borrow().clone() else {
            return;
        };
        let filter = self.search_entry.text().to_string().to_lowercase();
        let current_service_ref = self.current_service_ref.borrow().clone();
        let mut detail_channel: Option<Channel> = None;

        for channel in bouquet.channels {
            if !filter.is_empty()
                && !channel.name.to_lowercase().contains(&filter)
                && !channel
                    .epg
                    .as_ref()
                    .map(|event| event.title.to_lowercase().contains(&filter))
                    .unwrap_or(false)
            {
                continue;
            }

            let selected = current_service_ref
                .as_deref()
                .is_some_and(|current| same_service_ref(current, &channel.service_ref));
            if detail_channel.is_none() || selected {
                detail_channel = Some(channel.clone());
            }
            self.list_box
                .append(&self.create_channel_row(channel, selected));
        }

        if let Some(channel) = detail_channel {
            self.show_detail(&channel);
        } else {
            self.detail_title.set_text("No channels match");
            self.detail_time.set_text("");
            self.detail_progress.set_fraction(0.0);
            self.detail_description.set_text("");
        }
    }

    fn create_channel_row(&self, channel: Channel, selected: bool) -> gtk::Button {
        let button = gtk::Button::new();
        button.add_css_class("channel-row");
        if selected {
            button.add_css_class("channel-row-selected");
        }
        button.set_has_frame(false);
        button.set_halign(gtk::Align::Fill);

        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        row.set_valign(gtk::Align::Center);
        let label = gtk::Label::new(None);
        label.add_css_class("channel-row-title");
        label.set_markup(&channel_row_markup(&channel));
        label.set_xalign(0.0);
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        label.set_single_line_mode(true);
        label.set_hexpand(true);
        row.append(&label);

        let progress = gtk::ProgressBar::new();
        progress.add_css_class("channel-progress");
        progress.set_valign(gtk::Align::Center);
        progress.set_size_request(150, 10);
        progress.set_fraction(channel.epg.as_ref().map(EpgEvent::progress).unwrap_or(0.0));
        row.append(&progress);
        button.set_child(Some(&row));

        let hover_channel = channel.clone();
        let self_for_hover = self.self_weak.borrow().clone();
        let motion = gtk::EventControllerMotion::new();
        motion.connect_enter(move |_, _, _| {
            if let Some(overlay) = self_for_hover.upgrade() {
                overlay.show_detail(&hover_channel);
            }
        });
        button.add_controller(motion);

        let click_channel = channel.clone();
        let self_for_click = self.self_weak.borrow().clone();
        button.connect_clicked(move |_| {
            if let Some(overlay) = self_for_click.upgrade() {
                (overlay.on_activate)(click_channel.clone());
                overlay.hide();
            }
        });

        button
    }

    fn show_detail(&self, channel: &Channel) {
        if let Some(event) = &channel.epg {
            self.detail_title.set_text(&event.title);
            self.detail_time
                .set_text(&format!("{}  {}", channel.name, event.time_range()));
            self.detail_progress.set_fraction(event.progress());
            self.detail_description.set_text(&event.description());
        } else {
            self.detail_title.set_text(&channel.name);
            self.detail_time.set_text("No EPG event available");
            self.detail_progress.set_fraction(0.0);
            self.detail_description.set_text("");
        }
    }

    fn show_detail_message(&self, title: &str, description: &str) {
        self.detail_title.set_text(title);
        self.detail_time.set_text("");
        self.detail_progress.set_fraction(0.0);
        self.detail_description.set_text(description);
    }

    fn clear_list(&self) {
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
    }

    fn schedule_channel_body_position(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        glib::idle_add_local_once(move || {
            if let Some(overlay) = weak.upgrade() {
                overlay.position_channel_body();
            }
        });
    }

    fn position_channel_body(&self) {
        position_channel_body_for_width(
            &self.channel_list_scroll,
            &self.detail_pane,
            channel_body_layout_width(self.channel_body.width(), self.root.width()),
        );
    }

    fn reset_channel_body_widths(&self) {
        reset_channel_body_widths(&self.channel_list_scroll, &self.detail_pane);
    }
}

fn same_service_ref(left: &str, right: &str) -> bool {
    left.trim() == right.trim()
}

fn load_bouquet_by_index(
    client: &Enigma2Client,
    requested_index: usize,
) -> Result<LoadedBouquet, String> {
    let bouquets = client.bouquets().map_err(|err| err.to_string())?;
    if bouquets.is_empty() {
        return Err("No bouquets found".to_string());
    }

    let index = requested_index.min(bouquets.len() - 1);
    let mut loaded = load_bouquet_epg(client, bouquets[index].clone())?;
    loaded.index = index;
    loaded.bouquet_count = bouquets.len();
    Ok(loaded)
}

fn load_bouquet_epg(client: &Enigma2Client, bouquet: Bouquet) -> Result<LoadedBouquet, String> {
    match client.epg_now(&bouquet.service_ref) {
        Ok(events) => Ok(LoadedBouquet {
            index: 0,
            bouquet_count: 0,
            bouquet: attach_epg(bouquet, &events),
            epg_error: None,
        }),
        Err(err) => Ok(LoadedBouquet {
            index: 0,
            bouquet_count: 0,
            bouquet,
            epg_error: Some(err.to_string()),
        }),
    }
}

fn channel_row_markup(channel: &Channel) -> String {
    let channel_text = glib::markup_escape_text(&format!("{} {}", channel.position, channel.name));
    let event_title = channel
        .epg
        .as_ref()
        .map(|event| event.title.trim())
        .filter(|title| !title.is_empty())
        .unwrap_or("");
    if event_title.is_empty() {
        channel_text.to_string()
    } else {
        let event_text = glib::markup_escape_text(event_title);
        format!(
            "{}  <span foreground=\"{}\">{}</span>",
            channel_text, CHANNEL_ROW_EVENT_COLOR, event_text
        )
    }
}

fn icon_button(icon: &str, tooltip: &str) -> gtk::Button {
    let button = gtk::Button::new();
    button.add_css_class("icon-button");
    button.set_tooltip_text(Some(tooltip));
    button.set_child(Some(&gtk::Image::from_icon_name(icon)));
    button
}

fn position_channel_body(
    body: &gtk::Box,
    list_scroll: &gtk::ScrolledWindow,
    detail_pane: &gtk::Box,
    root: &gtk::Overlay,
) {
    position_channel_body_for_width(
        list_scroll,
        detail_pane,
        channel_body_layout_width(body.width(), root.width()),
    );
}

fn reset_channel_body_widths(list_scroll: &gtk::ScrolledWindow, detail_pane: &gtk::Box) {
    list_scroll.set_size_request(-1, -1);
    detail_pane.set_size_request(-1, -1);
}

fn position_channel_body_for_width(
    list_scroll: &gtk::ScrolledWindow,
    detail_pane: &gtk::Box,
    width: i32,
) {
    // Re-apply the 60/40 split after async rows change the body allocation.
    if width > 0 {
        let list_width = (width as f64 * CHANNEL_LIST_WIDTH_RATIO).round() as i32;
        list_scroll.set_size_request(list_width, -1);
        detail_pane.set_size_request((width - list_width).max(1), -1);
    }
}

#[doc(hidden)]
pub fn channel_body_layout_width(body_width: i32, root_width: i32) -> i32 {
    if root_width <= 0 {
        return body_width;
    }

    let root_available_width = (root_width - CHANNEL_OVERLAY_PANEL_HORIZONTAL_MARGIN).max(1);
    if body_width <= 0 {
        return root_available_width;
    }

    body_width.min(root_available_width)
}
