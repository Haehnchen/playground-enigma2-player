use super::store::{save, validate_settings, AppSettings};
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub fn show(
    parent: &gtk::ApplicationWindow,
    settings: Rc<RefCell<AppSettings>>,
    on_saved: impl Fn(AppSettings) + 'static,
) {
    let window = gtk::Window::builder()
        .title("Settings")
        .transient_for(parent)
        .modal(true)
        .default_width(760)
        .default_height(480)
        .build();
    window.set_decorated(true);
    window.set_resizable(true);

    let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    root.add_css_class("settings-root");
    root.set_hexpand(true);
    root.set_vexpand(true);
    window.set_child(Some(&root));

    let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 8);
    sidebar.add_css_class("settings-sidebar");
    sidebar.set_size_request(170, -1);
    root.append(&sidebar);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.add_css_class("settings-content");
    content.set_hexpand(true);
    content.set_vexpand(true);
    root.append(&content);

    let stack = gtk::Stack::new();
    stack.set_hexpand(true);
    stack.set_vexpand(true);
    content.append(&stack);

    let current = settings.borrow().clone();
    let url_entry = gtk::Entry::new();
    url_entry.set_text(&current.box_url);
    url_entry.set_placeholder_text(Some("http://receiver.local"));
    url_entry.add_css_class("settings-entry");
    url_entry.set_hexpand(true);

    let hwdec_check = gtk::CheckButton::with_label("Hardware decoding");
    hwdec_check.set_active(current.hwdec);
    hwdec_check.add_css_class("settings-check");

    let status = gtk::Label::new(None);
    status.add_css_class("settings-status");
    status.set_xalign(0.0);
    status.set_hexpand(true);

    let general = gtk::Box::new(gtk::Orientation::Vertical, 8);
    general.add_css_class("settings-page");
    general.append(&page_title("General"));
    general.append(&field_label("Dreambox / Enigma2 URL"));
    general.append(&url_entry);
    general.append(&hint_label(
        "The URL is used for bouquets, EPG and stream playlist resolving.",
    ));
    general.append(&spaced_check(&hwdec_check));
    general.append(&hint_label(
        "Let mpv use GPU video decoding where supported. Disable this if playback is unstable or video renders incorrectly.",
    ));
    general.append(&spacer());
    stack.add_named(&general, Some("general"));

    let about = gtk::Box::new(gtk::Orientation::Vertical, 10);
    about.add_css_class("settings-page");
    about.append(&page_title("About"));
    about.append(&about_row("Version", env!("ENIGMA2_PLAYER_BUILD_VERSION")));
    about.append(&about_row("Build date", env!("ENIGMA2_PLAYER_BUILD_DATE")));
    about.append(&spacer());
    stack.add_named(&about, Some("about"));

    let action_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    action_row.add_css_class("settings-action-row");
    action_row.add_css_class("settings-footer");
    action_row.set_vexpand(false);
    action_row.append(&status);
    let save_button = gtk::Button::with_label("Save");
    save_button.add_css_class("settings-primary-button");
    save_button.set_valign(gtk::Align::Center);
    save_button.set_size_request(72, 32);
    action_row.append(&save_button);
    content.append(&action_row);

    let general_button = sidebar_button("General");
    general_button.add_css_class("settings-sidebar-button-selected");
    let about_button = sidebar_button("About");
    sidebar.append(&general_button);
    sidebar.append(&about_button);
    stack.set_visible_child_name("general");

    {
        let stack = stack.clone();
        let general_button_for_click = general_button.clone();
        let about_button_for_click = about_button.clone();
        general_button.connect_clicked(move |_| {
            stack.set_visible_child_name("general");
            general_button_for_click.add_css_class("settings-sidebar-button-selected");
            about_button_for_click.remove_css_class("settings-sidebar-button-selected");
        });
    }
    {
        let stack = stack.clone();
        let general_button_for_click = general_button.clone();
        let about_button_for_click = about_button.clone();
        about_button.connect_clicked(move |_| {
            stack.set_visible_child_name("about");
            about_button_for_click.add_css_class("settings-sidebar-button-selected");
            general_button_for_click.remove_css_class("settings-sidebar-button-selected");
        });
    }

    let on_saved = Rc::new(on_saved);
    {
        let settings = settings.clone();
        let window = window.clone();
        let on_saved = on_saved.clone();
        save_button.connect_clicked(move |_| {
            let next = AppSettings {
                box_url: url_entry.text().to_string(),
                hwdec: hwdec_check.is_active(),
            };
            let next = match validate_settings(&next) {
                Ok(next) => next,
                Err(message) => {
                    show_error_dialog(&window, "Invalid receiver URL", &message);
                    return;
                }
            };

            match save(&next) {
                Ok(()) => {
                    *settings.borrow_mut() = next.clone();
                    on_saved(next);
                    window.close();
                }
                Err(err) => show_error_dialog(&window, "Saving failed", &err.to_string()),
            }
        });
    }

    window.present();
}

fn show_error_dialog(parent: &gtk::Window, title: &str, message: &str) {
    let dialog = gtk::MessageDialog::builder()
        .transient_for(parent)
        .modal(true)
        .message_type(gtk::MessageType::Warning)
        .buttons(gtk::ButtonsType::Ok)
        .text(title)
        .secondary_text(message)
        .build();
    dialog.connect_response(|dialog, _| dialog.close());
    dialog.present();
}

fn page_title(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.add_css_class("settings-page-title");
    label.set_xalign(0.0);
    label
}

fn field_label(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.add_css_class("settings-field-label");
    label.set_xalign(0.0);
    label
}

fn hint_label(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.add_css_class("settings-hint-label");
    label.set_xalign(0.0);
    label.set_wrap(true);
    label
}

fn about_row(key: &str, value: &str) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row.add_css_class("settings-about-row");
    let key_label = field_label(key);
    key_label.set_size_request(92, -1);
    let value_label = hint_label(value);
    value_label.set_hexpand(true);
    row.append(&key_label);
    row.append(&value_label);
    row
}

fn sidebar_button(text: &str) -> gtk::Button {
    let button = gtk::Button::new();
    button.add_css_class("settings-sidebar-button");
    button.set_halign(gtk::Align::Fill);
    button.set_hexpand(true);

    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.0);
    label.set_halign(gtk::Align::Fill);
    button.set_child(Some(&label));
    button
}

fn spaced_check(check: &gtk::CheckButton) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    row.add_css_class("settings-check-row");
    row.append(check);
    row
}

fn spacer() -> gtk::Box {
    let spacer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    spacer.set_vexpand(true);
    spacer
}
