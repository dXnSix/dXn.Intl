//! Ranges of dates and times of day: both ends as the date formatter shows them, joined by the
//! locale's range separator, and one value when they show the same.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dxn_intl::{
    DateFields, DateRangeFormatter, DateTime, DateTimeOptions, HourCycle, Length, TimePrecision,
    parse_locale,
};

fn range(tag: &str, options: DateTimeOptions, start: DateTime, end: DateTime) -> String {
    DateRangeFormatter::new(&parse_locale(tag).unwrap(), options)
        .unwrap()
        .format(&start, &end)
        .unwrap()
}

fn date(year: i32, month: u8, day: u8) -> DateTime {
    DateTime::date(year, month, day).unwrap()
}

#[test]
fn ends_with_the_locales_separator() {
    let medium = DateTimeOptions::date(Length::Medium);
    let long = DateTimeOptions::date(Length::Long);
    assert_eq!(
        range("en-US", medium, date(2026, 10, 27), date(2026, 10, 30)),
        "Oct 27, 2026\u{2009}–\u{2009}Oct 30, 2026"
    );
    assert_eq!(
        range("de", long, date(2026, 10, 27), date(2027, 1, 5)),
        "27. Oktober 2026\u{2009}–\u{2009}5. Januar 2027"
    );
    assert_eq!(
        range("ja", medium, date(2026, 10, 27), date(2026, 10, 30)),
        "2026/10/27～2026/10/30"
    );
    assert_eq!(
        range("ru", long, date(2026, 10, 27), date(2026, 10, 30)),
        "27 октября 2026\u{202f}г.—30 октября 2026\u{202f}г."
    );
    assert_eq!(
        range("zh-Hant", long, date(2026, 10, 27), date(2026, 10, 30)),
        "2026年10月27日至2026年10月30日"
    );
}

#[test]
fn times_of_day() {
    let start = DateTime::new(2026, 10, 27, 9, 5, 0).unwrap();
    let end = DateTime::new(2026, 10, 27, 17, 30, 0).unwrap();
    assert_eq!(
        range("en-US", DateTimeOptions::time(Length::Short), start, end),
        "9:05\u{202f}AM\u{2009}–\u{2009}5:30\u{202f}PM"
    );
    let h23 = DateTimeOptions::time(Length::Short).with_hour_cycle(HourCycle::H23);
    assert_eq!(
        range("en-US", h23, start, end),
        "09:05\u{2009}–\u{2009}17:30"
    );
    assert_eq!(
        range(
            "en-US",
            DateTimeOptions::date_time(Length::Medium),
            start,
            end
        ),
        "Oct 27, 2026, 9:05\u{202f}AM\u{2009}–\u{2009}Oct 27, 2026, 5:30\u{202f}PM"
    );
}

#[test]
fn ends_that_show_the_same_are_one() {
    let start = DateTime::new(2026, 10, 27, 9, 5, 0).unwrap();
    let later = DateTime::new(2026, 10, 27, 17, 30, 0).unwrap();
    assert_eq!(
        range("en-US", DateTimeOptions::date(Length::Long), start, later),
        "October 27, 2026"
    );
    let months = DateTimeOptions {
        date_fields: Some(DateFields::YearMonth),
        time_precision: None,
        length: Length::Long,
        hour_cycle: None,
    };
    assert_eq!(
        range("en-US", months, date(2026, 10, 1), date(2026, 10, 31)),
        "October 2026"
    );
    assert_eq!(
        range("en-US", months, date(2026, 10, 1), date(2026, 11, 1)),
        "October 2026\u{2009}–\u{2009}November 2026"
    );
    let hours = DateTimeOptions {
        date_fields: None,
        time_precision: Some(TimePrecision::Hour),
        length: Length::Short,
        hour_cycle: None,
    };
    assert_eq!(
        range(
            "en-US",
            hours,
            start,
            DateTime::new(2026, 10, 27, 9, 55, 0).unwrap()
        ),
        "9\u{202f}AM"
    );
}

#[test]
fn every_field_set_and_length_formats() {
    let fields = [
        Some(DateFields::Day),
        Some(DateFields::MonthDay),
        Some(DateFields::YearMonthDay),
        Some(DateFields::DayWeekday),
        Some(DateFields::MonthDayWeekday),
        Some(DateFields::YearMonthDayWeekday),
        Some(DateFields::Weekday),
        Some(DateFields::Month),
        Some(DateFields::YearMonth),
        Some(DateFields::Year),
        None,
    ];
    let start = DateTime::new(2026, 10, 27, 9, 5, 0).unwrap();
    let end = DateTime::new(2027, 1, 5, 17, 30, 0).unwrap();
    for tag in ["en-US", "de", "hr", "fr", "ar", "th", "fa"] {
        for length in [Length::Short, Length::Medium, Length::Long] {
            for date_fields in fields {
                let time_precision = if date_fields.is_none() {
                    Some(TimePrecision::Minute)
                } else {
                    None
                };
                let options = DateTimeOptions {
                    date_fields,
                    time_precision,
                    length,
                    hour_cycle: None,
                };
                let text = range(tag, options, start, end);
                assert!(!text.is_empty(), "{tag} {length:?} {date_fields:?}");
            }
        }
    }
}
