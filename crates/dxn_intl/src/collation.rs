use std::cmp::Ordering;

use icu::collator::options::{AlternateHandling, CollatorOptions as IcuCollatorOptions, Strength};
use icu::collator::preferences::{CollationCaseFirst, CollationNumericOrdering};
use icu::collator::{Collator as IcuCollator, CollatorBorrowed, CollatorPreferences};
use icu::locale::Locale;

use crate::Result;

/// Which differences between two strings tell them apart.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Sensitivity {
    /// Only base letters: `a` = `á` = `A`.
    Base,
    /// Base letters and accents: `a` ≠ `á`, `a` = `A`.
    Accent,
    /// Base letters and case: `a` ≠ `A`, `a` = `á`.
    Case,
    /// Base letters, accents and case: `a` ≠ `á` ≠ `A`.
    #[default]
    Variant,
}

/// Which case sorts first between strings that differ only in case.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CaseFirst {
    /// As the locale does (lowercase first in most).
    #[default]
    Locale,
    /// `A` before `a`.
    Upper,
    /// `a` before `A`.
    Lower,
}

/// How [`Collator`] compares strings; the defaults are `Intl.Collator`'s.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CollatorOptions {
    /// The differences that count.
    pub sensitivity: Sensitivity,
    /// Digits compare as numbers: `"2"` before `"10"`.
    pub numeric: bool,
    /// Which case sorts first.
    pub case_first: CaseFirst,
    /// Spaces and punctuation are ignored: `"ab"` = `"a-b"`.
    pub ignore_punctuation: bool,
}

/// Compares and sorts strings as the locale's alphabet orders them: `č` after `c` in Croatian,
/// `ä` with `a` in German and after `z` in Swedish, as `Intl.Collator` does.
///
/// ```
/// use dxn_intl::{Collator, CollatorOptions, parse_locale};
///
/// let hr = Collator::new(&parse_locale("hr")?, CollatorOptions::default())?;
/// let mut names = vec!["Čakovec", "Zagreb", "Cres", "Dubrovnik"];
/// hr.sort(&mut names);
/// assert_eq!(names, ["Cres", "Čakovec", "Dubrovnik", "Zagreb"]);
///
/// let options = CollatorOptions { numeric: true, ..CollatorOptions::default() };
/// let files = Collator::new(&parse_locale("en")?, options)?;
/// let mut list = vec!["file10", "file2", "file1"];
/// files.sort(&mut list);
/// assert_eq!(list, ["file1", "file2", "file10"]);
/// # Ok::<(), dxn_intl::IntlError>(())
/// ```
#[derive(Debug)]
pub struct Collator {
    collator: CollatorBorrowed<'static>,
}

impl Collator {
    /// A collator for `locale`; its `-u-co-` collation (`de-u-co-phonebk`), `-u-kn-` and
    /// `-u-kf-` extensions are honored unless `options` say otherwise.
    pub fn new(locale: &Locale, options: CollatorOptions) -> Result<Self> {
        let mut prefs = CollatorPreferences::from(locale);
        if options.numeric {
            prefs.numeric_ordering = Some(CollationNumericOrdering::True);
        }
        match options.case_first {
            CaseFirst::Locale => {}
            CaseFirst::Upper => prefs.case_first = Some(CollationCaseFirst::Upper),
            CaseFirst::Lower => prefs.case_first = Some(CollationCaseFirst::Lower),
        }
        let mut icu_options = IcuCollatorOptions::default();
        icu_options.strength = Some(match options.sensitivity {
            Sensitivity::Base => Strength::Primary,
            Sensitivity::Accent => Strength::Secondary,
            Sensitivity::Case | Sensitivity::Variant => Strength::Tertiary,
        });
        if options.sensitivity == Sensitivity::Case {
            // Case, without accents: primary strength with the case level on.
            icu_options.strength = Some(Strength::Primary);
            icu_options.case_level = Some(icu::collator::options::CaseLevel::On);
        }
        if options.ignore_punctuation {
            icu_options.alternate_handling = Some(AlternateHandling::Shifted);
        }
        Ok(Self {
            collator: IcuCollator::try_new(prefs, icu_options)?,
        })
    }

    /// How `a` sorts against `b`.
    pub fn compare(&self, a: &str, b: &str) -> Ordering {
        self.collator.compare(a, b)
    }

    /// Sorts `items` in the locale's order; equal items keep their order.
    pub fn sort<T: AsRef<str>>(&self, items: &mut [T]) {
        items.sort_by(|a, b| self.compare(a.as_ref(), b.as_ref()));
    }

    /// Sorts `items` by the text `key` gives each, in the locale's order; equal items keep
    /// their order.
    pub fn sort_by_key<T, K: AsRef<str>>(&self, items: &mut [T], mut key: impl FnMut(&T) -> K) {
        items.sort_by(|a, b| self.compare(key(a).as_ref(), key(b).as_ref()));
    }
}
