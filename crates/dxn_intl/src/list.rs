use icu::list::ListFormatter as IcuListFormatter;
use icu::list::options::{ListFormatterOptions, ListLength};
use icu::locale::Locale;

use crate::Result;

/// What a list's items are to each other.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ListType {
    /// All of them: "A, B, and C".
    #[default]
    And,
    /// One of them: "A, B, or C".
    Or,
    /// Parts of one measure: "3 feet, 7 inches".
    Unit,
}

/// How long the connecting words are.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ListStyle {
    /// "A, B, and C".
    #[default]
    Long,
    /// "A, B, & C".
    Short,
    /// "A, B, C".
    Narrow,
}

/// Joins items into a list with the locale's separators and words, as `Intl.ListFormat` does.
///
/// ```
/// use dxn_intl::{ListFormatter, ListStyle, ListType, parse_locale};
///
/// let en = ListFormatter::new(&parse_locale("en")?, ListType::And, ListStyle::Long)?;
/// assert_eq!(en.format(["Ana", "Iva", "Marko"]), "Ana, Iva, and Marko");
///
/// let es = ListFormatter::new(&parse_locale("es")?, ListType::Or, ListStyle::Long)?;
/// assert_eq!(es.format(["siete", "ocho"]), "siete u ocho"); // "o" becomes "u" before an "o"
/// # Ok::<(), dxn_intl::IntlError>(())
/// ```
#[derive(Debug)]
pub struct ListFormatter {
    formatter: IcuListFormatter,
}

impl ListFormatter {
    /// A formatter for `locale`.
    pub fn new(locale: &Locale, list_type: ListType, style: ListStyle) -> Result<Self> {
        let options = ListFormatterOptions::default().with_length(match style {
            ListStyle::Long => ListLength::Wide,
            ListStyle::Short => ListLength::Short,
            ListStyle::Narrow => ListLength::Narrow,
        });
        let formatter = match list_type {
            ListType::And => IcuListFormatter::try_new_and(locale.into(), options),
            ListType::Or => IcuListFormatter::try_new_or(locale.into(), options),
            ListType::Unit => IcuListFormatter::try_new_unit(locale.into(), options),
        }?;
        Ok(Self { formatter })
    }

    /// `items` joined in their order; no items is the empty text.
    pub fn format<I>(&self, items: I) -> String
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let items: Vec<I::Item> = items.into_iter().collect();
        self.formatter
            .format_to_string(items.iter().map(|item| item.as_ref()))
    }
}
