use super::{format_time, format_time_in_timezone};

#[test]
fn formats_epg_times_with_summer_and_winter_time() {
    let berlin = glib::TimeZone::new(Some("Europe/Berlin"));

    for (timestamp, expected) in [
        (1_794_144_600, "14:30"), // 2026-11-08 13:30 UTC (winter).
        (1_783_513_800, "14:30"), // 2026-07-08 12:30 UTC (summer).
        (1_774_744_200, "01:30"), // Before the spring clock change.
        (1_774_747_800, "03:30"), // After the spring clock change.
        (1_792_888_200, "02:30"), // Before the autumn clock change.
        (1_792_891_800, "02:30"), // After the autumn clock change.
    ] {
        assert_eq!(format_time_in_timezone(timestamp, &berlin), expected);
    }
}

#[test]
fn missing_or_invalid_epg_times_use_placeholder() {
    for timestamp in [0, -1, i64::MIN, i64::MAX] {
        assert_eq!(format_time(timestamp), "--:--");
    }
}
