use std::str::FromStr;

use icu::calendar::{Date, Iso};
use icu::datetime::fieldsets::builder::{DateFields as IcuDateFields, FieldSetBuilder};
use icu::datetime::fieldsets::enums::CompositeDateTimeFieldSet;
use icu::datetime::options::{Length as IcuLength, TimePrecision as IcuTimePrecision};
use icu::datetime::{DateTimeFormatter as IcuFormatter, DateTimeFormatterPreferences};
use icu::locale::Locale;
use icu::locale::preferences::extensions::unicode::keywords::HourCycle as IcuHourCycle;
use icu::time::Time;

use crate::{IntlError, Result};

/// A date and time of day on the ISO (Gregorian) calendar, as a wall clock shows it: no time
/// zone. A formatter shows it in the locale's own calendar (Buddhist in `th`, …).
///
/// To show an instant, take its local time in the user's zone first, or use
/// [`DateTime::from_unix_millis`] with the zone's offset at that instant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DateTime {
    /// The year; `0` is 1 BCE.
    pub year: i32,
    /// The month, 1–12.
    pub month: u8,
    /// The day of the month, from 1.
    pub day: u8,
    /// The hour, 0–23.
    pub hour: u8,
    /// The minute, 0–59.
    pub minute: u8,
    /// The second, 0–59.
    pub second: u8,
    /// The nanosecond within the second.
    pub nanosecond: u32,
}

impl DateTime {
    /// Midnight at the start of the given day.
    pub fn date(year: i32, month: u8, day: u8) -> Result<Self> {
        Self::new(year, month, day, 0, 0, 0)
    }

    /// The given day and time of day.
    pub fn new(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> Result<Self> {
        let value = Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            nanosecond: 0,
        };
        value.to_icu()?;
        Ok(value)
    }

    /// The wall time `millis` milliseconds after 1970-01-01T00:00:00Z, at `utc_offset_seconds`
    /// east of UTC (`3600` for UTC+1).
    pub fn from_unix_millis(millis: i64, utc_offset_seconds: i32) -> Result<Self> {
        let local = millis
            .checked_add(i64::from(utc_offset_seconds) * 1000)
            .ok_or_else(|| IntlError::InvalidDateTime(format!("{millis} ms is out of range")))?;
        let days = local.div_euclid(86_400_000);
        let in_day = local.rem_euclid(86_400_000);
        let (year, month, day) = civil_from_days(days)
            .ok_or_else(|| IntlError::InvalidDateTime(format!("{millis} ms is out of range")))?;
        let seconds = in_day / 1000;
        Ok(Self {
            year,
            month,
            day,
            hour: (seconds / 3600) as u8,
            minute: (seconds / 60 % 60) as u8,
            second: (seconds % 60) as u8,
            nanosecond: (in_day % 1000) as u32 * 1_000_000,
        })
    }

    fn to_icu(self) -> Result<icu::time::DateTime<Iso>> {
        let date = Date::try_new_iso(self.year, self.month, self.day)
            .map_err(|error| IntlError::InvalidDateTime(error.to_string()))?;
        let time = Time::try_new(self.hour, self.minute, self.second, self.nanosecond)
            .map_err(|error| IntlError::InvalidDateTime(error.to_string()))?;
        Ok(icu::time::DateTime { date, time })
    }
}

/// Howard Hinnant's `civil_from_days`: the proleptic Gregorian date `days` after 1970-01-01.
fn civil_from_days(days: i64) -> Option<(i32, u8, u8)> {
    let z = days.checked_add(719_468)?;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
    let year = yoe + era * 400 + i64::from(month <= 2);
    Some((i32::try_from(year).ok()?, month, day))
}

/// Which parts of the date to show.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DateFields {
    /// "1st".
    Day,
    /// "January 1".
    MonthDay,
    /// "January 1, 2000".
    YearMonthDay,
    /// "Saturday 1st".
    DayWeekday,
    /// "Saturday, January 1".
    MonthDayWeekday,
    /// "Saturday, January 1, 2000".
    YearMonthDayWeekday,
    /// "Saturday".
    Weekday,
    /// "January", alone; no time of day goes with it.
    Month,
    /// "January 2000"; no time of day goes with it.
    YearMonth,
    /// "2000"; no time of day goes with it.
    Year,
}

impl DateFields {
    fn to_icu(self) -> IcuDateFields {
        match self {
            Self::Day => IcuDateFields::D,
            Self::MonthDay => IcuDateFields::MD,
            Self::YearMonthDay => IcuDateFields::YMD,
            Self::DayWeekday => IcuDateFields::DE,
            Self::MonthDayWeekday => IcuDateFields::MDE,
            Self::YearMonthDayWeekday => IcuDateFields::YMDE,
            Self::Weekday => IcuDateFields::E,
            Self::Month => IcuDateFields::M,
            Self::YearMonth => IcuDateFields::YM,
            Self::Year => IcuDateFields::Y,
        }
    }
}

/// How long the formatted text is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Length {
    /// "1/1/00".
    Short,
    /// "Jan 1, 2000".
    #[default]
    Medium,
    /// "January 1, 2000".
    Long,
}

/// How precisely the time of day is shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TimePrecision {
    /// "3 PM".
    Hour,
    /// "3:45 PM".
    #[default]
    Minute,
    /// "3:45:12 PM".
    Second,
}

