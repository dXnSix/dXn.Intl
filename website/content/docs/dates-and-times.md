---
title: Dates and times
description: DateTime wall times on the ISO calendar, DateTimeFormatter and DateTimeOptions, lengths, hour cycles, calendars, and turning a Unix timestamp plus a zone offset into a wall time.
group: Formatting
order: 50
---

`DateTimeFormatter` writes a date, a time of day, or both, in a locale's words, field order, calendar and clock. You give it a `DateTime`, which is a wall time: the year, month, day and time a clock on the wall shows, with no time zone.

```rust
use dxn_intl::{DateTime, DateTimeFormatter, DateTimeOptions, HourCycle, Length, parse_locale};

let moment = DateTime::new(2026, 10, 27, 15, 45, 0)?;

let en = DateTimeFormatter::new(&parse_locale("en-US")?, DateTimeOptions::date(Length::Long))?;
assert_eq!(en.format(&moment)?, "October 27, 2026");

let de = DateTimeFormatter::new(&parse_locale("de-DE")?, DateTimeOptions::date(Length::Long))?;
assert_eq!(de.format(&moment)?, "27. Oktober 2026");

let clock = DateTimeOptions::time(Length::Short).with_hour_cycle(HourCycle::H23);
let en_24 = DateTimeFormatter::new(&parse_locale("en-US")?, clock)?;
assert_eq!(en_24.format(&moment)?, "15:45");
```

## DateTime

`DateTime` has public fields (`year`, `month`, `day`, `hour`, `minute`, `second`, `nanosecond`), is `Copy`, and orders chronologically. Months and days start at 1. The year `0` is 1 BCE.

