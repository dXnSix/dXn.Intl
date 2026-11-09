//! The names of languages, regions, scripts and locales in a locale's language.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dxn_intl::{DisplayNameStyle, DisplayNames, DisplayNamesOptions, parse_locale};

fn names(tag: &str) -> DisplayNames {
    DisplayNames::new(&parse_locale(tag).unwrap(), DisplayNamesOptions::default()).unwrap()
}

#[test]
fn languages_regions_and_scripts() {
    let en = names("en");
    assert_eq!(en.language("hr").as_deref(), Some("Croatian"));
    assert_eq!(en.language("de").as_deref(), Some("German"));
    assert_eq!(en.region("HR").as_deref(), Some("Croatia"));
    assert_eq!(en.region("419").as_deref(), Some("Latin America"));
    assert_eq!(en.script("Cyrl").as_deref(), Some("Cyrillic"));
    let de = names("de");
    assert_eq!(de.language("hr").as_deref(), Some("Kroatisch"));
    assert_eq!(de.region("HR").as_deref(), Some("Kroatien"));
    let hr = names("hr");
    assert_eq!(hr.language("hr").as_deref(), Some("hrvatski"));
    assert_eq!(hr.region("DE").as_deref(), Some("Njemačka"));
}

#[test]
fn codes_without_names() {
    let en = names("en");
    assert_eq!(en.language("not a code"), None);
    assert_eq!(en.region("XX"), None);
    assert_eq!(en.region("toolong"), None);
    assert_eq!(en.script("Zz1"), None);
    // CLDR names even the code for an unknown script.
    assert_eq!(en.script("Zzzz").as_deref(), Some("Unknown Script"));
    // Codes are read in any case, without surrounding spaces.
    assert_eq!(en.region(" hr ").as_deref(), Some("Croatia"));
}

#[test]
fn locales() {
    let en = names("en");
    assert_eq!(
        en.locale(&parse_locale("en-CA").unwrap()),
        "English (Canada)"
    );
    assert_eq!(
        en.locale(&parse_locale("sr-Latn-RS").unwrap()),
        "Serbian (Latin, Serbia)"
    );
    assert_eq!(en.locale(&parse_locale("hr").unwrap()), "Croatian");
    let dialects = DisplayNames::new(
        &parse_locale("en").unwrap(),
        DisplayNamesOptions {
            dialect: true,
            ..DisplayNamesOptions::default()
        },
    )
    .unwrap();
    assert_eq!(
        dialects.locale(&parse_locale("en-GB").unwrap()),
        "British English"
    );
    assert_eq!(
        names("de").locale(&parse_locale("en-GB").unwrap()),
        "Englisch (Vereinigtes Königreich)"
    );
}

#[test]
fn shorter_names() {
    let short = DisplayNames::new(
        &parse_locale("en").unwrap(),
        DisplayNamesOptions {
            style: DisplayNameStyle::Short,
            ..DisplayNamesOptions::default()
        },
    )
    .unwrap();
    assert_eq!(short.region("GB").as_deref(), Some("UK"));
    assert_eq!(short.region("HK").as_deref(), Some("Hong Kong"));
    assert_eq!(
        names("en").region("HK").as_deref(),
        Some("Hong Kong SAR China")
    );
    // A region with no shorter name has its name.
    assert_eq!(short.region("HR").as_deref(), Some("Croatia"));
}
