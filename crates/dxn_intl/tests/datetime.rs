//! Dates and times of day in a locale's calendar, words and clock.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dxn_intl::{
    DateFields, DateTime, DateTimeFormatter, DateTimeOptions, HourCycle, Length, TimePrecision,
    parse_locale,
};

fn format(tag: &str, options: DateTimeOptions, value: DateTime) -> String {
    DateTimeFormatter::new(&parse_locale(tag).unwrap(), options)
        .unwrap()
        .format(&value)
        .unwrap()
}

fn moment() -> DateTime {
    DateTime::new(2026, 10, 27, 15, 45, 30).unwrap()
}

#[test]
fn dates_in_three_lengths() {
    assert_eq!(
        format("en-US", DateTimeOptions::date(Length::Short), moment()),
        "10/27/26"
    );
    assert_eq!(
        format("en-US", DateTimeOptions::date(Length::Medium), moment()),
        "Oct 27, 2026"
    );
    assert_eq!(
        format("en-US", DateTimeOptions::date(Length::Long), moment()),
        "October 27, 2026"
    );
    assert_eq!(
        format("de-DE", DateTimeOptions::date(Length::Long), moment()),
        "27. Oktober 2026"
    );
    assert_eq!(
        format("hr-HR", DateTimeOptions::date(Length::Short), moment()),
        "27. 10. 2026."
    );
}

#[test]
fn fields_choose_what_is_shown() {
    let weekday = DateTimeOptions {
        date_fields: Some(DateFields::YearMonthDayWeekday),
        ..DateTimeOptions::date(Length::Long)
    };
    assert_eq!(
        format("en-US", weekday, moment()),
        "Tuesday, October 27, 2026"
    );
    let month = DateTimeOptions {
        date_fields: Some(DateFields::YearMonth),
        ..DateTimeOptions::date(Length::Long)
    };
    assert_eq!(format("en-US", month, moment()), "October 2026");
}

#[test]
fn times_follow_the_locale_or_the_hour_cycle() {
    assert_eq!(
        format("en-US", DateTimeOptions::time(Length::Short), moment()),
        "3:45\u{202f}PM"
    );
    assert_eq!(
        format("de-DE", DateTimeOptions::time(Length::Short), moment()),
        "15:45"
    );
    let h23 = DateTimeOptions::time(Length::Short).with_hour_cycle(HourCycle::H23);
    assert_eq!(format("en-US", h23, moment()), "15:45");
    let h12 = DateTimeOptions::time(Length::Short).with_hour_cycle(HourCycle::H12);
    assert_eq!(format("de-DE", h12, moment()), "3:45\u{202f}PM");
    let seconds = DateTimeOptions {
        time_precision: Some(TimePrecision::Second),
        ..DateTimeOptions::time(Length::Short)
    };
    assert_eq!(format("de-DE", seconds, moment()), "15:45:30");
    assert_eq!("h24".parse::<HourCycle>().unwrap(), HourCycle::H23);
}

#[test]
fn date_and_time_together() {
    assert_eq!(
        format(
            "en-US",
            DateTimeOptions::date_time(Length::Medium),
            moment()
        ),
        "Oct 27, 2026, 3:45\u{202f}PM"
    );
}

#[test]
fn other_calendars() {
    // Thai uses the Buddhist era.
    assert!(format("th-TH", DateTimeOptions::date(Length::Long), moment()).contains("2569"));
}

#[test]
fn unix_time_at_an_offset() {
    // 2026-10-27T14:45:30Z
    let millis = 1_793_112_330_000;
    assert_eq!(DateTime::from_unix_millis(millis, 3600).unwrap(), moment());
    assert_eq!(
        DateTime::from_unix_millis(0, 0).unwrap(),
        DateTime::date(1970, 1, 1).unwrap()
    );
    assert_eq!(
        DateTime::from_unix_millis(-1, 0).unwrap(),
        DateTime {
            nanosecond: 999_000_000,
            ..DateTime::new(1969, 12, 31, 23, 59, 59).unwrap()
        }
    );
}

#[test]
fn invalid_dates_are_errors() {
    assert!(DateTime::date(2026, 2, 30).is_err());
    assert!(DateTime::new(2026, 1, 1, 24, 0, 0).is_err());
}
