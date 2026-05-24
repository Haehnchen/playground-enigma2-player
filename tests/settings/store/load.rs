use crate::support::{restore_xdg_config_home, set_xdg_config_home, temp_config_dir, ENV_LOCK};
use enigma2_player_core::settings::store::{load, settings_path, AppSettings};
use std::fs;

#[test]
fn load_returns_default_when_settings_file_is_missing() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = temp_config_dir("load-missing");
    let previous = set_xdg_config_home(&dir);

    assert_eq!(load(), AppSettings::default());

    restore_xdg_config_home(previous);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn load_returns_default_when_settings_json_is_invalid() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = temp_config_dir("load-invalid-json");
    let previous = set_xdg_config_home(&dir);
    let path = settings_path();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "not-json").unwrap();

    assert_eq!(load(), AppSettings::default());

    restore_xdg_config_home(previous);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn load_returns_default_when_stored_box_url_has_no_supported_scheme() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = temp_config_dir("load-invalid-url");
    let previous = set_xdg_config_home(&dir);
    let path = settings_path();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, r#"{"box_url": "receiver.local", "hwdec": false}"#).unwrap();

    assert_eq!(load(), AppSettings::default());

    restore_xdg_config_home(previous);
    fs::remove_dir_all(dir).unwrap();
}
