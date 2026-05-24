use crate::app::desktop;
use crate::common::player::{style, window_chrome};
use crate::enigma2::api::Enigma2Client;
use crate::settings::store::{load, validate_box_url, AppSettings};
use crate::settings::window::show as show_settings_window;
use crate::ui::single_player::SinglePlayer;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct CliOptions {
    show_picker: bool,
    start_first: bool,
    box_url: Option<String>,
}

pub fn run_from_env() -> i32 {
    desktop::write_user_desktop_identity();
    let options = parse_args(std::env::args().skip(1));
    let app = gtk::Application::builder()
        .application_id("local.enigma2-player")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_activate(move |app| {
        style::install();

        let mut loaded_settings = load();
        if let Some(box_url) = &options.box_url {
            match validate_box_url(box_url) {
                Ok(box_url) => loaded_settings.box_url = box_url,
                Err(message) => eprintln!("enigma2-player: {message}"),
            }
        }
        let settings = Rc::new(RefCell::new(loaded_settings));
        let should_open_settings = settings.borrow().box_url.trim().is_empty();
        let client = Enigma2Client::new(settings.borrow().box_url.clone());

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("Enigma2 Player")
            .default_width(1280)
            .default_height(720)
            .build();
        window.set_decorated(false);
        window.set_size_request(480, 270);
        let fullscreen = Rc::new(RefCell::new(false));
        let minimize_window = {
            let window = window.clone();
            move || window.minimize()
        };
        let toggle_fullscreen = {
            let window = window.clone();
            let fullscreen = fullscreen.clone();
            move || toggle_window_fullscreen(&window, &fullscreen)
        };
        let close_window = {
            let window = window.clone();
            move || window.close()
        };

        let player_holder: Rc<RefCell<Option<Rc<SinglePlayer>>>> = Rc::new(RefCell::new(None));
        let window_for_settings = window.clone();
        let settings_for_window = settings.clone();
        let client_for_window = client.clone();
        let holder_for_window = player_holder.clone();
        let open_settings = move || {
            let settings = settings_for_window.clone();
            let client = client_for_window.clone();
            let holder = holder_for_window.clone();
            show_settings_window(&window_for_settings, settings, move |next: AppSettings| {
                client.set_base_url(&next.box_url);
                if let Some(player) = holder.borrow().as_ref() {
                    player.apply_settings(&next);
                }
            });
        };

        let player = SinglePlayer::new(
            settings.clone(),
            client.clone(),
            open_settings,
            minimize_window,
            toggle_fullscreen,
            close_window,
        );
        let player_widget = player.widget();
        window_chrome::add_resize_handles(&player_widget, &window, fullscreen.clone());
        window.set_child(Some(&player_widget));
        *player_holder.borrow_mut() = Some(player.clone());
        window.present();

        if should_open_settings {
            let window = window.clone();
            let settings = settings.clone();
            let client = client.clone();
            let player = player.clone();
            glib::idle_add_local_once(move || {
                let client = client.clone();
                let player = player.clone();
                show_settings_window(&window, settings, move |next: AppSettings| {
                    client.set_base_url(&next.box_url);
                    player.apply_settings(&next);
                });
            });
        }

        if !should_open_settings && options.show_picker {
            let player = player.clone();
            glib::idle_add_local_once(move || player.show_picker());
        }
        if !should_open_settings && options.start_first {
            let player = player.clone();
            glib::idle_add_local_once(move || player.play_first_channel());
        }
    });

    app.run_with_args(&["enigma2-player"]).into()
}

fn toggle_window_fullscreen(
    window: &gtk::ApplicationWindow,
    fullscreen: &Rc<RefCell<bool>>,
) -> bool {
    let next = !*fullscreen.borrow();
    if next {
        window.fullscreen();
    } else {
        window.unfullscreen();
    }
    *fullscreen.borrow_mut() = next;
    next
}

fn parse_args(args: impl Iterator<Item = String>) -> CliOptions {
    let mut options = CliOptions::default();
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--show-picker" => options.show_picker = true,
            "--start-first" => options.start_first = true,
            "--box-url" => {
                if let Some(value) = args.next() {
                    options.box_url = Some(value);
                }
            }
            value if value.starts_with("--box-url=") => {
                options.box_url = Some(value["--box-url=".len()..].to_string());
            }
            _ => {}
        }
    }
    options
}

#[cfg(test)]
mod tests {
    use super::parse_args;

    #[test]
    fn parses_launch_flags() {
        let options = parse_args(
            ["--show-picker", "--start-first"]
                .into_iter()
                .map(String::from),
        );
        assert!(options.show_picker);
        assert!(options.start_first);
    }

    #[test]
    fn parses_box_url_flag_forms() {
        let options = parse_args(
            ["--box-url", "receiver.local"]
                .into_iter()
                .map(String::from),
        );
        assert_eq!(options.box_url.as_deref(), Some("receiver.local"));

        let options = parse_args(
            ["--box-url=http://receiver.local"]
                .into_iter()
                .map(String::from),
        );
        assert_eq!(options.box_url.as_deref(), Some("http://receiver.local"));
    }
}
