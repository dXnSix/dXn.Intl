use icu::locale::Locale;
use icu::locale::subtags::{Language, Region, Script};
use icu_experimental::displaynames::multi::{
    LanguageDisplayNames, LocaleDisplayNamesFormatter, RegionDisplayNames, ScriptDisplayNames,
};
use icu_experimental::displaynames::{
    DisplayNamesOptions as IcuDisplayNamesOptions, LanguageDisplay, Style,
};

use crate::Result;

/// How long the names are.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DisplayNameStyle {
    /// "United Kingdom".
    #[default]
    Long,
    /// "UK", where the locale has a shorter name; else the long one.
    Short,
}

/// What [`DisplayNames`] shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DisplayNamesOptions {
    /// How long the names are.
    pub style: DisplayNameStyle,
    /// A locale with its dialect's own name: "British English" rather than "English (United
    /// Kingdom)".
    pub dialect: bool,
}

/// The names of languages, regions, scripts and locales in a locale's language, as
/// `Intl.DisplayNames` gives them: "Croatian", "Hrvatska", "Cyrillic", "English (Canada)".
///
/// ```
/// use dxn_intl::{DisplayNames, DisplayNamesOptions, parse_locale};
///
/// let en = DisplayNames::new(&parse_locale("en")?, DisplayNamesOptions::default())?;
/// assert_eq!(en.language("hr").as_deref(), Some("Croatian"));
/// assert_eq!(en.region("HR").as_deref(), Some("Croatia"));
/// assert_eq!(en.script("Cyrl").as_deref(), Some("Cyrillic"));
/// assert_eq!(en.locale(&parse_locale("en-CA")?), "English (Canada)");
///
/// let hr = DisplayNames::new(&parse_locale("hr")?, DisplayNamesOptions::default())?;
/// assert_eq!(hr.region("HR").as_deref(), Some("Hrvatska"));
/// # Ok::<(), dxn_intl::IntlError>(())
/// ```
#[derive(Debug)]
pub struct DisplayNames {
    languages: LanguageDisplayNames,
    regions: RegionDisplayNames,
    scripts: ScriptDisplayNames,
    locales: LocaleDisplayNamesFormatter,
}

impl DisplayNames {
    /// The names in `locale`'s language.
    pub fn new(locale: &Locale, options: DisplayNamesOptions) -> Result<Self> {
        let mut icu_options = IcuDisplayNamesOptions::default();
        icu_options.style = Some(match options.style {
            DisplayNameStyle::Long => Style::Long,
            DisplayNameStyle::Short => Style::Short,
        });
        icu_options.language_display = if options.dialect {
            LanguageDisplay::Dialect
        } else {
            LanguageDisplay::Standard
        };
        Ok(Self {
            languages: LanguageDisplayNames::try_new(locale.into(), icu_options)?,
            regions: RegionDisplayNames::try_new(locale.into(), icu_options)?,
            scripts: ScriptDisplayNames::try_new(locale.into(), icu_options)?,
            locales: LocaleDisplayNamesFormatter::try_new(locale.into(), icu_options)?,
        })
    }

    /// The name of the language `code` (`"hr"`, `"zh"`); `None` for a code that is not a
    /// language subtag or that the locale has no name for.
    pub fn language(&self, code: &str) -> Option<String> {
        let language = Language::try_from_str(code.trim()).ok()?;
        self.languages.of(language).map(str::to_owned)
    }

    /// The name of the region `code` (`"HR"`, `"419"`); `None` for a code that is not a region
    /// subtag or that the locale has no name for.
    pub fn region(&self, code: &str) -> Option<String> {
        let region = Region::try_from_str(code.trim()).ok()?;
        self.regions.of(region).map(str::to_owned)
    }

    /// The name of the script `code` (`"Latn"`, `"Cyrl"`); `None` for a code that is not a
    /// script subtag or that the locale has no name for.
    pub fn script(&self, code: &str) -> Option<String> {
        let script = Script::try_from_str(code.trim()).ok()?;
        self.scripts.of(script).map(str::to_owned)
    }

    /// The name of `locale`: its language's, with its script, region and variants in
    /// parentheses ("English (Canada)"), or its dialect's when the options ask ("Canadian
    /// English"). A subtag the locale has no name for shows as its code.
    pub fn locale(&self, locale: &Locale) -> String {
        self.locales.of(locale).into_owned()
    }
}
