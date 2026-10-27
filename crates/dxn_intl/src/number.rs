use std::fmt;
use std::str::FromStr;

use fixed_decimal::{Decimal, FloatPrecision, SignedRoundingMode, UnsignedRoundingMode};
use icu::decimal::options::{DecimalFormatterOptions, GroupingStrategy};
use icu::decimal::{DecimalFormatter, DecimalFormatterPreferences};
use icu::locale::Locale;
use icu_experimental::dimension::currency::CurrencyType;
use icu_experimental::dimension::currency::formatter::CurrencyFormatter;
use icu_experimental::dimension::percent::formatter::PercentFormatter;

use crate::{IntlError, Result};

/// A decimal number, exact as written: `"1.50"` keeps its two fraction digits, which plural
/// rules read (`1.50` is "other" in English, `1` is "one").
///
/// Made from integers, from text, or from a finite float (`TryFrom<f64>`, the shortest text
/// that reads back as the same float).
#[derive(Clone, Debug, PartialEq)]
pub struct Number(pub(crate) Decimal);

impl Number {
    /// The ICU4X decimal this number holds.
    pub fn as_decimal(&self) -> &Decimal {
        &self.0
    }

    /// Whether the number has no fraction digits, trailing zeros included (`2.0` has one).
    pub fn is_integer(&self) -> bool {
        *self.0.absolute.magnitude_range().start() >= 0
    }

    /// The number as the nearest float.
    pub fn to_f64(&self) -> f64 {
        self.0.to_string().parse().unwrap_or(f64::NAN)
    }
}

impl From<Decimal> for Number {
    fn from(decimal: Decimal) -> Self {
        Self(decimal)
    }
}

macro_rules! from_integer {
    ($($ty:ty),*) => {$(
        impl From<$ty> for Number {
            fn from(value: $ty) -> Self {
                Self(Decimal::from(value))
            }
        }
    )*};
}
from_integer!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

impl TryFrom<f64> for Number {
    type Error = IntlError;

    fn try_from(value: f64) -> Result<Self> {
        Decimal::try_from_f64(value, FloatPrecision::RoundTrip)
            .map(Self)
            .map_err(|_| IntlError::InvalidNumber(value.to_string()))
    }
}

impl TryFrom<f32> for Number {
    type Error = IntlError;

    fn try_from(value: f32) -> Result<Self> {
        // Through the float's shortest text, so 0.1f32 is 0.1 and not 0.100000001490116.
        value.to_string().parse()
    }
}

impl FromStr for Number {
    type Err = IntlError;

    fn from_str(text: &str) -> Result<Self> {
        Decimal::try_from_str(text.trim())
            .map(Self)
            .map_err(|_| IntlError::InvalidNumber(text.to_string()))
    }
}

impl fmt::Display for Number {
    /// The number as plain ASCII text (`-1234.50`), not localized.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// What a number stands for, which decides the symbols around it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum NumberStyle {
    /// A plain number: `1,234.5`.
    #[default]
    Decimal,
    /// A fraction shown as a percentage: `0.25` is `25%`.
    Percent,
    /// An amount of money in the ISO 4217 currency (`"EUR"`), with its symbol and the
    /// currency's own fraction digits: `€1,234.50`.
    Currency(String),
}

/// When grouping separators are shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Grouping {
    /// As the locale does: in most locales from four digits on (`1,234`).
    #[default]
    Auto,
    /// Never: `1234567`.
    Never,
    /// Always, even where the locale would leave four digits ungrouped.
    Always,
    /// Only from five digits on: `1234`, `12,345`.
    Min2,
}

/// When the sign is shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SignDisplay {
    /// For negative numbers, negative zero included.
    #[default]
    Auto,
    /// Never.
    Never,
    /// For every number, `+` for positive ones and zero.
    Always,
    /// For every number but zero.
    ExceptZero,
    /// For negative numbers other than negative zero.
    Negative,
}

/// How [`NumberFormatter`] shows numbers; the defaults are ECMA-402's for a decimal number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumberOptions {
    /// Decimal, percent or currency.
    pub style: NumberStyle,
    /// Leading zeros pad the integer part to this many digits (`1` by default).
    pub minimum_integer_digits: u8,
    /// Trailing zeros pad the fraction to this many digits (`0` by default). Currencies always
    /// show their own digits (two for euros, none for yen) and ignore both fraction options.
    pub minimum_fraction_digits: Option<u8>,
    /// The fraction is rounded, half away from zero, to this many digits (`3` by default, `0`
    /// for percentages).
    pub maximum_fraction_digits: Option<u8>,
    /// Grouping separators.
    pub grouping: Grouping,
    /// The sign.
    pub sign_display: SignDisplay,
}

impl Default for NumberOptions {
    fn default() -> Self {
        Self {
            style: NumberStyle::Decimal,
            minimum_integer_digits: 1,
            minimum_fraction_digits: None,
            maximum_fraction_digits: None,
            grouping: Grouping::Auto,
            sign_display: SignDisplay::Auto,
        }
    }
}

impl NumberOptions {
    /// Options for a percentage (`0.25` is `25%`).
    pub fn percent() -> Self {
        Self {
            style: NumberStyle::Percent,
            ..Self::default()
        }
    }

    /// Options for an amount of money in `currency`, an ISO 4217 code such as `"USD"`.
    pub fn currency(currency: impl Into<String>) -> Self {
        Self {
            style: NumberStyle::Currency(currency.into()),
            ..Self::default()
        }
    }

    /// With exactly `digits` fraction digits.
    pub fn with_fraction_digits(mut self, digits: u8) -> Self {
        self.minimum_fraction_digits = Some(digits);
        self.maximum_fraction_digits = Some(digits);
        self
    }

