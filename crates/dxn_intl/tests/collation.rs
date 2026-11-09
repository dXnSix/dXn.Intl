//! Comparing and sorting strings in the locale's alphabetical order, with the sensitivity,
//! numeric, case and punctuation options.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::cmp::Ordering;

use dxn_intl::{CaseFirst, Collator, CollatorOptions, Sensitivity, parse_locale};

fn collator(tag: &str, options: CollatorOptions) -> Collator {
    Collator::new(&parse_locale(tag).unwrap(), options).unwrap()
}

fn sorted(tag: &str, options: CollatorOptions, items: &[&'static str]) -> Vec<&'static str> {
    let mut items = items.to_vec();
    collator(tag, options).sort(&mut items);
    items
}

#[test]
fn each_locale_has_its_alphabet() {
    let default = CollatorOptions::default();
    assert_eq!(
        sorted(
            "hr",
            default,
            &[
                "Zagreb",
                "Čakovec",
                "Cres",
                "Ćuk",
                "Dubrovnik",
                "Šibenik",
                "Split"
            ]
        ),
        [
            "Cres",
            "Čakovec",
            "Ćuk",
            "Dubrovnik",
            "Split",
            "Šibenik",
            "Zagreb"
        ]
    );
    // German sorts "ä" with "a"; Swedish after "z".
    assert_eq!(
        sorted("de", default, &["zebra", "äpfel", "affe"]),
        ["affe", "äpfel", "zebra"]
    );
    assert_eq!(
        sorted("sv", default, &["zebra", "äpfel", "affe"]),
        ["affe", "zebra", "äpfel"]
    );
    // Byte order would put every capital first.
    assert_eq!(
        sorted("en", default, &["banana", "Apple", "cherry"]),
        ["Apple", "banana", "cherry"]
    );
}

#[test]
fn sensitivity() {
    let compare = |sensitivity, a: &str, b: &str| {
        collator(
            "en",
            CollatorOptions {
                sensitivity,
                ..CollatorOptions::default()
            },
        )
        .compare(a, b)
    };
    assert_eq!(compare(Sensitivity::Base, "a", "á"), Ordering::Equal);
    assert_eq!(compare(Sensitivity::Base, "a", "A"), Ordering::Equal);
    assert_ne!(compare(Sensitivity::Accent, "a", "á"), Ordering::Equal);
    assert_eq!(compare(Sensitivity::Accent, "a", "A"), Ordering::Equal);
    assert_eq!(compare(Sensitivity::Case, "a", "á"), Ordering::Equal);
    assert_ne!(compare(Sensitivity::Case, "a", "A"), Ordering::Equal);
    assert_ne!(compare(Sensitivity::Variant, "a", "á"), Ordering::Equal);
    assert_ne!(compare(Sensitivity::Variant, "a", "A"), Ordering::Equal);
    assert_eq!(compare(Sensitivity::Base, "a", "b"), Ordering::Less);
}

#[test]
fn numbers_case_and_punctuation() {
    let numeric = CollatorOptions {
        numeric: true,
        ..CollatorOptions::default()
    };
    assert_eq!(
        sorted("en", numeric, &["item 10", "item 2", "item 1"]),
        ["item 1", "item 2", "item 10"]
    );
    assert_eq!(
        sorted(
            "en",
            CollatorOptions::default(),
            &["item 10", "item 2", "item 1"]
        ),
        ["item 1", "item 10", "item 2"]
    );
    let upper = CollatorOptions {
        case_first: CaseFirst::Upper,
        ..CollatorOptions::default()
    };
    assert_eq!(sorted("en", upper, &["a", "A"]), ["A", "a"]);
    let lower = CollatorOptions {
        case_first: CaseFirst::Lower,
        ..CollatorOptions::default()
    };
    assert_eq!(sorted("en", lower, &["A", "a"]), ["a", "A"]);
    let ignore = CollatorOptions {
        ignore_punctuation: true,
        sensitivity: Sensitivity::Base,
        ..CollatorOptions::default()
    };
    assert_eq!(
        collator("en", ignore).compare("co-op", "coop"),
        Ordering::Equal
    );
}

#[test]
fn the_locales_extensions() {
    // German phone book order sorts "ä" as "ae".
    let phonebook = sorted(
        "de-u-co-phonebk",
        CollatorOptions::default(),
        &["Ärger", "Adler", "Affe"],
    );
    assert_eq!(phonebook, ["Adler", "Ärger", "Affe"]);
    // `-u-kn` turns numeric ordering on.
    assert_eq!(
        sorted("en-u-kn", CollatorOptions::default(), &["10", "9"]),
        ["9", "10"]
    );
}

#[test]
fn sorting_by_key_keeps_equal_items_in_order() {
    let en = collator(
        "en",
        CollatorOptions {
            sensitivity: Sensitivity::Base,
            ..CollatorOptions::default()
        },
    );
    let mut people = vec![("Émile", 1), ("bob", 2), ("Emile", 3), ("Bob", 4)];
    en.sort_by_key(&mut people, |(name, _)| *name);
    assert_eq!(people, [("bob", 2), ("Bob", 4), ("Émile", 1), ("Emile", 3)]);
}
