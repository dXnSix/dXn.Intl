use std::fmt;
use std::str::FromStr;

use icu::locale::Locale;
use icu::plurals::{
    PluralCategory as IcuCategory, PluralOperands, PluralRules as IcuRules, PluralRulesPreferences,
};

use crate::{IntlError, Number, Result};

/// Which plural rules: counting things, or ordering them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PluralKind {
    /// "1 day", "2 days".
    #[default]
    Cardinal,
    /// "1st", "2nd", "3rd", "4th".
    Ordinal,
}

/// A CLDR plural category.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PluralCategory {
    /// `zero`, as Arabic's and Latvian's.
    Zero,
    /// `one`.
    One,
    /// `two`, as Arabic's and Welsh's.
    Two,
    /// `few`, as Polish's 2–4 and Czech's.
    Few,
    /// `many`, as Polish's 5 and up and Arabic's.
    Many,
    /// `other`, which every language has.
    Other,
}

impl PluralCategory {
    /// Every category, from `zero` to `other`.
    pub const ALL: [Self; 6] = [
        Self::Zero,
        Self::One,
        Self::Two,
        Self::Few,
        Self::Many,
        Self::Other,
    ];

    /// The category's CLDR keyword: `"zero"`, `"one"`, `"two"`, `"few"`, `"many"` or `"other"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Zero => "zero",
            Self::One => "one",
            Self::Two => "two",
            Self::Few => "few",
            Self::Many => "many",
            Self::Other => "other",
        }
    }
}

impl fmt::Display for PluralCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for PluralCategory {
    type Err = IntlError;

    fn from_str(keyword: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|category| category.as_str() == keyword)
            .ok_or_else(|| {
                IntlError::InvalidOptions(format!("`{keyword}` is not a plural category"))
            })
    }
}

impl From<IcuCategory> for PluralCategory {
    fn from(category: IcuCategory) -> Self {
        match category {
            IcuCategory::Zero => Self::Zero,
            IcuCategory::One => Self::One,
            IcuCategory::Two => Self::Two,
            IcuCategory::Few => Self::Few,
            IcuCategory::Many => Self::Many,
            IcuCategory::Other => Self::Other,
        }
    }
}

/// A locale's plural rules, from CLDR for every language it has.
///
/// ```
/// use dxn_intl::{Number, PluralCategory, PluralKind, PluralRules, parse_locale};
///
/// let pl = PluralRules::new(&parse_locale("pl")?, PluralKind::Cardinal)?;
/// assert_eq!(pl.category(1), PluralCategory::One);
/// assert_eq!(pl.category(3), PluralCategory::Few);
/// assert_eq!(pl.category(5), PluralCategory::Many);
/// assert_eq!(pl.category("1.5".parse::<Number>()?), PluralCategory::Other);
///
/// let en = PluralRules::new(&parse_locale("en")?, PluralKind::Ordinal)?;
/// assert_eq!(en.category(22), PluralCategory::Two); // 22nd
/// # Ok::<(), dxn_intl::IntlError>(())
/// ```
#[derive(Debug)]
pub struct PluralRules {
    rules: IcuRules,
    kind: PluralKind,
}

impl PluralRules {
    /// The rules of `kind` for `locale`; a language CLDR has no rules for uses the root's
    /// (everything is `other`).
    pub fn new(locale: &Locale, kind: PluralKind) -> Result<Self> {
        let prefs = PluralRulesPreferences::from(locale);
        let rules = match kind {
            PluralKind::Cardinal => IcuRules::try_new_cardinal(prefs)?,
            PluralKind::Ordinal => IcuRules::try_new_ordinal(prefs)?,
        };
        Ok(Self { rules, kind })
    }

    /// Cardinal or ordinal.
    pub fn kind(&self) -> PluralKind {
        self.kind
    }

    /// The category of `number`, read with its visible fraction digits (`"1.0"` is not `1`).
    pub fn category(&self, number: impl Into<Number>) -> PluralCategory {
        let number = number.into();
        self.rules
            .category_for(PluralOperands::from(&number.0))
            .into()
    }

    /// The categories the locale's rules use, from `zero` to `other`.
    pub fn categories(&self) -> impl Iterator<Item = PluralCategory> + '_ {
        self.rules.categories().map(PluralCategory::from)
    }
}
