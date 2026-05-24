#![allow(dead_code)]

use enigma2_player_core::enigma2::api::{Enigma2Client, Enigma2Error};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub static ENV_LOCK: Mutex<()> = Mutex::new(());
use std::time::{SystemTime, UNIX_EPOCH};

pub fn client_with_responses(
    responses: Vec<(&str, &str)>,
) -> (Enigma2Client, Arc<Mutex<Vec<String>>>) {
    let responses = Arc::new(Mutex::new(
        responses
            .into_iter()
            .map(|(url, body)| (url.to_string(), body.to_string()))
            .collect::<Vec<_>>(),
    ));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let requests_for_get = requests.clone();
    let responses_for_get = responses.clone();
    let client = Enigma2Client::new_with_http_getter("http://receiver.local", move |url| {
        requests_for_get.lock().unwrap().push(url.to_string());
        let mut responses = responses_for_get.lock().unwrap();
        if responses.is_empty() {
            return Err(Enigma2Error::Http(format!("unexpected request {url}")));
        }
        let (expected_url, body) = responses.remove(0);
        assert_eq!(url, expected_url);
        Ok(body)
    });
    (client, requests)
}

pub fn temp_config_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("enigma2-player-test-{name}-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

pub fn set_xdg_config_home(dir: &Path) -> Option<OsString> {
    let previous = std::env::var_os("XDG_CONFIG_HOME");
    std::env::set_var("XDG_CONFIG_HOME", dir);
    previous
}

pub fn restore_xdg_config_home(previous: Option<OsString>) {
    if let Some(previous) = previous {
        std::env::set_var("XDG_CONFIG_HOME", previous);
    } else {
        std::env::remove_var("XDG_CONFIG_HOME");
    }
}
