//! Locale-aware plurals, numbers, dates, lists, names and sorting for Rust, on
//! [ICU4X](https://icu4x.unicode.org)'s CLDR data for every language it has, independent of any
//! UI framework and building for `wasm32-unknown-unknown`.
//!
//! - [`PluralRules`]: a number's cardinal or ordinal [`PluralCategory`] in a locale.
//! - [`NumberFormatter`]: decimal numbers, percentages and amounts of money, with digit,
//!   grouping and sign options; [`CompactNumberFormatter`]: compact notation ("1.2K");
//!   [`UnitFormatter`]: amounts of units of measure ("5 km", "3 hours").
//! - [`DateTimeFormatter`]: dates and times of day in the locale's calendar, words and clock;
//!   with the `tz` feature, [`DateTime::from_instant`] takes an instant's wall time in an IANA
//!   time zone. [`DateRangeFormatter`]: ranges of them; [`RelativeTimeFormatter`]: times
//!   relative to now ("in 3 days", "yesterday").
//! - [`ListFormatter`]: lists ("A, B, and C"); [`DisplayNames`]: the names of languages,
//!   regions, scripts and locales; [`Collator`]: comparing and sorting text in the locale's
//!   order.
//! - [`parse_locale`], [`negotiate`] and [`direction`]: reading a language tag, picking the
//!   best available locale for a user's list, and the direction a locale is written in.
//!
//! The locale data is compiled in (ICU4X's `compiled_data`); a program carries only the data
//! of the formatters it uses.

mod collation;
mod compact;
mod datetime;
mod display_names;
mod error;
mod list;
mod locale;
mod number;
mod plural;
mod relative;
mod unit;

pub use collation::{CaseFirst, Collator, CollatorOptions, Sensitivity};
pub use compact::{CompactDisplay, CompactNumberFormatter};
pub use datetime::{
    DateFields, DateRangeFormatter, DateTime, DateTimeFormatter, DateTimeOptions, HourCycle,
    Length, TimePrecision,
};
pub use display_names::{DisplayNameStyle, DisplayNames, DisplayNamesOptions};
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
