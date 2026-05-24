use super::icons;
use super::session::AudioTrack;
use gtk::prelude::*;

#[derive(Clone)]
pub struct StreamSettingsPopover {
    popover: gtk::Popover,
    settings_box: gtk::Box,
    audio_header: gtk::Box,
    audio_list: gtk::Box,
    divider: gtk::Separator,
    info_button: gtk::Button,
}

impl StreamSettingsPopover {
    pub fn new(relative_to: &impl IsA<gtk::Widget>) -> Self {
        let popover = gtk::Popover::new();
        popover.add_css_class("stream-settings-popover");
        popover.set_position(gtk::PositionType::Top);
        popover.set_has_arrow(false);
        popover.set_parent(relative_to);

        let settings_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
        settings_box.add_css_class("stream-settings-menu");
        settings_box.add_css_class("stream-settings-menu-compact");
        popover.set_child(Some(&settings_box));

        let audio_header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        audio_header.set_halign(gtk::Align::Fill);
        audio_header.set_valign(gtk::Align::Center);
        let audio_title = settings_label("Audio", "stream-settings-heading");
        audio_title.set_valign(gtk::Align::Center);
        audio_header.append(&audio_title);
        settings_box.append(&audio_header);

        let audio_list = gtk::Box::new(gtk::Orientation::Vertical, 2);
        audio_list.set_halign(gtk::Align::Fill);
        settings_box.append(&audio_list);

        let divider = gtk::Separator::new(gtk::Orientation::Horizontal);
        divider.add_css_class("stream-settings-divider");
        settings_box.append(&divider);

        let info_button = info_button();
        settings_box.append(&info_button);

        audio_header.set_visible(false);
        audio_list.set_visible(false);
        divider.set_visible(false);

        Self {
            popover,
            settings_box,
            audio_header,
            audio_list,
            divider,
            info_button,
        }
    }

    pub fn popup(&self) {
        self.popover.popup();
    }

    pub fn popdown(&self) {
        self.popover.popdown();
    }

    pub fn is_visible(&self) -> bool {
        self.popover.is_visible()
    }

    pub fn connect_info_clicked(&self, f: impl Fn() + 'static) {
        self.info_button.connect_clicked(move |_| f());
    }

    pub fn set_audio_tracks<F>(&self, tracks: &[AudioTrack], on_select: F)
    where
        F: Fn(i64) + Clone + 'static,
    {
        clear_box(&self.audio_list);

        let has_choices = tracks.len() > 1;
        if has_choices {
            self.settings_box
                .remove_css_class("stream-settings-menu-compact");
            self.settings_box
                .add_css_class("stream-settings-menu-with-audio");
        } else {
            self.settings_box
                .remove_css_class("stream-settings-menu-with-audio");
            self.settings_box
                .add_css_class("stream-settings-menu-compact");
        }
        self.audio_header.set_visible(has_choices);
        self.audio_list.set_visible(has_choices);
        self.divider.set_visible(has_choices);

        if !has_choices {
            return;
        }

        for track in audio_tracks_in_menu_order(tracks) {
            self.append_audio_track_button(track, on_select.clone());
        }
    }

    fn append_audio_track_button<F>(&self, track: &AudioTrack, on_select: F)
    where
        F: Fn(i64) + 'static,
    {
        let button = audio_track_button(track);
        let id = track.id;
        button.connect_clicked(move |_| on_select(id));
        self.audio_list.append(&button);
    }
}

fn settings_label(text: &str, css_class: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.0);
    label.set_halign(gtk::Align::Start);
    label.add_css_class(css_class);
    label
}

fn audio_track_button(track: &AudioTrack) -> gtk::Button {
    let button = gtk::Button::new();
    let label = gtk::Label::new(Some(&audio_track_button_label(track)));

    label.set_xalign(0.0);
    label.set_halign(gtk::Align::Fill);
    label.set_hexpand(true);
    button.set_child(Some(&label));
    button.add_css_class("stream-settings-item");
    if track.selected {
        button.add_css_class("stream-settings-item-selected");
    }
    button.set_halign(gtk::Align::Fill);
    button
}

fn audio_track_button_label(track: &AudioTrack) -> String {
    if track.selected {
        format!("{} (current)", track.label)
    } else {
        track.label.clone()
    }
}

fn info_button() -> gtk::Button {
    let button = gtk::Button::new();
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let label = gtk::Label::new(Some("Stream Info"));

    button.add_css_class("stream-settings-item");
    button.set_halign(gtk::Align::Fill);
    button.set_hexpand(true);
    content.set_halign(gtk::Align::Fill);
    content.set_hexpand(true);
    label.set_xalign(0.0);
    label.set_hexpand(true);

    content.append(&icons::info());
    content.append(&label);
    button.set_child(Some(&content));

    button
}

fn clear_box(box_: &gtk::Box) {
    while let Some(child) = box_.first_child() {
        box_.remove(&child);
    }
}

#[doc(hidden)]
pub fn audio_tracks_in_menu_order(tracks: &[AudioTrack]) -> Vec<&AudioTrack> {
    tracks
        .iter()
        .filter(|track| !track.selected)
        .chain(tracks.iter().filter(|track| track.selected))
        .collect()
}
