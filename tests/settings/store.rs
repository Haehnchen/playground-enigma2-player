use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

struct TestConfig(PathBuf);

impl TestConfig {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "enigma2-player-test-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }

    fn path(&self) -> PathBuf {
        self.0.join("enigma2-player/settings.json")
    }
}

impl Drop for TestConfig {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn load_returns_default_when_settings_file_is_missing() {
    let dir = TestConfig::new();

    assert_eq!(load_from(&dir.path()), AppSettings::default());
}

#[test]
fn load_returns_default_when_settings_json_is_invalid() {
    let dir = TestConfig::new();
    let path = dir.path();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "not-json").unwrap();

    assert_eq!(load_from(&dir.path()), AppSettings::default());
}

#[test]
fn load_returns_default_when_stored_box_url_has_no_supported_scheme() {
    let dir = TestConfig::new();
    let path = dir.path();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, r#"{"box_url": "receiver.local", "hwdec": false}"#).unwrap();

    assert_eq!(load_from(&dir.path()), AppSettings::default());
}

#[test]
fn save_writes_normalized_settings() {
    let dir = TestConfig::new();
    let settings = AppSettings {
        box_url: "http://receiver.local/".to_string(),
        hwdec: false,
    };

    save_to(&settings, &dir.path()).unwrap();

    assert!(dir.path().exists());
    let loaded = load_from(&dir.path());
    assert_eq!(loaded.box_url, "http://receiver.local");
    assert!(!loaded.hwdec);
}

#[test]
fn save_rejects_box_url_without_http_or_https_scheme() {
    let dir = TestConfig::new();
    let settings = AppSettings {
        box_url: "receiver.local".to_string(),
        hwdec: true,
    };

    let err = save_to(&settings, &dir.path()).unwrap_err();

    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    assert!(!dir.path().exists());
}

#[test]
fn validate_box_url_accepts_http_and_https_urls() {
    assert_eq!(
        validate_box_url(" http://receiver.local/ ").unwrap(),
        "http://receiver.local"
    );
    assert_eq!(
        validate_box_url("https://receiver.local/").unwrap(),
        "https://receiver.local"
    );
}

#[test]
fn validate_box_url_rejects_empty_and_scheme_less_urls() {
    assert_eq!(validate_box_url("").unwrap_err(), BOX_URL_SCHEME_ERROR);
    assert_eq!(
        validate_box_url("receiver.local").unwrap_err(),
        BOX_URL_SCHEME_ERROR
    );
}

#[test]
fn load_preserves_url_when_hwdec_is_missing() {
    let dir = TestConfig::new();
    let path = dir.path();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, r#"{"box_url": "http://receiver.local/"}"#).unwrap();

    assert_eq!(
        load_from(&path),
        AppSettings {
            box_url: "http://receiver.local".to_string(),
            hwdec: true,
        }
    );
}

#[test]
fn missing_settings_fields_use_application_defaults() {
    let settings: AppSettings = serde_json::from_str("{}").unwrap();
    assert_eq!(settings, AppSettings::default());
    let settings: AppSettings = serde_json::from_str(r#"{"hwdec": false}"#).unwrap();
    assert_eq!(
        settings,
        AppSettings {
            hwdec: false,
            ..AppSettings::default()
        }
    );
}
