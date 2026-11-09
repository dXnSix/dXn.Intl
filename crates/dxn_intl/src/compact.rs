use fixed_decimal::{SignedRoundingMode, UnsignedRoundingMode};
use icu::locale::Locale;
use icu_decimal::{CompactDecimalFormatter, options::CompactDecimalFormatterOptions};

use crate::{IntlError, Number, Result};

/// How [`CompactNumberFormatter`] writes the magnitude.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CompactDisplay {
    /// "1.2K", "3.4M".
    #[default]
    Short,
    /// "1.2 thousand", "3.4 million".
    Long,
}

/// Formats numbers in compact notation: "1.2K", "3.4 million", "12 Mio." — the magnitude as a
/// word or abbreviation, as CLDR's compact patterns say, and the number rounded as
/// `Intl.NumberFormat`'s compact notation does: half away from zero, to whole units of the
/// magnitude from two digits on (`15K`), else to two significant digits (`1.3M`, `0.012`).
///
/// ```
/// use dxn_intl::{CompactDisplay, CompactNumberFormatter, parse_locale};
///
/// let en = CompactNumberFormatter::new(&parse_locale("en")?, CompactDisplay::Short)?;
/// assert_eq!(en.format(1234)?, "1.2K");
/// assert_eq!(en.format(15_300_000)?, "15M");
///
/// let de = CompactNumberFormatter::new(&parse_locale("de")?, CompactDisplay::Long)?;
/// assert_eq!(de.format(3_400_000)?, "3,4 Millionen");
/// # Ok::<(), dxn_intl::IntlError>(())
/// ```
#[derive(Debug)]
pub struct CompactNumberFormatter {
    formatter: CompactDecimalFormatter,
}

impl CompactNumberFormatter {
    /// A formatter for `locale` (its `-u-nu-` numbering system is honored).
    pub fn new(locale: &Locale, display: CompactDisplay) -> Result<Self> {
        let options = CompactDecimalFormatterOptions::default();
        let formatter = match display {
            CompactDisplay::Short => CompactDecimalFormatter::try_new_short(locale.into(), options),
            CompactDisplay::Long => CompactDecimalFormatter::try_new_long(locale.into(), options),
        }?;
        Ok(Self { formatter })
    }

    /// `number` in compact notation, rounded to the digits the notation shows.
    pub fn format(&self, number: impl TryInto<Number, Error: Into<IntlError>>) -> Result<String> {
        let Number(mut value) = number.try_into().map_err(Into::into)?;
        // ICU4X rounds half to even; round half away from zero first, as the other formatters
        // and ECMA-402 do, so it has nothing left to round.
        let magnitude = value.absolute.nonzero_magnitude_start();
        let exponent = i16::from(self.formatter.compact_exponent_for_magnitude(magnitude));
        let position = if magnitude - exponent >= 1 {
            exponent
        } else {
            magnitude - 1
        };
        value.round_with_mode(
            position,
            SignedRoundingMode::Unsigned(UnsignedRoundingMode::HalfExpand),
        );
        value.absolute.trim_end();
        Ok(self.formatter.format_to_string(&value))
    }
}
