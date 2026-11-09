use icu::locale::Locale;
use icu_experimental::dimension::units::categorized_formatter::CategorizedFormatter;
use icu_experimental::dimension::units::options::{UnitsFormatterOptions, Width};
use icu_experimental::measure::category::{Area, Duration, Length, Mass, Volume};

use crate::number::{fraction_digits, round_fraction};
use crate::{IntlError, Number, Result};

/// How long a unit's name is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum UnitDisplay {
    /// "5 km".
    #[default]
    Short,
    /// "5 kilometers".
    Long,
    /// "5km", where the locale has a narrower form.
    Narrow,
}

/// How [`UnitFormatter`] shows amounts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct UnitOptions {
    /// How long the unit's name is.
    pub display: UnitDisplay,
    /// Trailing zeros pad the fraction to this many digits (`0` by default).
    pub minimum_fraction_digits: Option<u8>,
    /// The fraction is rounded, half away from zero, to this many digits (`3` by default).
    pub maximum_fraction_digits: Option<u8>,
}

impl UnitOptions {
    /// The unit's name at `display`, with the default digits.
    pub fn new(display: UnitDisplay) -> Self {
        Self {
            display,
            ..Self::default()
        }
    }
}

/// The CLDR units [`UnitFormatter`] knows: the ones ICU4X has names for, by category and by the
/// data set that has their names in most locales.
const UNITS: &[(&str, Category, Tier)] = &[
    ("kilometer", Category::Length, Tier::Core),
    ("meter", Category::Length, Tier::Core),
    ("decimeter", Category::Length, Tier::Outlier),
    ("centimeter", Category::Length, Tier::Core),
    ("millimeter", Category::Length, Tier::Core),
    ("micrometer", Category::Length, Tier::Outlier),
    ("nanometer", Category::Length, Tier::Outlier),
    ("picometer", Category::Length, Tier::Outlier),
    ("mile", Category::Length, Tier::Core),
    ("yard", Category::Length, Tier::Extended),
    ("foot", Category::Length, Tier::Core),
    ("inch", Category::Length, Tier::Core),
    ("parsec", Category::Length, Tier::Outlier),
    ("light-year", Category::Length, Tier::Outlier),
    ("astronomical-unit", Category::Length, Tier::Outlier),
    ("furlong", Category::Length, Tier::Outlier),
    ("fathom", Category::Length, Tier::Outlier),
    ("nautical-mile", Category::Length, Tier::Outlier),
    ("mile-scandinavian", Category::Length, Tier::Extended),
    ("point", Category::Length, Tier::Outlier),
    ("solar-radius", Category::Length, Tier::Outlier),
    ("earth-radius", Category::Length, Tier::Outlier),
    ("square-kilometer", Category::Area, Tier::Core),
    ("hectare", Category::Area, Tier::Core),
    ("square-meter", Category::Area, Tier::Core),
    ("square-centimeter", Category::Area, Tier::Core),
    ("square-mile", Category::Area, Tier::Core),
    ("acre", Category::Area, Tier::Core),
    ("square-yard", Category::Area, Tier::Outlier),
    ("square-foot", Category::Area, Tier::Core),
    ("square-inch", Category::Area, Tier::Core),
    ("dunam", Category::Area, Tier::Outlier),
    ("cubic-kilometer", Category::Volume, Tier::Outlier),
    ("cubic-meter", Category::Volume, Tier::Core),
    ("cubic-centimeter", Category::Volume, Tier::Core),
    ("cubic-mile", Category::Volume, Tier::Outlier),
    ("cubic-yard", Category::Volume, Tier::Outlier),
    ("cubic-foot", Category::Volume, Tier::Core),
    ("cubic-inch", Category::Volume, Tier::Core),
    ("megaliter", Category::Volume, Tier::Outlier),
    ("hectoliter", Category::Volume, Tier::Outlier),
    ("liter", Category::Volume, Tier::Core),
    ("deciliter", Category::Volume, Tier::Outlier),
    ("centiliter", Category::Volume, Tier::Outlier),
    ("milliliter", Category::Volume, Tier::Core),
    ("pint-metric", Category::Volume, Tier::Outlier),
    ("cup-metric", Category::Volume, Tier::Outlier),
    ("acre-foot", Category::Volume, Tier::Outlier),
    ("bushel", Category::Volume, Tier::Outlier),
    ("gallon", Category::Volume, Tier::Core),
    ("gallon-imperial", Category::Volume, Tier::Extended),
    ("quart", Category::Volume, Tier::Core),
    ("quart-imperial", Category::Volume, Tier::Outlier),
    ("pint", Category::Volume, Tier::Core),
    ("pint-imperial", Category::Volume, Tier::Outlier),
    ("cup", Category::Volume, Tier::Core),
    ("fluid-ounce", Category::Volume, Tier::Core),
    ("fluid-ounce-imperial", Category::Volume, Tier::Extended),
    ("fluid-ounce-metric", Category::Volume, Tier::Outlier),
    ("tablespoon", Category::Volume, Tier::Core),
    ("teaspoon", Category::Volume, Tier::Core),
    ("barrel", Category::Volume, Tier::Core),
    ("dessert-spoon", Category::Volume, Tier::Outlier),
    ("dessert-spoon-imperial", Category::Volume, Tier::Outlier),
    ("drop", Category::Volume, Tier::Outlier),
    ("dram", Category::Volume, Tier::Outlier),
    ("jigger", Category::Volume, Tier::Outlier),
    ("pinch", Category::Volume, Tier::Outlier),
    ("tonne", Category::Mass, Tier::Core),
    ("kilogram", Category::Mass, Tier::Core),
    ("gram", Category::Mass, Tier::Core),
    ("milligram", Category::Mass, Tier::Core),
    ("microgram", Category::Mass, Tier::Core),
    ("ton", Category::Mass, Tier::Core),
    ("stone", Category::Mass, Tier::Extended),
    ("pound", Category::Mass, Tier::Core),
    ("ounce", Category::Mass, Tier::Core),
    ("ounce-troy", Category::Mass, Tier::Outlier),
    ("carat", Category::Mass, Tier::Outlier),
    ("dalton", Category::Mass, Tier::Outlier),
    ("earth-mass", Category::Mass, Tier::Outlier),
    ("solar-mass", Category::Mass, Tier::Outlier),
    ("grain", Category::Mass, Tier::Outlier),
    ("century", Category::Duration, Tier::Outlier),
    ("decade", Category::Duration, Tier::Outlier),
    ("year", Category::Duration, Tier::Outlier),
    ("quarter", Category::Duration, Tier::Outlier),
    ("month", Category::Duration, Tier::Outlier),
    ("week", Category::Duration, Tier::Outlier),
    ("day", Category::Duration, Tier::Core),
    ("hour", Category::Duration, Tier::Core),
    ("minute", Category::Duration, Tier::Core),
    ("second", Category::Duration, Tier::Core),
    ("millisecond", Category::Duration, Tier::Core),
    ("microsecond", Category::Duration, Tier::Core),
    ("nanosecond", Category::Duration, Tier::Core),
];

