//! Amounts of units of measure: CLDR's units of length, area, volume, mass and duration, in the
//! locale's names and plural forms.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dxn_intl::{IntlError, Number, UnitDisplay, UnitFormatter, UnitOptions, parse_locale};

fn unit(
    tag: &str,
    unit: &str,
    display: UnitDisplay,
    amount: impl TryInto<Number, Error: Into<IntlError>>,
) -> String {
    UnitFormatter::new(&parse_locale(tag).unwrap(), unit, UnitOptions::new(display))
        .unwrap()
        .format(amount)
        .unwrap()
}

#[test]
fn short_long_and_narrow_names() {
    assert_eq!(unit("en", "kilometer", UnitDisplay::Short, 5), "5 km");
    assert_eq!(unit("en", "mile", UnitDisplay::Long, 1), "1 mile");
    assert_eq!(unit("en", "mile", UnitDisplay::Long, 26.2), "26.2 miles");
    assert_eq!(unit("en", "foot", UnitDisplay::Narrow, 6), "6′");
    assert_eq!(unit("en", "kilogram", UnitDisplay::Short, 2.5), "2.5 kg");
    assert_eq!(unit("en", "pound", UnitDisplay::Long, 1), "1 pound");
    assert_eq!(unit("en", "hour", UnitDisplay::Long, 3), "3 hours");
    assert_eq!(
        unit("en", "square-meter", UnitDisplay::Short, 120),
        "120 m²"
    );
    assert_eq!(unit("de", "liter", UnitDisplay::Short, 1.5), "1,5 l");
}

#[test]
fn names_follow_the_locales_plurals_and_separators() {
    assert_eq!(unit("hr", "liter", UnitDisplay::Long, 1), "1 litra");
    assert_eq!(unit("hr", "liter", UnitDisplay::Long, 3), "3 litre");
    assert_eq!(unit("hr", "liter", UnitDisplay::Long, 5), "5 litara");
    assert_eq!(unit("de", "kilogram", UnitDisplay::Short, 2.5), "2,5 kg");
    assert_eq!(unit("de", "hour", UnitDisplay::Long, 2), "2 Stunden");
    assert_eq!(unit("ja", "kilometer", UnitDisplay::Short, 5), "5 km");
    assert_eq!(
        unit("ar-u-nu-arab", "kilogram", UnitDisplay::Short, 3),
        "٣ كغم"
    );
}

#[test]
fn fraction_digits() {
    let en = parse_locale("en").unwrap();
    let default = UnitFormatter::new(&en, "kilometer", UnitOptions::default()).unwrap();
    assert_eq!(default.format(1.23456).unwrap(), "1.235 km");
    assert_eq!(default.format(2.5).unwrap(), "2.5 km");
    let options = UnitOptions {
        minimum_fraction_digits: Some(1),
        maximum_fraction_digits: Some(1),
        ..UnitOptions::default()
    };
    let one = UnitFormatter::new(&en, "kilometer", options).unwrap();
    assert_eq!(one.format(3).unwrap(), "3.0 km");
    assert_eq!(one.format(2.25).unwrap(), "2.3 km");
    let bad = UnitOptions {
        minimum_fraction_digits: Some(3),
        maximum_fraction_digits: Some(1),
        ..UnitOptions::default()
    };
    assert!(matches!(
        UnitFormatter::new(&en, "kilometer", bad),
        Err(IntlError::InvalidOptions(_))
    ));
}

#[test]
fn every_listed_unit_has_names_in_every_display() {
    for tag in ["en", "en-GB", "de", "hr", "ja", "ar", "zh", "hi"] {
        let locale = parse_locale(tag).unwrap();
        for id in UnitFormatter::units() {
            for display in [UnitDisplay::Short, UnitDisplay::Long, UnitDisplay::Narrow] {
                let formatter = UnitFormatter::new(&locale, id, UnitOptions::new(display))
                    .unwrap_or_else(|error| panic!("{tag} {id} {display:?}: {error}"));
                assert_eq!(formatter.unit(), id);
                assert!(!formatter.format(2).unwrap().is_empty());
            }
        }
    }
    assert!(UnitFormatter::units().count() > 90);
}

#[test]
fn units_without_names_are_errors() {
    let en = parse_locale("en").unwrap();
    for unknown in ["celsius", "kilometer-per-hour", "byte", "metre", ""] {
        assert!(
            matches!(
                UnitFormatter::new(&en, unknown, UnitOptions::default()),
                Err(IntlError::InvalidOptions(_))
            ),
            "{unknown}"
        );
    }
    // The id is read without surrounding spaces.
    assert!(UnitFormatter::new(&en, " meter ", UnitOptions::default()).is_ok());
}
