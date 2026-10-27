//! Reading language tags, negotiating a locale and a locale's direction.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dxn_intl::{Direction, Locale, direction, negotiate, parse_locale};

fn locales(tags: &[&str]) -> Vec<Locale> {
    tags.iter().map(|tag| parse_locale(tag).unwrap()).collect()
}

#[test]
fn reads_bcp47_and_posix_tags() {
    assert_eq!(parse_locale("en-US").unwrap().to_string(), "en-US");
    assert_eq!(parse_locale("pt_BR.UTF-8").unwrap().to_string(), "pt-BR");
    assert_eq!(parse_locale("sr_RS@latin").unwrap().to_string(), "sr-RS");
    assert!(parse_locale("not a tag").is_err());
}

#[test]
fn negotiation_walks_each_requested_locale_before_the_next() {
    let available = locales(&["en", "de", "fr-CA", "zh-Hant"]);
    let default = parse_locale("en").unwrap();
    let pick =
        |requested: &[&str]| negotiate(&locales(requested), &available, &default).to_string();
    assert_eq!(pick(&["de-AT", "fr"]), "de");
    assert_eq!(pick(&["fr-CA"]), "fr-CA");
    // Another region of the same language beats the default.
    assert_eq!(pick(&["it", "fr-FR"]), "fr-CA");
    assert_eq!(pick(&["zh-TW"]), "zh-Hant");
    assert_eq!(pick(&["zh-CN"]), "en");
    let regional = locales(&["pt-PT", "pt"]);
    assert_eq!(
        negotiate(&locales(&["pt-BR"]), &regional, &default).to_string(),
        "pt"
    );
    assert_eq!(pick(&[]), "en");
    // A user's preferences stay on the answer.
    assert_eq!(pick(&["de-CH-u-hc-h12"]), "de-u-hc-h12");
}

#[test]
fn scripts_must_match() {
    let available = locales(&["sr", "en"]);
    let default = parse_locale("en").unwrap();
    assert_eq!(
        negotiate(&locales(&["sr-Latn"]), &available, &default).to_string(),
        "en"
    );
    assert_eq!(
        negotiate(&locales(&["sr-Cyrl-RS"]), &available, &default).to_string(),
        "sr"
    );
}

#[test]
fn directions() {
    assert_eq!(direction(&parse_locale("ar").unwrap()), Direction::Rtl);
    assert_eq!(direction(&parse_locale("he-IL").unwrap()), Direction::Rtl);
    assert_eq!(direction(&parse_locale("en").unwrap()), Direction::Ltr);
    assert_eq!(direction(&parse_locale("az-Arab").unwrap()), Direction::Rtl);
    assert_eq!(Direction::Rtl.as_str(), "rtl");
}
