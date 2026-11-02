//! Instants in IANA time zones (the `tz` feature): an instant's wall time where the user is.

#![cfg(feature = "tz")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dxn_intl::{DateTime, DateTimeFormatter, DateTimeOptions, HourCycle, Length, parse_locale};

fn wall(value: DateTime) -> (i32, u8, u8, u8, u8, u8) {
    (
        value.year,
        value.month,
        value.day,
        value.hour,
        value.minute,
        value.second,
    )
}

#[test]
fn an_instant_takes_the_zone_s_offset_at_that_instant() {
    // Zagreb is on CET (UTC+1) in November and on CEST (UTC+2) in July.
    let winter = DateTime::from_instant("2026-11-02T09:30:00Z", "Europe/Zagreb").unwrap();
    assert_eq!(wall(winter), (2026, 11, 2, 10, 30, 0));
    let summer = DateTime::from_instant("2026-07-01T09:30:00Z", "Europe/Zagreb").unwrap();
    assert_eq!(wall(summer), (2026, 7, 1, 11, 30, 0));
    // A day earlier in New York (EST, UTC-5).
    let new_york = DateTime::from_instant("2026-11-02T03:00:00Z", "America/New_York").unwrap();
    assert_eq!(wall(new_york), (2026, 11, 1, 22, 0, 0));
}

#[test]
fn javascript_s_iso_strings_and_offsets_are_instants() {
    // `Date.toISOString()` writes milliseconds and `Z`.
    let iso = DateTime::from_instant("2026-11-02T09:30:00.250Z", "UTC").unwrap();
    assert_eq!(wall(iso), (2026, 11, 2, 9, 30, 0));
    assert_eq!(iso.nanosecond, 250_000_000);
    let offset = DateTime::from_instant("2026-11-02T10:30:00+01:00", "Asia/Tokyo").unwrap();
    assert_eq!(wall(offset), (2026, 11, 2, 18, 30, 0));
}

#[test]
fn milliseconds_since_the_epoch_are_instants_too() {
    let tokyo = DateTime::from_unix_millis_in(0, "Asia/Tokyo").unwrap();
    assert_eq!(wall(tokyo), (1970, 1, 1, 9, 0, 0));
    let zagreb = DateTime::from_unix_millis_in(1_793_611_800_000, "Europe/Zagreb").unwrap();
    assert_eq!(
        zagreb,
        DateTime::from_instant("2026-11-02T09:30:00Z", "Europe/Zagreb").unwrap()
    );
}

#[test]
fn a_zoned_instant_formats_as_its_wall_time() {
    let local = DateTime::from_instant("2026-11-02T09:30:00Z", "Europe/Zagreb").unwrap();
    let formatter = DateTimeFormatter::new(
        &parse_locale("hr").unwrap(),
        DateTimeOptions::time(Length::Short).with_hour_cycle(HourCycle::H23),
    )
    .unwrap();
    assert_eq!(formatter.format(&local).unwrap(), "10:30");
}

#[test]
fn a_wall_time_or_an_unknown_zone_is_refused() {
    for text in ["2026-11-02T09:30:00", "2026-11-02", "yesterday", ""] {
        assert!(
            DateTime::from_instant(text, "UTC").is_err(),
            "{text:?} names no instant"
        );
    }
    let error = DateTime::from_instant("2026-11-02T09:30:00Z", "Europe/Atlantis")
        .unwrap_err()
        .to_string();
    assert!(error.contains("Europe/Atlantis"), "{error}");
}
