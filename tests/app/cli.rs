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
