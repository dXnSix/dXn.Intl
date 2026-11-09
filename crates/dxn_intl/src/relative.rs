use icu::locale::Locale;
use icu_experimental::relativetime::options::Numeric;
use icu_experimental::relativetime::{
    RelativeTimeFormatter as IcuRelativeTimeFormatter, RelativeTimeFormatterOptions,
};

use crate::{IntlError, Number, Result};

/// The unit a relative time counts in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RelativeTimeUnit {
    /// "in 5 seconds".
    Second,
    /// "5 minutes ago".
    Minute,
    /// "in 2 hours".
    Hour,
    /// "in 3 days", "tomorrow".
    Day,
    /// "last week".
    Week,
    /// "in 2 months".
    Month,
    /// "next quarter".
    Quarter,
    /// "3 years ago".
    Year,
}

/// How long the words are.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RelativeTimeStyle {
    /// "in 3 months".
    #[default]
    Long,
    /// "in 3 mo.".
    Short,
    /// "in 3mo", where the locale has narrower words.
    Narrow,
}

/// Whether the locale's words for nearby times replace numbers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RelativeTimeNumeric {
    /// Always a number: "in 1 day", "1 day ago".
    #[default]
    Always,
    /// A word where the locale has one: "tomorrow", "yesterday", "now".
    Auto,
}

/// What [`RelativeTimeFormatter`] shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RelativeTimeOptions {
    /// How long the words are.
    pub style: RelativeTimeStyle,
    /// Numbers, or words for nearby times.
    pub numeric: RelativeTimeNumeric,
}

impl RelativeTimeOptions {
    /// The locale's words for nearby times, at `style`: "yesterday", "next week".
    pub fn auto(style: RelativeTimeStyle) -> Self {
        Self {
            style,
            numeric: RelativeTimeNumeric::Auto,
        }
    }
}

/// Formats a time relative to now in one unit: "in 3 days", "2 hours ago", "tomorrow", as
/// `Intl.RelativeTimeFormat` does. A negative value is in the past.
///
/// ```
/// use dxn_intl::{RelativeTimeFormatter, RelativeTimeOptions, RelativeTimeStyle, RelativeTimeUnit, parse_locale};
///
/// let en = parse_locale("en")?;
/// let days = RelativeTimeFormatter::new(&en, RelativeTimeUnit::Day, RelativeTimeOptions::default())?;
/// assert_eq!(days.format(3)?, "in 3 days");
/// assert_eq!(days.format(-1)?, "1 day ago");
///
/// let auto = RelativeTimeOptions::auto(RelativeTimeStyle::Long);
/// let days = RelativeTimeFormatter::new(&en, RelativeTimeUnit::Day, auto)?;
/// assert_eq!(days.format(-1)?, "yesterday");
/// # Ok::<(), dxn_intl::IntlError>(())
/// ```
#[derive(Debug)]
pub struct RelativeTimeFormatter {
    formatter: IcuRelativeTimeFormatter,
}

impl RelativeTimeFormatter {
    /// A formatter for `locale` counting in `unit` (its `-u-nu-` numbering system is honored).
    pub fn new(
        locale: &Locale,
        unit: RelativeTimeUnit,
        options: RelativeTimeOptions,
    ) -> Result<Self> {
        use RelativeTimeStyle::{Long, Narrow, Short};
        use RelativeTimeUnit::*;
        let mut icu_options = RelativeTimeFormatterOptions::default();
        icu_options.numeric = Some(match options.numeric {
            RelativeTimeNumeric::Always => Numeric::Always,
            RelativeTimeNumeric::Auto => Numeric::Auto,
        });
        let prefs = locale.into();
        let formatter = match (options.style, unit) {
            (Long, Second) => IcuRelativeTimeFormatter::try_new_long_second(prefs, icu_options),
            (Long, Minute) => IcuRelativeTimeFormatter::try_new_long_minute(prefs, icu_options),
            (Long, Hour) => IcuRelativeTimeFormatter::try_new_long_hour(prefs, icu_options),
            (Long, Day) => IcuRelativeTimeFormatter::try_new_long_day(prefs, icu_options),
            (Long, Week) => IcuRelativeTimeFormatter::try_new_long_week(prefs, icu_options),
            (Long, Month) => IcuRelativeTimeFormatter::try_new_long_month(prefs, icu_options),
            (Long, Quarter) => IcuRelativeTimeFormatter::try_new_long_quarter(prefs, icu_options),
            (Long, Year) => IcuRelativeTimeFormatter::try_new_long_year(prefs, icu_options),
            (Short, Second) => IcuRelativeTimeFormatter::try_new_short_second(prefs, icu_options),
            (Short, Minute) => IcuRelativeTimeFormatter::try_new_short_minute(prefs, icu_options),
            (Short, Hour) => IcuRelativeTimeFormatter::try_new_short_hour(prefs, icu_options),
            (Short, Day) => IcuRelativeTimeFormatter::try_new_short_day(prefs, icu_options),
            (Short, Week) => IcuRelativeTimeFormatter::try_new_short_week(prefs, icu_options),
            (Short, Month) => IcuRelativeTimeFormatter::try_new_short_month(prefs, icu_options),
            (Short, Quarter) => IcuRelativeTimeFormatter::try_new_short_quarter(prefs, icu_options),
            (Short, Year) => IcuRelativeTimeFormatter::try_new_short_year(prefs, icu_options),
            (Narrow, Second) => IcuRelativeTimeFormatter::try_new_narrow_second(prefs, icu_options),
            (Narrow, Minute) => IcuRelativeTimeFormatter::try_new_narrow_minute(prefs, icu_options),
            (Narrow, Hour) => IcuRelativeTimeFormatter::try_new_narrow_hour(prefs, icu_options),
            (Narrow, Day) => IcuRelativeTimeFormatter::try_new_narrow_day(prefs, icu_options),
            (Narrow, Week) => IcuRelativeTimeFormatter::try_new_narrow_week(prefs, icu_options),
            (Narrow, Month) => IcuRelativeTimeFormatter::try_new_narrow_month(prefs, icu_options),
            (Narrow, Quarter) => {
                IcuRelativeTimeFormatter::try_new_narrow_quarter(prefs, icu_options)
            }
            (Narrow, Year) => IcuRelativeTimeFormatter::try_new_narrow_year(prefs, icu_options),
        }?;
        Ok(Self { formatter })
    }

    /// `value` units from now: in the future when positive, in the past when negative (`-0` is
    /// in the past too).
    pub fn format(&self, value: impl TryInto<Number, Error: Into<IntlError>>) -> Result<String> {
        let Number(value) = value.try_into().map_err(Into::into)?;
        Ok(self.formatter.format(value).to_string())
    }
}