/// ICU4X splits a category's unit names into three data sets: the units a locale's region uses
/// (core), others in common use (extended) and the rest (outlier). Which set has a unit can
/// depend on the locale (`foot` is core in `en`, extended in `ja`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tier {
    Core,
    Extended,
    Outlier,
}

impl Tier {
    /// The sets to look in: this one first, then the others.
    fn order(self) -> [Tier; 3] {
        match self {
            Tier::Core => [Tier::Core, Tier::Extended, Tier::Outlier],
            Tier::Extended => [Tier::Extended, Tier::Core, Tier::Outlier],
            Tier::Outlier => [Tier::Outlier, Tier::Extended, Tier::Core],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Category {
    Length,
    Area,
    Volume,
    Mass,
    Duration,
}

#[derive(Debug)]
enum Kind {
    Length(CategorizedFormatter<Length>),
    Area(CategorizedFormatter<Area>),
    Volume(CategorizedFormatter<Volume>),
    Mass(CategorizedFormatter<Mass>),
    Duration(CategorizedFormatter<Duration>),
}

/// ICU4X's formatter of the unit `id` in the category `$category`, from the first of the data
/// sets in `$tier`'s order that has its names; the template unit's id is replaced by the unit's,
/// which picks its names.
macro_rules! load {
    ($category:ident, $template:expr, $prefs:expr, $id:expr, $tier:expr, $options:expr) => {{
        let unit = || {
            let mut unit = $template;
            unit.unit.id = Some($id);
            unit
        };
        let mut result = Err(icu_provider::DataError::custom("no unit names"));
        for tier in $tier.order() {
            result = match tier {
                Tier::Core => {
                    CategorizedFormatter::<$category>::try_new_core($prefs, unit(), $options)
                }
                Tier::Extended => {
                    CategorizedFormatter::<$category>::try_new_extended($prefs, unit(), $options)
                }
                Tier::Outlier => {
                    CategorizedFormatter::<$category>::try_new_outlier($prefs, unit(), $options)
                }
            };
            if result.is_ok() {
                break;
            }
        }
        result
    }};
}

/// Formats amounts of a unit of measure: "5 km", "5 kilometers", "3,5 l", as
/// `Intl.NumberFormat`'s unit style does.
///
/// The units are CLDR's of length, area, volume, mass and duration (`"kilometer"`,
/// `"square-meter"`, `"liter"`, `"pound"`, `"hour"`, …: [`UnitFormatter::units`]), the ones
/// ICU4X has names for; the amount is shown as given, not converted.
///
/// ```
/// use dxn_intl::{UnitDisplay, UnitFormatter, UnitOptions, parse_locale};
///
/// let en = parse_locale("en")?;
/// let km = UnitFormatter::new(&en, "kilometer", UnitOptions::default())?;
/// assert_eq!(km.format(5.25)?, "5.25 km");
///
/// let long = UnitFormatter::new(&en, "mile", UnitOptions::new(UnitDisplay::Long))?;
/// assert_eq!(long.format(1)?, "1 mile");
/// assert_eq!(long.format(5)?, "5 miles");
///
/// let hr = UnitFormatter::new(&parse_locale("hr")?, "liter", UnitOptions::new(UnitDisplay::Long))?;
/// assert_eq!(hr.format(3)?, "3 litre");
/// # Ok::<(), dxn_intl::IntlError>(())
/// ```
#[derive(Debug)]
pub struct UnitFormatter {
    unit: &'static str,
    fraction: (u8, u8),
    kind: Kind,
}

impl UnitFormatter {
    /// A formatter of amounts of `unit`, a CLDR unit id such as `"kilometer"`, for `locale`
    /// (its `-u-nu-` numbering system is honored).
    pub fn new(locale: &Locale, unit: &str, options: UnitOptions) -> Result<Self> {
        let fraction = fraction_digits(
            options.minimum_fraction_digits,
            options.maximum_fraction_digits,
            3,
        )?;
        let trimmed = unit.trim();
        let Some(&(id, category, tier)) = UNITS.iter().find(|(id, _, _)| *id == trimmed) else {
            return Err(IntlError::InvalidOptions(format!(
                "`{unit}` is not a unit of length, area, volume, mass or duration that names \
                 are known for"
            )));
        };
        let mut icu_options = UnitsFormatterOptions::default();
        icu_options.width = match options.display {
            UnitDisplay::Short => Width::Short,
            UnitDisplay::Long => Width::Long,
            UnitDisplay::Narrow => Width::Narrow,
        };
        let prefs = locale.into();
        let kind = match category {
            Category::Length => Kind::Length(load!(
                Length,
                Length::meter(),
                prefs,
                id,
                tier,
                icu_options
            )?),
            Category::Area => Kind::Area(load!(
                Area,
                Area::square_meter(),
                prefs,
                id,
                tier,
                icu_options
            )?),
            Category::Volume => Kind::Volume(load!(
                Volume,
                Volume::liter(),
                prefs,
                id,
                tier,
                icu_options
            )?),
            Category::Mass => Kind::Mass(load!(Mass, Mass::gram(), prefs, id, tier, icu_options)?),
            Category::Duration => Kind::Duration(load!(
                Duration,
                Duration::second(),
                prefs,
                id,
                tier,
                icu_options
            )?),
        };
        Ok(Self {
            unit: id,
            fraction,
            kind,
        })
    }

    /// The units a formatter can be made for, by their CLDR ids.
    pub fn units() -> impl Iterator<Item = &'static str> {
        UNITS.iter().map(|(id, _, _)| *id)
    }

    /// The unit's CLDR id.
    pub fn unit(&self) -> &'static str {
        self.unit
    }

    /// `amount` of the unit, its fraction rounded and padded as the options say.
    pub fn format(&self, amount: impl TryInto<Number, Error: Into<IntlError>>) -> Result<String> {
        let Number(mut value) = amount.try_into().map_err(Into::into)?;
        round_fraction(&mut value, self.fraction);
        Ok(match &self.kind {
            Kind::Length(formatter) => formatter.format_fixed_decimal(&value).to_string(),
            Kind::Area(formatter) => formatter.format_fixed_decimal(&value).to_string(),
            Kind::Volume(formatter) => formatter.format_fixed_decimal(&value).to_string(),
            Kind::Mass(formatter) => formatter.format_fixed_decimal(&value).to_string(),
            Kind::Duration(formatter) => formatter.format_fixed_decimal(&value).to_string(),
        })
    }
}
