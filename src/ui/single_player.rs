use crate::common::player::icons::{self, WindowIcon};
use crate::common::player::render::MpvVideo;
use crate::common::player::session::PlayerSession;
use crate::common::player::stream_settings::StreamSettingsPopover;
use crate::enigma2::api::Enigma2Client;
use crate::enigma2::model::Channel;
use crate::settings::store::AppSettings;
use crate::ui::channel_overlay::ChannelOverlay;
use crate::ui::epg_overlay::EpgOverlay;
use gtk::gdk::prelude::*;
use gtk::glib;
use gtk::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

const VOLUME_SCROLL_STEP: f64 = 5.0;
const VOLUME_MIN: f64 = 0.0;
const VOLUME_MAX: f64 = 130.0;
const PLAYER_OVERLAY_HIDE_DELAY_MS: u64 = 1_800;
const MOTION_EPSILON: f64 = 0.5;

pub struct SinglePlayer {
    root: gtk::Overlay,
    session: Rc<RefCell<PlayerSession>>,
    video: Rc<MpvVideo>,
    client: Enigma2Client,
    channel_overlay: RefCell<Option<Rc<ChannelOverlay>>>,
    epg_overlay: RefCell<Option<Rc<EpgOverlay>>>,
    current_service_ref: RefCell<Option<String>>,
    current_channel: RefCell<Option<Channel>>,
    self_weak: RefCell<Weak<SinglePlayer>>,
    channel_button: gtk::Button,
    channel_label: gtk::Label,
    title_label: gtk::Label,
    meta_label: gtk::Label,
    empty_button: gtk::Button,
    fullscreen_button: gtk::Button,
    fullscreen_active: Cell<bool>,
    top_left_controls: RefCell<Option<gtk::Box>>,
    top_right_controls: RefCell<Option<gtk::Box>>,
    footer_controls: RefCell<Option<gtk::Box>>,
    overlay_hide_source: RefCell<Option<glib::SourceId>>,
    motion_has_last: Cell<bool>,
    motion_last_x: Cell<f64>,
    motion_last_y: Cell<f64>,
    move_pressed: Cell<bool>,
    move_press_x: Cell<f64>,
    move_press_y: Cell<f64>,
    mute_button: gtk::Button,
    epg_button: gtk::Button,
    stream_settings_button: gtk::Button,
    stream_settings_popover: StreamSettingsPopover,
    volume_scale: gtk::Scale,
}

impl SinglePlayer {
    pub fn new(
        settings: Rc<RefCell<AppSettings>>,
        client: Enigma2Client,
        open_settings: impl Fn() + 'static,
        on_minimize: impl Fn() + 'static,
        on_fullscreen: impl Fn() -> bool + 'static,
        on_close: impl Fn() + 'static,
    ) -> Rc<Self> {
        let session = Rc::new(RefCell::new(PlayerSession::new(settings.borrow().hwdec)));
        let video = MpvVideo::new(session.clone());
        let root = gtk::Overlay::new();
        root.add_css_class("video-shell");
        root.set_hexpand(true);
        root.set_vexpand(true);
        root.set_child(Some(&video.widget()));

        let channel_button = gtk::Button::new();
        channel_button.add_css_class("channel-button");
        channel_button.add_css_class("stream-dropdown");
        channel_button.set_halign(gtk::Align::Fill);
        channel_button.set_hexpand(true);
        let channel_label = gtk::Label::new(Some("Channels"));
        channel_label.add_css_class("channel-button-label");
        channel_label.add_css_class("stream-button-label");
        channel_label.set_xalign(0.0);
        channel_label.set_halign(gtk::Align::Fill);
        channel_label.set_margin_end(22);
        channel_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        channel_button.set_child(Some(&channel_label));

        let title_label = gtk::Label::new(Some("No channel selected"));
        title_label.add_css_class("stream-title");
        title_label.add_css_class("stream-title-label");
        title_label.set_xalign(0.0);
        title_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        title_label.set_hexpand(true);

        let meta_label = gtk::Label::new(Some("Open the channel overlay to start playback"));
        meta_label.add_css_class("stream-meta");
        meta_label.add_css_class("stream-metadata-label");
        meta_label.set_xalign(0.0);
        meta_label.set_ellipsize(gtk::pango::EllipsizeMode::End);

        let empty_button = gtk::Button::new();
        empty_button.add_css_class("empty-select-button");
        empty_button.set_child(Some(&gtk::Image::from_icon_name("list-add-symbolic")));
        empty_button.set_halign(gtk::Align::Center);
        empty_button.set_valign(gtk::Align::Center);
        root.add_overlay(&empty_button);

        let fullscreen_button =
            icon_button_child(icons::window(WindowIcon::Fullscreen), "Fullscreen");
        let mute_button = icon_button_child(icons::volume(false), "Mute");
        mute_button.add_css_class("volume-mute-button");
        let epg_button = icon_button_child(icons::epg(), "Channel EPG");
        epg_button.add_css_class("epg-overlay-button");
        epg_button.set_visible(false);
        let stream_settings_button = icon_button_child(icons::stream_settings(), "Stream settings");
        stream_settings_button.add_css_class("stream-settings-button");
        let stream_settings_popover = StreamSettingsPopover::new(&stream_settings_button);
        let volume_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 130.0, 1.0);
        volume_scale.add_css_class("volume-scale");
        volume_scale.set_draw_value(false);
        volume_scale.set_value(session.borrow().volume());

