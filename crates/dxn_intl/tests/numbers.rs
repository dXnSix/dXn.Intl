//! Decimal, percent and currency formatting, and the numbers formatters take.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dxn_intl::{
    Grouping, IntlError, Number, NumberFormatter, NumberOptions, SignDisplay, parse_locale,
};

fn format(
    tag: &str,
    options: NumberOptions,
    value: impl TryInto<Number, Error: Into<IntlError>>,
) -> String {
    NumberFormatter::new(&parse_locale(tag).unwrap(), options)
        .unwrap()
        .format(value)
        .unwrap()
}

#[test]
fn decimals_use_the_locales_separators() {
    assert_eq!(
        format("en-US", NumberOptions::default(), 1234567.891),
        "1,234,567.891"
    );
    assert_eq!(
        format("de-DE", NumberOptions::default(), 1234567.891),
        "1.234.567,891"
    );
    assert_eq!(
        format("fr-FR", NumberOptions::default(), 1234.5),
        "1\u{202f}234,5"
    );
    assert_eq!(
        format("en-IN", NumberOptions::default(), 1234567),
        "12,34,567"
    );
    assert_eq!(format("ar-EG", NumberOptions::default(), 12), "١٢");
    assert_eq!(
        format("ar-EG-u-nu-latn", NumberOptions::default(), 12),
        "12"
    );
}

#[test]
fn fraction_digits_round_half_away_from_zero_and_pad() {
    let two = NumberOptions::default().with_fraction_digits(2);
    assert_eq!(format("en", two.clone(), 2.005_f64), "2.01");
    assert_eq!(format("en", two.clone(), 2.5), "2.50");
    assert_eq!(format("en", two, -1.005_f64), "-1.01");
    // Three fraction digits at most by default.
    assert_eq!(format("en", NumberOptions::default(), 1.23456), "1.235");
    let whole = NumberOptions {
        maximum_fraction_digits: Some(0),
        ..NumberOptions::default()
    };
    assert_eq!(format("en", whole, 2.5), "3");
    let integer_digits = NumberOptions {
        minimum_integer_digits: 3,
        ..NumberOptions::default()
    };
    assert_eq!(format("en", integer_digits, 7), "007");
}

#[test]
fn grouping_and_sign() {
    let never = NumberOptions {
        grouping: Grouping::Never,
        ..NumberOptions::default()
    };
    assert_eq!(format("en", never, 1234567), "1234567");
    let min2 = NumberOptions {
        grouping: Grouping::Min2,
        ..NumberOptions::default()
    };
    assert_eq!(format("en", min2.clone(), 1234), "1234");
    assert_eq!(format("en", min2, 12345), "12,345");
    let always = NumberOptions {
        sign_display: SignDisplay::Always,
        ..NumberOptions::default()
    };
    assert_eq!(format("en", always, 5), "+5");
    let except_zero = NumberOptions {
        sign_display: SignDisplay::ExceptZero,
        ..NumberOptions::default()
    };
    assert_eq!(format("en", except_zero, 0), "0");
}

#[test]
fn percentages_scale_by_a_hundred() {
    assert_eq!(format("en", NumberOptions::percent(), 0.25), "25%");
    assert_eq!(format("de", NumberOptions::percent(), 0.25), "25\u{a0}%");
    assert_eq!(format("en", NumberOptions::percent(), 0.1234), "12%");
    assert_eq!(
        format(
            "en",
            NumberOptions::percent().with_fraction_digits(1),
            0.1234
        ),
        "12.3%"
    );
}

#[test]
fn currencies_show_their_symbol_and_digits() {
    assert_eq!(
        format("en-US", NumberOptions::currency("USD"), 1234.5),
        "$1,234.50"
    );
    assert_eq!(
        format("de-DE", NumberOptions::currency("EUR"), 1234.5),
        "1.234,50\u{a0}€"
    );
    assert_eq!(
        format("ja-JP", NumberOptions::currency("JPY"), 1234.5),
        "￥1,235"
    );
    assert!(matches!(
        NumberFormatter::new(
            &parse_locale("en").unwrap(),
            NumberOptions::currency("EURO")
        ),
        Err(IntlError::InvalidCurrency(_))
    ));
}

#[test]
fn resolve_gives_the_value_that_is_shown() {
    let formatter = NumberFormatter::new(
        &parse_locale("en").unwrap(),
        NumberOptions::default().with_fraction_digits(1),
    )
    .unwrap();
    assert_eq!(formatter.resolve(1).unwrap().to_string(), "1.0");
    let percent =
        NumberFormatter::new(&parse_locale("en").unwrap(), NumberOptions::percent()).unwrap();
    assert_eq!(percent.resolve(0.01).unwrap().to_string(), "1");
}

#[test]
fn numbers_keep_their_digits() {
    let n: Number = "1.50".parse().unwrap();
    assert_eq!(n.to_string(), "1.50");
    assert!(!n.is_integer());
    assert!(Number::from(3).is_integer());
    assert_eq!(Number::try_from(0.1).unwrap().to_string(), "0.1");
    assert_eq!(Number::try_from(0.1_f32).unwrap().to_string(), "0.1");
    assert_eq!(Number::try_from(-2.5).unwrap().to_f64(), -2.5);
    assert!(Number::try_from(f64::NAN).is_err());
    assert!("1,5".parse::<Number>().is_err());
}

#[test]
fn bad_options_are_errors() {
    let options = NumberOptions {
        minimum_fraction_digits: Some(3),
        maximum_fraction_digits: Some(1),
        ..NumberOptions::default()
    };
    assert!(matches!(
        NumberFormatter::new(&parse_locale("en").unwrap(), options),
        Err(IntlError::InvalidOptions(_))
    ));
}
