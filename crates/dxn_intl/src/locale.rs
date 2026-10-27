use icu::locale::fallback::{LocaleFallbackConfig, LocaleFallbacker};
use icu::locale::{Direction as IcuDirection, Locale, LocaleDirectionality};
use icu_provider::DataLocale;

use crate::{IntlError, Result};

/// Reads a BCP 47 language tag such as `en-US`, `sr-Latn-RS` or `de-CH-u-nu-latn`.
///
/// POSIX-style tags (`en_US`, `pt_BR.UTF-8`) are accepted too, as platforms report them: the
/// underscore becomes a hyphen and an encoding or modifier suffix is dropped.
pub fn parse_locale(tag: &str) -> Result<Locale> {
    let trimmed = tag.trim();
    let without_suffix = trimmed.split(['.', '@']).next().unwrap_or(trimmed);
    let normalized = without_suffix.replace('_', "-");
    Locale::try_from_str(&normalized).map_err(|_| IntlError::InvalidLocale(tag.to_string()))
}

/// The direction a locale's script is written in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Left to right, as Latin and Cyrillic.
    Ltr,
    /// Right to left, as Arabic and Hebrew.
    Rtl,
}

impl Direction {
    /// `"ltr"` or `"rtl"`, as HTML's `dir` and CSS's `direction` name them.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }
}

/// The direction `locale`'s script is written in, from its likely script when the tag names
/// none (`ar` is written in Arabic script, so right to left). Unknown languages are left to
/// right.
pub fn direction(locale: &Locale) -> Direction {
    let directionality = LocaleDirectionality::new_common();
    match directionality.get(&locale.id) {
        Some(IcuDirection::RightToLeft) => Direction::Rtl,
        _ => Direction::Ltr,
    }
}

/// Picks the locale to use from `available` for a user who prefers `requested`, in order.
///
/// Each requested locale is tried with CLDR's fallback chain (`de-AT` → `de`, `zh-Hant-HK` →
/// `zh-Hant`, …) before the next one; an available locale matches when its language and script
/// and, where the step still has one, region match. A step without a region takes the
/// available locale without one first, else any region's (`fr-FR` gets `fr-CA` over a
/// different language). The first match wins; with none,
/// `default` is the answer. Unicode extensions of the requested locale (`-u-hc-h23`) are kept
/// on the answer, so a user's preferences reach the formatters.
pub fn negotiate(requested: &[Locale], available: &[Locale], default: &Locale) -> Locale {
    let fallbacker = LocaleFallbacker::new().for_config(LocaleFallbackConfig::default());
    let scripts: Vec<_> = available.iter().map(likely_script).collect();
    for wanted in requested {
        let wanted_script = likely_script(wanted);
        let mut chain = fallbacker.fallback_for(DataLocale::from(wanted));
        loop {
            let step = chain.get();
            if step.is_unknown() {
                break;
            }
            let step_script = step.script.or(wanted_script);
            let mut matches = available
                .iter()
                .zip(&scripts)
                .filter_map(|(candidate, script)| {
                    (candidate.id.language == step.language
                        && *script == step_script
                        && (step.region.is_none() || candidate.id.region == step.region))
                        .then_some(candidate)
                });
            // At a step without a region, the locale without one goes before regional ones.
            let first = matches.next();
            let found = match first {
                Some(candidate) if candidate.id.region.is_some() => Some(
                    matches
                        .find(|other| other.id.region.is_none())
                        .unwrap_or(candidate),
                ),
                other => other,
            };
            if let Some(found) = found {
                let mut chosen = found.clone();
                chosen.extensions.unicode = wanted.extensions.unicode.clone();
                return chosen;
            }
            chain.step();
        }
    }
    default.clone()
}

/// The script `locale` names, or the one its language is most likely written in.
fn likely_script(locale: &Locale) -> Option<icu::locale::subtags::Script> {
    if locale.id.script.is_some() {
        return locale.id.script;
    }
    let mut expanded = locale.id.clone();
    icu::locale::LocaleExpander::new_common().maximize(&mut expanded);
    expanded.script
}
