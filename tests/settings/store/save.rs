use crate::support::{restore_xdg_config_home, set_xdg_config_home, temp_config_dir, ENV_LOCK};
use enigma2_player_core::settings::store::{load, save, settings_path, AppSettings};
use std::fs;
use std::io;

#[test]
fn save_writes_normalized_settings_to_xdg_config_home() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = temp_config_dir("save");
    let previous = set_xdg_config_home(&dir);
    let settings = AppSettings {
        box_url: "http://receiver.local/".to_string(),
        hwdec: false,
    };

    save(&settings).unwrap();

    assert!(settings_path().exists());
    let loaded = load();
    assert_eq!(loaded.box_url, "http://receiver.local");
    assert!(!loaded.hwdec);

    restore_xdg_config_home(previous);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn save_rejects_box_url_without_http_or_https_scheme() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = temp_config_dir("save-invalid-url");
    let previous = set_xdg_config_home(&dir);
    let settings = AppSettings {
        box_url: "receiver.local".to_string(),
        hwdec: true,
    };

    let err = save(&settings).unwrap_err();

    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    assert!(!settings_path().exists());

    restore_xdg_config_home(previous);
    fs::remove_dir_all(dir).unwrap();
}