        let player = Rc::new(Self {
            root,
            session,
            video,
            client,
            channel_overlay: RefCell::new(None),
            epg_overlay: RefCell::new(None),
            current_service_ref: RefCell::new(None),
            current_channel: RefCell::new(None),
            self_weak: RefCell::new(Weak::new()),
            channel_button,
            channel_label,
            title_label,
            meta_label,
            empty_button,
            fullscreen_button,
            fullscreen_active: Cell::new(false),
            top_left_controls: RefCell::new(None),
            top_right_controls: RefCell::new(None),
            footer_controls: RefCell::new(None),
            overlay_hide_source: RefCell::new(None),
            motion_has_last: Cell::new(false),
            motion_last_x: Cell::new(0.0),
            motion_last_y: Cell::new(0.0),
            move_pressed: Cell::new(false),
            move_press_x: Cell::new(0.0),
            move_press_y: Cell::new(0.0),
            mute_button,
            epg_button,
            stream_settings_button,
            stream_settings_popover,
            volume_scale,
        });
        *player.self_weak.borrow_mut() = Rc::downgrade(&player);

        let fullscreen_callback = Rc::new(on_fullscreen);
        player.build_overlays(
            Rc::new(open_settings),
            Rc::new(on_minimize),
            fullscreen_callback.clone(),
            Rc::new(on_close),
        );
        player.connect_controls(fullscreen_callback);
        if settings.borrow().box_url.trim().is_empty() {
            player.title_label.set_text("No receiver configured");
            player
                .meta_label
                .set_text("Open settings to enter the Dreambox / Enigma2 URL");
        }
        player
    }

    pub fn widget(&self) -> gtk::Overlay {
        self.root.clone()
    }

    pub fn show_picker(&self) {
        if let Some(overlay) = self.epg_overlay.borrow().as_ref() {
            overlay.hide();
        }
        if let Some(overlay) = self.channel_overlay.borrow().as_ref() {
            overlay.set_current_service_ref(self.current_service_ref.borrow().as_deref());
            overlay.show();
        }
        self.show_player_overlay();
    }

    pub fn play_first_channel(&self) {
        let result = self.client.bouquet_with_epg(0);
        match result {
            Ok(bouquet) => {
                if let Some(channel) = bouquet.channels.into_iter().next() {
                    self.play_channel(channel);
                }
            }
            Err(err) => self.meta_label.set_text(&format!("Loading failed: {err}")),
        }
    }

    pub fn apply_settings(&self, settings: &AppSettings) {
        self.session.borrow_mut().set_hwdec_enabled(settings.hwdec);
        self.client.set_base_url(&settings.box_url);
    }

    fn build_overlays(
        self: &Rc<Self>,
        open_settings: Rc<dyn Fn()>,
        on_minimize: Rc<dyn Fn()>,
        on_fullscreen: Rc<dyn Fn() -> bool>,
        on_close: Rc<dyn Fn()>,
    ) {
        let top_left = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        top_left.add_css_class("top-controls");
        top_left.add_css_class("top-overlay-controls");
        top_left.set_halign(gtk::Align::Start);
        top_left.set_valign(gtk::Align::Start);
        let settings_button = icon_button_child(icons::settings(), "Settings");
        settings_button.add_css_class("settings-overlay-button");
        top_left.append(&settings_button);
        self.root.add_overlay(&top_left);
        *self.top_left_controls.borrow_mut() = Some(top_left.clone());
        {
            let weak = Rc::downgrade(self);
            let open_settings = open_settings.clone();
            settings_button.connect_clicked(move |_| {
                if let Some(player) = weak.upgrade() {
                    player.show_player_overlay();
                }
                open_settings();
            });
        }

        let top_right = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        top_right.add_css_class("top-controls");
        top_right.add_css_class("top-overlay-controls");
        top_right.set_halign(gtk::Align::End);
        top_right.set_valign(gtk::Align::Start);
        let minimize_button = icon_button_child(icons::window(WindowIcon::Minimize), "Minimize");
        let close_button = icon_button_child(icons::window(WindowIcon::Close), "Close");
        close_button.add_css_class("close-button");
        top_right.append(&minimize_button);
        top_right.append(&self.fullscreen_button);
        top_right.append(&close_button);
        self.root.add_overlay(&top_right);
        *self.top_right_controls.borrow_mut() = Some(top_right.clone());
        {
            let weak = Rc::downgrade(self);
            let on_minimize = on_minimize.clone();
            minimize_button.connect_clicked(move |_| {
                if let Some(player) = weak.upgrade() {
                    player.show_player_overlay();
                }
                on_minimize();
            });
        }
        {
            let on_fullscreen = on_fullscreen.clone();
            let weak = Rc::downgrade(self);
            self.fullscreen_button.connect_clicked(move |_| {
                let fullscreen = on_fullscreen();
                if let Some(player) = weak.upgrade() {
                    player.set_fullscreen_state(fullscreen);
                    player.show_player_overlay();
                }
            });
        }
        {
            let weak = Rc::downgrade(self);
            let on_close = on_close.clone();
            close_button.connect_clicked(move |_| {
                if let Some(player) = weak.upgrade() {
                    player.show_player_overlay();
                }
                on_close();
            });
        }

        let footer = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        footer.add_css_class("player-footer");
        footer.add_css_class("video-footer");
        footer.set_halign(gtk::Align::Fill);
        footer.set_valign(gtk::Align::End);

        let stream_info = gtk::Box::new(gtk::Orientation::Vertical, 0);
        stream_info.add_css_class("stream-info");
        stream_info.add_css_class("stream-info-labels");
        stream_info.set_hexpand(true);
        stream_info.append(&self.title_label);
        stream_info.append(&self.meta_label);

        let stream_selector = gtk::Overlay::new();
        stream_selector.add_css_class("stream-selector");
        stream_selector.set_halign(gtk::Align::Start);
        stream_selector.set_hexpand(false);
        stream_selector.set_size_request(140, -1);
        stream_selector.set_child(Some(&self.channel_button));

        let refresh = icon_button("view-refresh-symbolic", "Resync video");
        refresh.add_css_class("stream-refresh-button");
        refresh.add_css_class("player-refresh-button");
        refresh.set_halign(gtk::Align::End);
        refresh.set_valign(gtk::Align::Center);
        refresh.set_margin_end(3);
        stream_selector.add_overlay(&refresh);

        footer.append(&stream_selector);
        footer.append(&stream_info);
        footer.append(&self.epg_button);
        footer.append(&self.mute_button);
        footer.append(&self.volume_scale);
        footer.append(&self.stream_settings_button);
        self.root.add_overlay(&footer);
        *self.footer_controls.borrow_mut() = Some(footer.clone());

        {
            let weak = Rc::downgrade(self);
            refresh.connect_clicked(move |_| {
                if let Some(player) = weak.upgrade() {
                    player.session.borrow().drop_buffers();
                    player.video.queue_render();
                    player.show_player_overlay();
                }
            });
        }
        {
            let weak = Rc::downgrade(self);
            self.stream_settings_button.connect_clicked(move |_| {
                if let Some(player) = weak.upgrade() {
                    player.show_stream_settings();
                }
            });
        }
        {
            let weak = Rc::downgrade(self);
            self.stream_settings_popover.connect_info_clicked(move || {
                if let Some(player) = weak.upgrade() {
                    player.session.borrow().toggle_stream_info();
                    player.stream_settings_popover.popdown();
                    player.show_player_overlay();
                }
            });
        }

        let weak: Weak<Self> = Rc::downgrade(self);
        let overlay = ChannelOverlay::new(&self.root, self.client.clone(), move |channel| {
            if let Some(player) = weak.upgrade() {
                player.play_channel(channel);
            }
        });
        *self.channel_overlay.borrow_mut() = Some(overlay);

        let epg_overlay = EpgOverlay::new(&self.root, self.client.clone());
        *self.epg_overlay.borrow_mut() = Some(epg_overlay);
    }

    fn connect_controls(self: &Rc<Self>, on_fullscreen: Rc<dyn Fn() -> bool>) {
        {
            let weak = Rc::downgrade(self);
            self.channel_button.connect_clicked(move |_| {
                if let Some(player) = weak.upgrade() {
                    player.show_picker();
                    player.show_player_overlay();
                }
            });
        }
        {
            let weak = Rc::downgrade(self);
            self.empty_button.connect_clicked(move |_| {
                if let Some(player) = weak.upgrade() {
                    player.show_picker();
                    player.show_player_overlay();
                }
            });
        }
        {
            let weak = Rc::downgrade(self);
            self.epg_button.connect_clicked(move |_| {
                if let Some(player) = weak.upgrade() {
                    player.show_epg_overlay();
                    player.show_player_overlay();
                }
            });
        }
        {
            let weak = Rc::downgrade(self);
            self.mute_button.connect_clicked(move |_| {
                if let Some(player) = weak.upgrade() {
                    player.session.borrow_mut().toggle_muted();
                    player.update_mute_icon();
                    player.show_player_overlay();
                }
            });
        }
        {
            let weak = Rc::downgrade(self);
            self.volume_scale.connect_value_changed(move |scale| {
                if let Some(player) = weak.upgrade() {
                    if player.session.borrow().muted() {
                        player.session.borrow_mut().set_muted(false);
                        player.update_mute_icon();
                    }
                    player.session.borrow_mut().set_volume(scale.value());
                    player.show_player_overlay();
                }
            });
        }

        let video_click = gtk::GestureClick::new();
        video_click.set_button(1);
        {
            let weak = Rc::downgrade(self);
            video_click.connect_pressed(move |_, n_press, _, _| {
                if n_press == 2 {
                    let fullscreen = on_fullscreen();
                    if let Some(player) = weak.upgrade() {
                        player.set_fullscreen_state(fullscreen);
                        player.show_player_overlay();
                    }
                }
            });
        }
        self.video.widget().add_controller(video_click);

        self.add_window_move_controller();

        self.add_volume_scroll_controls();

        let context_click = gtk::GestureClick::new();
        context_click.set_button(3);
        {
            let weak = Rc::downgrade(self);
            context_click.connect_pressed(move |_, _, _, _| {
                if let Some(player) = weak.upgrade() {
                    player.show_picker();
                    player.show_player_overlay();
                }
            });
        }
        self.root.add_controller(context_click);

        self.add_overlay_motion_controller();
        self.show_player_overlay();
    }

    fn add_overlay_motion_controller(self: &Rc<Self>) {
        let motion = gtk::EventControllerMotion::new();
        motion.set_propagation_phase(gtk::PropagationPhase::Capture);
        {
            let weak = Rc::downgrade(self);
            motion.connect_motion(move |_, x, y| {
                if let Some(player) = weak.upgrade() {
                    player.show_player_overlay_for_motion(x, y);
                }
            });
        }
        self.root.add_controller(motion);
    }

    fn show_player_overlay_for_motion(&self, x: f64, y: f64) {
        if self.motion_has_last.get()
            && (x - self.motion_last_x.get()).abs() < MOTION_EPSILON
            && (y - self.motion_last_y.get()).abs() < MOTION_EPSILON
        {
            return;
        }

        self.motion_has_last.set(true);
        self.motion_last_x.set(x);
        self.motion_last_y.set(y);
        self.show_player_overlay();
    }

    fn show_player_overlay(&self) {
        if let Some(top_left) = self.top_left_controls.borrow().as_ref() {
            top_left.set_visible(true);
        }
        if let Some(top_right) = self.top_right_controls.borrow().as_ref() {
            top_right.set_visible(true);
        }
        if let Some(footer) = self.footer_controls.borrow().as_ref() {
            footer.set_visible(true);
        }
        self.schedule_player_overlay_hide();
    }

    fn schedule_player_overlay_hide(&self) {
        if let Some(source) = self.overlay_hide_source.borrow_mut().take() {
            source.remove();
        }

        let weak = self.self_weak.borrow().clone();
        let source = glib::timeout_add_local(
            std::time::Duration::from_millis(PLAYER_OVERLAY_HIDE_DELAY_MS),
            move || {
                if let Some(player) = weak.upgrade() {
                    *player.overlay_hide_source.borrow_mut() = None;
                    player.hide_player_overlay();
                }
                glib::ControlFlow::Break
            },
        );
        *self.overlay_hide_source.borrow_mut() = Some(source);
    }

    fn hide_player_overlay(&self) {
        if self
            .channel_overlay
            .borrow()
            .as_ref()
            .is_some_and(|overlay| overlay.is_visible())
            || self
                .epg_overlay
                .borrow()
                .as_ref()
                .is_some_and(|overlay| overlay.is_visible())
            || self.stream_settings_popover.is_visible()
        {
            self.schedule_player_overlay_hide();
            return;
        }

        if let Some(top_left) = self.top_left_controls.borrow().as_ref() {
            top_left.set_visible(false);
        }
        if let Some(top_right) = self.top_right_controls.borrow().as_ref() {
            top_right.set_visible(false);
        }
        if let Some(footer) = self.footer_controls.borrow().as_ref() {
            footer.set_visible(false);
        }
    }

    fn add_window_move_controller(self: &Rc<Self>) {
        let controller = gtk::EventControllerLegacy::new();
        {
            let weak = Rc::downgrade(self);
            controller.connect_event(move |controller, event| {
                if let Some(player) = weak.upgrade() {
                    player.handle_window_move_event(controller, event)
                } else {
                    glib::Propagation::Proceed
                }
            });
        }
        self.video.widget().add_controller(controller);
    }

    fn handle_window_move_event(
        &self,
        controller: &gtk::EventControllerLegacy,
        event: &gtk::gdk::Event,
    ) -> glib::Propagation {
        if self.fullscreen_active.get() {
            return glib::Propagation::Proceed;
        }

        if event.event_type() == gtk::gdk::EventType::ButtonPress {
            self.show_player_overlay();
        }

        match event.event_type() {
            gtk::gdk::EventType::ButtonPress => {
                let Some(button) = event.downcast_ref::<gtk::gdk::ButtonEvent>() else {
                    return glib::Propagation::Proceed;
                };
                if button.button() == 1 {
                    if let Some((x, y)) = event.position() {
                        self.move_pressed.set(true);
                        self.move_press_x.set(x);
                        self.move_press_y.set(y);
                    }
                }
                glib::Propagation::Proceed
            }
            gtk::gdk::EventType::ButtonRelease => {
                if event
                    .downcast_ref::<gtk::gdk::ButtonEvent>()
                    .is_some_and(|button| button.button() == 1)
                {
                    self.move_pressed.set(false);
                }
                glib::Propagation::Proceed
            }
            gtk::gdk::EventType::MotionNotify => {
                if !self.move_pressed.get() {
                    return glib::Propagation::Proceed;
                }
                if !event
                    .modifier_state()
                    .contains(gtk::gdk::ModifierType::BUTTON1_MASK)
                {
                    self.move_pressed.set(false);
                    return glib::Propagation::Proceed;
                }
                let Some((x, y)) = event.position() else {
                    return glib::Propagation::Proceed;
                };
                if (x - self.move_press_x.get()).abs() < 4.0
                    && (y - self.move_press_y.get()).abs() < 4.0
                {
                    return glib::Propagation::Proceed;
                }

                self.move_pressed.set(false);
                if begin_window_move_from_event(controller, event, 1) {
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
            _ => glib::Propagation::Proceed,
        }
    }

    fn add_volume_scroll_controls(self: &Rc<Self>) {
        let video_scroll =
            gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        {
            let weak = Rc::downgrade(self);
            video_scroll.connect_scroll(move |_, dx, dy| {
                if let Some(player) = weak.upgrade() {
                    player.apply_volume_scroll(dx, dy)
                } else {
                    glib::Propagation::Proceed
                }
            });
        }
        self.video.widget().add_controller(video_scroll);

        let scale_scroll =
            gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        {
            let weak = Rc::downgrade(self);
            scale_scroll.connect_scroll(move |_, dx, dy| {
                if let Some(player) = weak.upgrade() {
                    player.apply_volume_scroll(dx, dy)
                } else {
                    glib::Propagation::Proceed
                }
            });
        }
        self.volume_scale.add_controller(scale_scroll);
    }

    fn apply_volume_scroll(&self, dx: f64, dy: f64) -> glib::Propagation {
        if dy == 0.0 || dy.abs() < dx.abs() {
            return glib::Propagation::Proceed;
        }

        if self.session.borrow().muted() {
            self.session.borrow_mut().set_muted(false);
            self.update_mute_icon();
        }

        let next =
            (self.volume_scale.value() - dy * VOLUME_SCROLL_STEP).clamp(VOLUME_MIN, VOLUME_MAX);
        self.volume_scale.set_value(next);
        self.show_player_overlay();
        glib::Propagation::Stop
    }

    fn show_stream_settings(&self) {
        if !self.session.borrow().is_playing() {
            self.show_player_overlay();
            return;
        }

        self.update_stream_settings_tracks();
        self.stream_settings_popover.popup();
        self.show_player_overlay();
    }

    fn update_stream_settings_tracks(&self) {
        let tracks = self.session.borrow().audio_tracks();
        let weak = self.self_weak.borrow().clone();
        self.stream_settings_popover
            .set_audio_tracks(&tracks, move |track_id| {
                if let Some(player) = weak.upgrade() {
                    player.select_audio_track(track_id);
                }
            });
    }

    fn select_audio_track(&self, track_id: i64) {
        self.session.borrow_mut().set_audio_track(track_id);
        self.stream_settings_popover.popdown();
        self.show_player_overlay();
    }

    fn play_channel(&self, channel: Channel) {
        self.meta_label.set_text("Resolving stream...");
        match self.client.resolve_stream_url(&channel.service_ref) {
            Ok(url) => {
                *self.current_service_ref.borrow_mut() = Some(channel.service_ref.clone());
                *self.current_channel.borrow_mut() = Some(channel.clone());
                self.session.borrow_mut().load_stream(&url, &channel.name);
                self.channel_label.set_text(&channel.name);
                self.empty_button.set_visible(false);
                self.epg_button.set_visible(true);
                if let Some(event) = channel.epg {
                    self.title_label.set_text(&event.title);
                    self.meta_label
                        .set_text(&format!("{}  {}", channel.name, event.time_range()));
                } else {
                    self.title_label.set_text(&channel.name);
                    self.meta_label.set_text("No EPG event available");
                }
                self.video.queue_render();
            }
            Err(err) => {
                self.meta_label.set_text(&format!("Stream failed: {err}"));
            }
        }
    }

    fn show_epg_overlay(&self) {
        let Some(channel) = self.current_channel.borrow().clone() else {
            return;
        };
        if let Some(overlay) = self.channel_overlay.borrow().as_ref() {
            overlay.hide();
        }
        if let Some(overlay) = self.epg_overlay.borrow().as_ref() {
            overlay.show_for_channel(channel);
        }
    }

    fn update_mute_icon(&self) {
        self.mute_button
            .set_child(Some(&icons::volume(self.session.borrow().muted())));
    }

    fn set_fullscreen_state(&self, fullscreen: bool) {
        let (icon, tooltip) = if fullscreen {
            (WindowIcon::Restore, "Leave fullscreen")
        } else {
            (WindowIcon::Fullscreen, "Fullscreen")
        };
        self.fullscreen_active.set(fullscreen);
        self.fullscreen_button.set_child(Some(&icons::window(icon)));
        self.fullscreen_button.set_tooltip_text(Some(tooltip));
    }
}

fn begin_window_move_from_event(
    controller: &gtk::EventControllerLegacy,
    event: &gtk::gdk::Event,
    button: i32,
) -> bool {
    let Some(device) = event.device() else {
        return false;
    };
    let Some((x, y)) = event.position() else {
        return false;
    };
    let Some(widget) = controller.widget() else {
        return false;
    };
    let Some(root) = widget.root() else {
        return false;
    };
    let Ok(window) = root.downcast::<gtk::Window>() else {
        return false;
    };
    let Some(surface) = window.surface() else {
        return false;
    };
    let Ok(toplevel) = surface.downcast::<gtk::gdk::Toplevel>() else {
        return false;
    };

    toplevel.begin_move(&device, button, x, y, event.time());
    true
}

fn icon_button(icon: &str, tooltip: &str) -> gtk::Button {
    icon_button_child(gtk::Image::from_icon_name(icon), tooltip)
}

fn icon_button_child(icon: impl IsA<gtk::Widget>, tooltip: &str) -> gtk::Button {
    let button = gtk::Button::new();
    button.add_css_class("icon-button");
    button.add_css_class("overlay-icon-button");
    button.set_has_frame(false);
    button.set_tooltip_text(Some(tooltip));
    button.set_child(Some(&icon));
    button
}
