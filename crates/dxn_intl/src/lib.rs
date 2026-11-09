//! Locale-aware plurals, numbers and dates for Rust, on [ICU4X](https://icu4x.unicode.org)'s
//! CLDR data for every language it has, independent of any UI framework and building for
//! `wasm32-unknown-unknown`.
//!
//! - [`PluralRules`]: a number's cardinal or ordinal [`PluralCategory`] in a locale.
//! - [`NumberFormatter`]: decimal numbers, percentages and amounts of money, with digit,
//!   grouping and sign options.
//! - [`DateTimeFormatter`]: dates and times of day in the locale's calendar, words and clock;
//!   with the `tz` feature, [`DateTime::from_instant`] takes an instant's wall time in an IANA
//!   time zone.
//! - [`parse_locale`], [`negotiate`] and [`direction`]: reading a language tag, picking the
//!   best available locale for a user's list, and the direction a locale is written in.
//!
//! The locale data is compiled in (ICU4X's `compiled_data`); a program carries only the data
//! of the formatters it uses.

mod compact;
mod datetime;
mod error;
mod list;
mod locale;
mod number;
mod plural;
mod relative;
mod unit;

pub use compact::{CompactDisplay, CompactNumberFormatter};
pub use datetime::{
    DateFields, DateTime, DateTimeFormatter, DateTimeOptions, HourCycle, Length, TimePrecision,
};
pub use error::{IntlError, Result};
pub use icu::locale::Locale;
pub use list::{ListFormatter, ListStyle, ListType};
pub use locale::{Direction, direction, negotiate, parse_locale};
pub use number::{Grouping, Number, NumberFormatter, NumberOptions, NumberStyle, SignDisplay};
pub use plural::{PluralCategory, PluralKind, PluralRules};
pub use relative::{
    RelativeTimeFormatter, RelativeTimeNumeric, RelativeTimeOptions, RelativeTimeStyle,
    RelativeTimeUnit,
};
pub use unit::{UnitDisplay, UnitFormatter, UnitOptions};

/// Runs the README's examples as doctests.
#[cfg(doctest)]
#[doc = include_str!("../../../README.md")]
struct ReadmeExamples;
