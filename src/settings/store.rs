use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const BOX_URL_SCHEME_ERROR: &str =
    "Enter a Dreambox / Enigma2 URL starting with http:// or https://.";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct AppSettings {
    pub box_url: String,
    pub hwdec: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            box_url: String::new(),
            hwdec: true,
        }
    }
}

impl AppSettings {
    fn normalized(mut self) -> Self {
        self.box_url = crate::enigma2::urls::normalize_base_url(&self.box_url);
        self
    }
}

pub fn validate_box_url(value: &str) -> Result<String, String> {
    let normalized = crate::enigma2::urls::normalize_base_url(value);
    if !normalized.is_empty() && crate::enigma2::urls::has_supported_url_scheme(&normalized) {
        Ok(normalized)
    } else {
        Err(BOX_URL_SCHEME_ERROR.to_string())
    }
}

pub fn validate_settings(settings: &AppSettings) -> Result<AppSettings, String> {
    Ok(AppSettings {
        box_url: validate_box_url(&settings.box_url)?,
        hwdec: settings.hwdec,
    })
}

pub fn load() -> AppSettings {
    load_from(&settings_path())
}

fn load_from(path: &Path) -> AppSettings {
    let Ok(data) = fs::read_to_string(path) else {
        return AppSettings::default();
    };
    let Ok(settings) = serde_json::from_str::<AppSettings>(&data) else {
        return AppSettings::default();
    };
    let settings = settings.normalized();
    if settings.box_url.is_empty()
        || crate::enigma2::urls::has_supported_url_scheme(&settings.box_url)
    {
        settings
    } else {
        AppSettings::default()
    }
}

pub fn save(settings: &AppSettings) -> io::Result<()> {
    save_to(settings, &settings_path())
}

fn save_to(settings: &AppSettings, path: &Path) -> io::Result<()> {
    let settings = validate_settings(settings)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(&settings)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    fs::write(path, data)
}

fn settings_path() -> PathBuf {
    config_root().join("enigma2-player").join("settings.json")
}

fn config_root() -> PathBuf {
    if let Some(value) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(value);
    }
    if let Some(value) = std::env::var_os("HOME") {
        return PathBuf::from(value).join(".config");
    }
    PathBuf::from(".")
}

#[cfg(test)]
#[path = "../../tests/settings/store.rs"]
mod tests;