| Constructor | Result |
|---|---|
| `DateTime::date(2026, 10, 27)` | Midnight at the start of the day |
| `DateTime::new(2026, 10, 27, 15, 45, 30)` | The day and time of day, nanoseconds 0 |
| `DateTime::from_unix_millis(millis, offset_seconds)` | The wall time of an instant; see [below](#from-an-instant-to-a-wall-time) |

The constructors return `Result` because the value is checked against the calendar: February 30th, month 13 and hour 24 are `IntlError::InvalidDateTime`. The year must be within -9999 to 9999. The fields are public, so a struct literal can hold an invalid date; it is checked again by `DateTimeFormatter::format`, which returns the same error.

```rust
use dxn_intl::{DateTime, IntlError};

assert!(DateTime::date(2024, 2, 29).is_ok());   // a leap year
assert!(matches!(DateTime::date(2026, 2, 29), Err(IntlError::InvalidDateTime(_))));
assert!(DateTime::new(2026, 1, 1, 24, 0, 0).is_err());
assert!(DateTime::date(10_000, 1, 1).is_err());

assert!(DateTime::date(2026, 1, 1)? < DateTime::date(2026, 1, 2)?);
```

The documented range of `second` is 0 to 59, but ICU4X's time type also accepts 60, so `DateTime::new(2026, 10, 27, 23, 59, 60)` succeeds; do not rely on it. The ISO calendar is the input only: the formatter converts the date to the locale's calendar.

## DateTimeOptions

`DateTimeOptions` has four public fields. The constructors cover the common cases.

| Field | Type | Meaning |
|---|---|---|
| `date_fields` | `Option<DateFields>` | Which parts of the date; `None` shows no date |
| `time_precision` | `Option<TimePrecision>` | Hour, minute or second; `None` shows no time of day |
| `length` | `Length` | `Short`, `Medium` (the default) or `Long` |
| `hour_cycle` | `Option<HourCycle>` | The clock, when not the locale's |

| Constructor | Shows |
|---|---|
| `DateTimeOptions::date(length)` | Year, month and day |
| `DateTimeOptions::time(length)` | Hours and minutes |
| `DateTimeOptions::date_time(length)` | Both |
| `.with_hour_cycle(cycle)` | The same options on that clock |

`DateTimeOptions::default()` is `date(Length::Medium)`. Options are `Copy`, so one value can make formatters for many locales.

### Length

`Length` sets how long the whole text is. Dates for `2026-10-27`:

| Locale | `Short` | `Medium` | `Long` |
|---|---|---|---|
| `en-US` | `10/27/26` | `Oct 27, 2026` | `October 27, 2026` |
| `en-GB` | `27/10/2026` | `27 Oct 2026` | `27 October 2026` |
| `de` | `27.10.26` | `27.10.2026` | `27. Oktober 2026` |
| `hr` | `27. 10. 2026.` | `27. lis 2026.` | `27. listopada 2026.` |
| `ja` | `2026/10/27` | `2026/10/27` | `2026年10月27日` |

Combine a long date with a time and the locale adds its joiner: `October 27, 2026 at 3:45:30 PM` in English, `27. Oktober 2026 um 15:45:30` in German. Length does not change a time-only text.

### Date fields

`DateFields` chooses which parts of the date to show. The `en-US` text for `2026-10-27` at `Length::Medium`:

| `DateFields` | Text |
|---|---|
| `Day` | `27` |
| `MonthDay` | `Oct 27` |
| `YearMonthDay` | `Oct 27, 2026` |
| `DayWeekday` | `27 Tue` |
| `MonthDayWeekday` | `Tue, Oct 27` |
| `YearMonthDayWeekday` | `Tue, Oct 27, 2026` |
| `Weekday` | `Tue` |
| `Month` | `Oct` |
| `YearMonth` | `Oct 2026` |
| `Year` | `2026` |

```rust
use dxn_intl::{DateFields, DateTime, DateTimeFormatter, DateTimeOptions, Length, parse_locale};

let options = DateTimeOptions {
    date_fields: Some(DateFields::YearMonthDayWeekday),
    ..DateTimeOptions::date(Length::Long)
};
let formatter = DateTimeFormatter::new(&parse_locale("en-US")?, options)?;
assert_eq!(formatter.format(&DateTime::date(2026, 10, 27)?)?, "Tuesday, October 27, 2026");

let de = DateTimeFormatter::new(&parse_locale("de")?, options)?;
assert_eq!(de.format(&DateTime::date(2026, 10, 27)?)?, "Dienstag, 27. Oktober 2026");
```

`Month`, `YearMonth` and `Year` cannot be combined with a time of day. `DateTimeFormatter::new` returns `IntlError::InvalidOptions` for them with a time, and for options that show neither a date nor a time.

### Time of day

`TimePrecision` is `Hour` (`3 PM`), `Minute` (`3:45 PM`, the default) or `Second` (`3:45:30 PM`).

```rust
use dxn_intl::{DateTime, DateTimeFormatter, DateTimeOptions, Length, TimePrecision, parse_locale};

let moment = DateTime::new(2026, 10, 27, 15, 45, 30)?;
let options = DateTimeOptions {
    time_precision: Some(TimePrecision::Second),
    ..DateTimeOptions::time(Length::Short)
};
let de = DateTimeFormatter::new(&parse_locale("de-DE")?, options)?;
assert_eq!(de.format(&moment)?, "15:45:30");

let en = DateTimeFormatter::new(&parse_locale("en-US")?, options)?;
assert_eq!(en.format(&moment)?, "3:45:30\u{202f}PM"); // narrow no-break space before PM
```

Nothing below the second is shown, so `nanosecond` has no effect on the text.

## Hour cycles

The clock comes from the locale: English uses 12 hours with `AM` and `PM`, German 24 hours. Override it with `with_hour_cycle`, or let the user's own tag do it with a `-u-hc-` extension.

| `HourCycle` | Clock | 12:05 reads |
|---|---|---|
| `H11` | 0 to 11, with AM and PM | `0:05 PM` |
| `H12` | 1 to 12, with AM and PM | `12:05 PM` |
| `H23` | 0 to 23 | `12:05` |

```rust
use dxn_intl::{DateTime, DateTimeFormatter, DateTimeOptions, HourCycle, Length, parse_locale};

let moment = DateTime::new(2026, 10, 27, 15, 45, 0)?;
let time = DateTimeOptions::time(Length::Short);

let from_tag = DateTimeFormatter::new(&parse_locale("de-u-hc-h12")?, time)?;
assert_eq!(from_tag.format(&moment)?, "3:45\u{202f}PM");

let from_option = DateTimeFormatter::new(&parse_locale("en-US")?, time.with_hour_cycle(HourCycle::H23))?;
assert_eq!(from_option.format(&moment)?, "15:45");

// The option wins over the tag.
let both = DateTimeFormatter::new(&parse_locale("en-u-hc-h23")?, time.with_hour_cycle(HourCycle::H12))?;
assert_eq!(both.format(&moment)?, "3:45\u{202f}PM");

assert_eq!("h24".parse::<HourCycle>()?, HourCycle::H23);
```

`HourCycle` reads the Unicode names with `FromStr`, case-insensitively and trimmed: `h11`, `h12`, `h23`, and `h24`, which maps to `H23` because CLDR no longer has a 24-hour clock that starts at 1. Anything else is `IntlError::InvalidOptions`.

## Calendars

The formatter shows the date in the locale's own calendar, while `DateTime` stays ISO. The same day, long:

| Locale | Text | Calendar |
|---|---|---|
| `en` | `October 27, 2026` | Gregorian |
| `th` | `27 ตุลาคม 2569` | Buddhist era |
| `fa` | `۵ آبان ۱۴۰۵` | Persian |
| `ja-JP-u-ca-japanese` | `令和8年10月27日` | Japanese eras |
| `ar-SA-u-ca-islamic-umalqura` | `١٦ جمادى الأولى ١٤٤٨ هـ` | Umm al-Qura |
| `fa-u-ca-gregory` | `۲۷ اکتبر ۲۰۲۶` | Gregorian, Persian digits |

A `-u-ca-` extension picks the calendar, and `-u-nu-` the digits: `ar-u-nu-latn` writes `27 أكتوبر 2026`. Because `th` and `fa` default to their national calendars, a Thai or Persian user sees 2569 or 1405 unless the locale you pass says otherwise. Add `-u-ca-gregory` when you need Gregorian dates, for example for a date that must match a contract.

:::note Which locales default to which calendar is CLDR's decision
The table was produced by running the formatter. A locale's default calendar can differ from what you expect for its region (`ar-SA` printed Gregorian here), so test the locales you ship.
:::

## From an instant to a wall time

A Unix timestamp names an instant; the user reads the wall time in their zone. `DateTime::from_unix_millis(millis, utc_offset_seconds)` adds the offset to the instant and splits it into fields.

```rust
use dxn_intl::DateTime;

let millis = 1_793_112_330_000; // 2026-10-27T14:45:30Z

assert_eq!(DateTime::from_unix_millis(millis, 0)?, DateTime::new(2026, 10, 27, 14, 45, 30)?);
assert_eq!(DateTime::from_unix_millis(millis, 3600)?, DateTime::new(2026, 10, 27, 15, 45, 30)?);   // UTC+1
assert_eq!(DateTime::from_unix_millis(millis, -5 * 3600)?, DateTime::new(2026, 10, 27, 9, 45, 30)?); // UTC-5
assert_eq!(DateTime::from_unix_millis(millis, 5 * 3600 + 1800)?, DateTime::new(2026, 10, 27, 20, 15, 30)?); // UTC+5:30

assert_eq!(DateTime::from_unix_millis(0, 0)?, DateTime::date(1970, 1, 1)?);
assert_eq!(DateTime::from_unix_millis(1_793_112_330_123, 0)?.nanosecond, 123_000_000);
```

The offset is in seconds east of UTC (`3600` for UTC+1). Milliseconds keep their millisecond precision in `nanosecond`; negative timestamps work. A value so far from 1970 that its year is outside -9999 to 9999 is built without complaint, and `format` then returns `IntlError::InvalidDateTime`.

### Time zones

The crate has no time zone database and no clock. The offset belongs to the zone **and** the instant: Zagreb is UTC+1 in January and UTC+2 in July, so a fixed number gives the wrong hour for half the year. Get the offset at the instant from something that knows the rules:

- **Native:** a crate with the zone database, such as `jiff` or `chrono-tz`. The [cookbook](./cookbook.md#format-a-timestamp-for-the-users-zone) has a complete example.
- **Browser:** `new Date(millis).getTimezoneOffset()` returns minutes **west** of UTC and varies with the date, so pass `-minutes * 60` as the offset.
- **A server:** the user's zone name from their profile or from the client, resolved with a zone database.

:::warning Offset at the instant, not offset now
To show a timestamp from last July, compute the offset for that July instant. Using today's offset shifts every date that falls on the other side of a daylight-saving change by an hour, and can move an event near midnight to the wrong day.
:::

## Reference

| Item | Description |
|---|---|
| `DateTimeFormatter::new(&Locale, DateTimeOptions) -> Result<DateTimeFormatter>` | `InvalidOptions` for field combinations that cannot be shown; `Data` if the locale's data cannot load. |
| `format(&DateTime) -> Result<String>` | `InvalidDateTime` when the value is outside the calendar. |
| `DateFields` | `Day`, `MonthDay`, `YearMonthDay`, `DayWeekday`, `MonthDayWeekday`, `YearMonthDayWeekday`, `Weekday`, `Month`, `YearMonth`, `Year`. |
| `Length` | `Short`, `Medium` (default), `Long`. |
| `TimePrecision` | `Hour`, `Minute` (default), `Second`. |
| `HourCycle` | `H11`, `H12`, `H23`; `FromStr` reads `h11`, `h12`, `h23`, `h24`. |

## Next

[Cookbook](./cookbook.md) turns a timestamp and a zone name into text. [Errors](./errors.md) lists what each call can return.
