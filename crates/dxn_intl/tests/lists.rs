//! Lists joined with the locale's separators and words: conjunctions, disjunctions and units.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dxn_intl::{ListFormatter, ListStyle, ListType, parse_locale};

fn list(tag: &str, list_type: ListType, style: ListStyle, items: &[&str]) -> String {
    ListFormatter::new(&parse_locale(tag).unwrap(), list_type, style)
        .unwrap()
        .format(items)
}

#[test]
fn conjunctions() {
    let and = |items: &[&str]| list("en", ListType::And, ListStyle::Long, items);
    assert_eq!(and(&[]), "");
    assert_eq!(and(&["Ana"]), "Ana");
    assert_eq!(and(&["Ana", "Iva"]), "Ana and Iva");
    assert_eq!(and(&["Ana", "Iva", "Marko"]), "Ana, Iva, and Marko");
    assert_eq!(
        list(
            "en-GB",
            ListType::And,
            ListStyle::Long,
            &["Ana", "Iva", "Marko"]
        ),
        "Ana, Iva and Marko"
    );
    assert_eq!(
        list("en", ListType::And, ListStyle::Short, &["A", "B", "C"]),
        "A, B, & C"
    );
    assert_eq!(
        list(
            "hr",
            ListType::And,
            ListStyle::Long,
            &["Ana", "Iva", "Marko"]
        ),
        "Ana, Iva i Marko"
    );
    assert_eq!(
        list("ja", ListType::And, ListStyle::Long, &["りんご", "みかん"]),
        "りんご、みかん"
    );
}

#[test]
fn disjunctions_and_units() {
    assert_eq!(
        list(
            "en",
            ListType::Or,
            ListStyle::Long,
            &["tea", "coffee", "juice"]
        ),
        "tea, coffee, or juice"
    );
    // Spanish writes "u" before a word that starts with an "o" sound, and "e" before "i".
    assert_eq!(
        list("es", ListType::Or, ListStyle::Long, &["siete", "ocho"]),
        "siete u ocho"
    );
    assert_eq!(
        list("es", ListType::And, ListStyle::Long, &["España", "Italia"]),
        "España e Italia"
    );
    assert_eq!(
        list(
            "en",
            ListType::Unit,
            ListStyle::Long,
            &["3 feet", "7 inches"]
        ),
        "3 feet, 7 inches"
    );
    assert_eq!(
        list("en", ListType::Unit, ListStyle::Narrow, &["3′", "7″"]),
        "3′ 7″"
    );
}

#[test]
fn items_are_any_text() {
    let formatter =
        ListFormatter::new(&parse_locale("en").unwrap(), ListType::And, ListStyle::Long).unwrap();
    let names = vec![String::from("Ana"), String::from("Iva")];
    assert_eq!(formatter.format(&names), "Ana and Iva");
    assert_eq!(
        formatter.format(names.iter().map(String::as_str)),
        "Ana and Iva"
    );
}