    fn fraction_digits(&self) -> Result<(u8, u8)> {
        let default_max = match self.style {
            NumberStyle::Percent => 0,
            _ => 3,
        };
        let min = self.minimum_fraction_digits.unwrap_or(0);
        let max = self.maximum_fraction_digits.unwrap_or(default_max.max(min));
        if min > max {
            return Err(IntlError::InvalidOptions(format!(
                "minimum fraction digits {min} is above the maximum {max}"
            )));
        }
        if max > 100 || self.minimum_integer_digits > 100 {
            return Err(IntlError::InvalidOptions(
                "more than 100 digits".to_string(),
            ));
        }
        Ok((min, max))
    }
}

/// Formats numbers for a locale: its digits, decimal and grouping separators, and percent and
/// currency patterns.
///
/// ```
/// use dxn_intl::{NumberFormatter, NumberOptions, parse_locale};
///
/// let de = parse_locale("de-DE")?;
/// let numbers = NumberFormatter::new(&de, NumberOptions::default())?;
/// assert_eq!(numbers.format(1234.5)?, "1.234,5");
///
/// let percent = NumberFormatter::new(&de, NumberOptions::percent())?;
/// assert_eq!(percent.format(0.25)?, "25\u{a0}%");
/// # Ok::<(), dxn_intl::IntlError>(())
/// ```
#[derive(Debug)]
pub struct NumberFormatter {
    options: NumberOptions,
    fraction: (u8, u8),
    kind: Kind,
}

#[derive(Debug)]
enum Kind {
    Decimal(DecimalFormatter),
    Percent(Box<PercentFormatter<DecimalFormatter>>),
    Currency(Box<CurrencyFormatter<DecimalFormatter>>),
}

impl NumberFormatter {
    /// A formatter for `locale` (its `-u-nu-` numbering system is honored).
    pub fn new(locale: &Locale, options: NumberOptions) -> Result<Self> {
        let fraction = options.fraction_digits()?;
        let grouping = match options.grouping {
            Grouping::Auto => GroupingStrategy::Auto,
            Grouping::Never => GroupingStrategy::Never,
            Grouping::Always => GroupingStrategy::Always,
            Grouping::Min2 => GroupingStrategy::Min2,
        };
        let decimal_options = DecimalFormatterOptions::from(grouping);
        let decimal =
            DecimalFormatter::try_new(DecimalFormatterPreferences::from(locale), decimal_options)?;
        let kind = match &options.style {
            NumberStyle::Decimal => Kind::Decimal(decimal),
            NumberStyle::Percent => {
                Kind::Percent(Box::new(PercentFormatter::try_new_with_decimal_formatter(
                    locale.into(),
                    decimal,
                    Default::default(),
                )?))
            }
            NumberStyle::Currency(code) => {
                let code = currency_code(code)?;
                Kind::Currency(Box::new(CurrencyFormatter::try_new_symbol(
                    locale.into(),
                    code,
                    Default::default(),
                )?))
            }
        };
        Ok(Self {
            options,
            fraction,
            kind,
        })
    }

    /// The options this formatter was made with.
    pub fn options(&self) -> &NumberOptions {
        &self.options
    }

    /// `number` as the formatter shows it, before localization: scaled for percentages, rounded
    /// and padded to the fraction digits, padded to the integer digits.
    ///
    /// Plural selection reads this value, so "1.0 items" picks the form for `1.0`, not `1`.
    pub fn resolve(&self, number: impl TryInto<Number, Error: Into<IntlError>>) -> Result<Number> {
        let Number(mut value) = number.try_into().map_err(Into::into)?;
        if self.options.style == NumberStyle::Percent {
            value.multiply_pow10(2);
        }
        // A currency formatter rounds to the currency's own digits.
        if !matches!(self.options.style, NumberStyle::Currency(_)) {
            let (min, max) = self.fraction;
            value.round_with_mode(
                -i16::from(max),
                SignedRoundingMode::Unsigned(UnsignedRoundingMode::HalfExpand),
            );
            value.absolute.trim_end();
            value.absolute.pad_end(-i16::from(min));
        }
        value
            .absolute
            .pad_start(i16::from(self.options.minimum_integer_digits));
        value.apply_sign_display(match self.options.sign_display {
            SignDisplay::Auto => fixed_decimal::SignDisplay::Auto,
            SignDisplay::Never => fixed_decimal::SignDisplay::Never,
            SignDisplay::Always => fixed_decimal::SignDisplay::Always,
            SignDisplay::ExceptZero => fixed_decimal::SignDisplay::ExceptZero,
            SignDisplay::Negative => fixed_decimal::SignDisplay::Negative,
        });
        Ok(Number(value))
    }

    /// `number` formatted for the locale.
    pub fn format(&self, number: impl TryInto<Number, Error: Into<IntlError>>) -> Result<String> {
        let Number(value) = self.resolve(number)?;
        Ok(match &self.kind {
            Kind::Decimal(formatter) => formatter.format(&value).to_string(),
            Kind::Percent(formatter) => formatter.format(&value).to_string(),
            Kind::Currency(formatter) => formatter.format_fixed_decimal(&value).to_string(),
        })
    }
}

impl From<std::convert::Infallible> for IntlError {
    fn from(never: std::convert::Infallible) -> Self {
        match never {}
    }
}

fn currency_code(code: &str) -> Result<CurrencyType> {
    let trimmed = code.trim();
    if trimmed.len() != 3 || !trimmed.bytes().all(|b| b.is_ascii_alphabetic()) {
        return Err(IntlError::InvalidCurrency(code.to_string()));
    }
    CurrencyType::try_from_str(trimmed).map_err(|_| IntlError::InvalidCurrency(code.to_string()))
}
