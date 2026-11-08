use super::quote_desktop_path;
use std::path::Path;

#[test]
fn quotes_desktop_exec_paths() {
    assert_eq!(
        quote_desktop_path(Path::new("/tmp/enigma2 player/$bin")),
        r#""/tmp/enigma2 player/\$bin""#
    );
}
