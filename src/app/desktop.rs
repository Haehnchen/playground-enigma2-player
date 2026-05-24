use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const APP_ID: &str = "local.enigma2-player";
const APP_NAME: &str = "Enigma2 Player";
const APP_COMMENT: &str = "Watch Dreambox and Enigma2 TV streams with mpv";
const APP_ICON: &str = include_str!("../../data/local.enigma2-player.svg");

pub fn write_user_desktop_identity() {
    if let Err(err) = try_write_user_desktop_identity() {
        eprintln!("enigma2-player: could not write desktop identity: {err}");
    }
}

fn try_write_user_desktop_identity() -> std::io::Result<()> {
    let data_dir = user_data_dir();
    let applications_dir = data_dir.join("applications");
    let icons_dir = data_dir.join("icons/hicolor/scalable/apps");
    let desktop_path = applications_dir.join(format!("{APP_ID}.desktop"));
    let icon_path = icons_dir.join(format!("{APP_ID}.svg"));

    fs::create_dir_all(&applications_dir)?;
    fs::create_dir_all(&icons_dir)?;
    fs::write(&icon_path, APP_ICON)?;
    fs::write(desktop_path, desktop_entry(&executable_path(), &icon_path))?;
    Ok(())
}

fn user_data_dir() -> PathBuf {
    env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from(".local/share"))
}

fn executable_path() -> PathBuf {
    env::current_exe()
        .or_else(|_| {
            env::args_os()
                .next()
                .map(PathBuf::from)
                .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))
        })
        .unwrap_or_else(|_| PathBuf::from("enigma2-player"))
}

fn desktop_entry(exec_path: &Path, icon_path: &Path) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName={APP_NAME}\nComment={APP_COMMENT}\nExec={}\nIcon={}\nTerminal=false\nCategories=AudioVideo;Video;Player;TV;\nStartupNotify=true\nStartupWMClass={APP_ID}\n",
        quote_desktop_path(exec_path),
        icon_path.display(),
    )
}

fn quote_desktop_path(path: &Path) -> String {
    let raw = path.to_string_lossy();
    let mut quoted = String::with_capacity(raw.len() + 2);
    quoted.push('"');
    for ch in raw.chars() {
        if matches!(ch, '"' | '\\' | '`' | '$') {
            quoted.push('\\');
        }
        quoted.push(ch);
    }
    quoted.push('"');
    quoted
}

#[cfg(test)]
mod tests {
    use super::quote_desktop_path;
    use std::path::Path;

    #[test]
    fn quotes_desktop_exec_paths() {
        assert_eq!(
            quote_desktop_path(Path::new("/tmp/enigma2 player/$bin")),
            r#""/tmp/enigma2 player/\$bin""#
        );
    }
}
