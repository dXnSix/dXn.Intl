//! Plural categories across languages, cardinal and ordinal.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dxn_intl::{Number, PluralCategory as P, PluralKind, PluralRules, parse_locale};

fn cardinal(tag: &str) -> PluralRules {
    PluralRules::new(&parse_locale(tag).unwrap(), PluralKind::Cardinal).unwrap()
}

fn number(text: &str) -> Number {
    text.parse().unwrap()
}

#[test]
fn english_counts_one_and_other() {
    let en = cardinal("en");
    assert_eq!(en.category(1), P::One);
    assert_eq!(en.category(0), P::Other);
    assert_eq!(en.category(2), P::Other);
    // Visible fraction digits count: "1.0 items".
    assert_eq!(en.category(number("1.0")), P::Other);
}

#[test]
fn slavic_languages_have_few_and_many() {
    let pl = cardinal("pl");
    assert_eq!(pl.category(1), P::One);
    assert_eq!(pl.category(2), P::Few);
    assert_eq!(pl.category(5), P::Many);
    assert_eq!(pl.category(22), P::Few);
    assert_eq!(pl.category(12), P::Many);

    let hr = cardinal("hr");
    assert_eq!(hr.category(21), P::One);
    assert_eq!(hr.category(23), P::Few);
    assert_eq!(hr.category(25), P::Other);
    assert_eq!(hr.category(number("0.1")), P::One);

    let ru = cardinal("ru");
    assert_eq!(ru.category(11), P::Many);
    assert_eq!(ru.category(number("1.5")), P::Other);
}

#[test]
fn arabic_uses_all_six() {
    let ar = cardinal("ar");
    assert_eq!(ar.category(0), P::Zero);
    assert_eq!(ar.category(1), P::One);
    assert_eq!(ar.category(2), P::Two);
    assert_eq!(ar.category(3), P::Few);
    assert_eq!(ar.category(11), P::Many);
    assert_eq!(ar.category(100), P::Other);
    assert_eq!(ar.categories().count(), 6);
}

#[test]
fn languages_without_plurals_are_other() {
    let ja = cardinal("ja");
    assert_eq!(ja.category(1), P::Other);
    assert_eq!(ja.categories().collect::<Vec<_>>(), vec![P::Other]);
}

#[test]
fn regional_and_unknown_tags_fall_back() {
    assert_eq!(cardinal("pt-PT").category(0), P::Other);
    assert_eq!(cardinal("pt-BR").category(0), P::One);
    assert_eq!(cardinal("de-AT").category(1), P::One);
    // No rules for this language: the root's.
    assert_eq!(cardinal("tlh").category(1), P::Other);
}

#[test]
fn ordinals() {
    let en = PluralRules::new(&parse_locale("en").unwrap(), PluralKind::Ordinal).unwrap();
    assert_eq!(en.kind(), PluralKind::Ordinal);
    assert_eq!(en.category(1), P::One);
    assert_eq!(en.category(2), P::Two);
    assert_eq!(en.category(3), P::Few);
    assert_eq!(en.category(4), P::Other);
    assert_eq!(en.category(11), P::Other);
    assert_eq!(en.category(21), P::One);
}

#[test]
fn category_keywords_round_trip() {
    for category in P::ALL {
        assert_eq!(category.as_str().parse::<P>().unwrap(), category);
        assert_eq!(category.to_string(), category.as_str());
    }
    assert!("several".parse::<P>().is_err());
}