/// The clock the time of day is shown on, overriding the locale's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HourCycle {
    /// 0–11, as sometimes in Japan (`h11`).
    H11,
    /// 1–12, with AM and PM (`h12`).
    H12,
    /// 0–23 (`h23`).
    H23,
}

impl FromStr for HourCycle {
    type Err = IntlError;

    /// Reads a Unicode hour cycle: `h11`, `h12`, `h23`, or `h24` (shown as `h23`: CLDR has no
    /// 24-hour clock that starts at 1 any more).
    fn from_str(text: &str) -> Result<Self> {
        match text.trim().to_ascii_lowercase().as_str() {
            "h11" => Ok(Self::H11),
            "h12" => Ok(Self::H12),
            "h23" | "h24" => Ok(Self::H23),
            _ => Err(IntlError::InvalidOptions(format!(
                "`{text}` is not an hour cycle"
            ))),
        }
    }
}

/// What [`DateTimeFormatter`] shows: the date's fields, the time of day's precision, or both.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DateTimeOptions {
    /// The date's fields; `None` shows no date.
    pub date_fields: Option<DateFields>,
    /// The time of day's precision; `None` shows no time of day.
    pub time_precision: Option<TimePrecision>,
    /// The length of the whole text.
    pub length: Length,
    /// The clock, when not the locale's (nor its `-u-hc-` extension's).
    pub hour_cycle: Option<HourCycle>,
}

impl DateTimeOptions {
    /// Year, month and day at `length`: "Jan 1, 2000".
    pub fn date(length: Length) -> Self {
        Self {
            date_fields: Some(DateFields::YearMonthDay),
            time_precision: None,
            length,
            hour_cycle: None,
        }
    }

    /// Hours and minutes at `length`: "3:45 PM".
    pub fn time(length: Length) -> Self {
        Self {
            date_fields: None,
            time_precision: Some(TimePrecision::Minute),
            length,
            hour_cycle: None,
        }
    }

    /// Year, month and day with hours and minutes at `length`: "Jan 1, 2000, 3:45 PM".
    pub fn date_time(length: Length) -> Self {
        Self {
            date_fields: Some(DateFields::YearMonthDay),
            time_precision: Some(TimePrecision::Minute),
            length,
            hour_cycle: None,
        }
    }

    /// The same options on the clock `hour_cycle`.
    pub fn with_hour_cycle(mut self, hour_cycle: HourCycle) -> Self {
        self.hour_cycle = Some(hour_cycle);
        self
    }
}

impl Default for DateTimeOptions {
    fn default() -> Self {
        Self::date(Length::Medium)
    }
}

/// Formats dates and times of day for a locale, in its calendar, words and order.
///
/// ```
/// use dxn_intl::{DateTime, DateTimeFormatter, DateTimeOptions, HourCycle, Length, parse_locale};
///
/// let moment = DateTime::new(2026, 10, 27, 15, 45, 0)?;
///
/// let en = DateTimeFormatter::new(&parse_locale("en-US")?, DateTimeOptions::date(Length::Long))?;
/// assert_eq!(en.format(&moment)?, "October 27, 2026");
///
/// let options = DateTimeOptions::time(Length::Short).with_hour_cycle(HourCycle::H23);
/// let en_24 = DateTimeFormatter::new(&parse_locale("en-US")?, options)?;
/// assert_eq!(en_24.format(&moment)?, "15:45");
/// # Ok::<(), dxn_intl::IntlError>(())
/// ```
#[derive(Debug)]
pub struct DateTimeFormatter {
    formatter: IcuFormatter<CompositeDateTimeFieldSet>,
}

impl DateTimeFormatter {
    /// A formatter for `locale` showing what `options` asks for.
    pub fn new(locale: &Locale, options: DateTimeOptions) -> Result<Self> {
        let mut builder = FieldSetBuilder::new();
        builder.length = Some(match options.length {
            Length::Short => IcuLength::Short,
            Length::Medium => IcuLength::Medium,
            Length::Long => IcuLength::Long,
        });
        builder.date_fields = options.date_fields.map(DateFields::to_icu);
        builder.time_precision = options.time_precision.map(|precision| match precision {
            TimePrecision::Hour => IcuTimePrecision::Hour,
            TimePrecision::Minute => IcuTimePrecision::Minute,
            TimePrecision::Second => IcuTimePrecision::Second,
        });
        let field_set = builder
            .build_composite_datetime()
            .map_err(|error| IntlError::InvalidOptions(format!("{error:?}")))?;
        let mut prefs = DateTimeFormatterPreferences::from(locale);
        if let Some(hour_cycle) = options.hour_cycle {
            prefs.hour_cycle = Some(match hour_cycle {
                HourCycle::H11 => IcuHourCycle::H11,
                HourCycle::H12 => IcuHourCycle::H12,
                HourCycle::H23 => IcuHourCycle::H23,
            });
        }
        let formatter = IcuFormatter::try_new(prefs, field_set)
            .map_err(|error| IntlError::Data(error.to_string()))?;
        Ok(Self { formatter })
    }

    /// `value` formatted for the locale.
    pub fn format(&self, value: &DateTime) -> Result<String> {
        Ok(self.formatter.format(&value.to_icu()?).to_string())
    }
}
