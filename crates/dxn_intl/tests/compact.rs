//! Compact notation: the magnitude as an abbreviation or a word, in the locale's words and
//! digits.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dxn_intl::{CompactDisplay, CompactNumberFormatter, IntlError, Number, parse_locale};

fn compact(
    tag: &str,
    display: CompactDisplay,
    value: impl TryInto<Number, Error: Into<IntlError>>,
) -> String {
    CompactNumberFormatter::new(&parse_locale(tag).unwrap(), display)
        .unwrap()
        .format(value)
        .unwrap()
}

#[test]
fn short_compact_numbers() {
    assert_eq!(compact("en", CompactDisplay::Short, 999), "999");
    assert_eq!(compact("en", CompactDisplay::Short, 1234), "1.2K");
    assert_eq!(compact("en", CompactDisplay::Short, 15_300), "15K");
    assert_eq!(compact("en", CompactDisplay::Short, 2_500_000), "2.5M");
    assert_eq!(
        compact("en", CompactDisplay::Short, 7_000_000_000_u64),
        "7B"
    );
    assert_eq!(compact("en", CompactDisplay::Short, -1234), "-1.2K");
    assert_eq!(compact("en", CompactDisplay::Short, 0), "0");
    // Exact decimals and floats are numbers too.
    assert_eq!(
        compact(
            "en",
            CompactDisplay::Short,
            "1250000".parse::<Number>().unwrap()
        ),
        "1.3M"
    );
    assert_eq!(compact("en", CompactDisplay::Short, 1234.5), "1.2K");
}

#[test]
fn compact_numbers_round_half_away_from_zero() {
    assert_eq!(compact("en", CompactDisplay::Short, 1_250_000), "1.3M");
    assert_eq!(compact("en", CompactDisplay::Short, -1_250_000), "-1.3M");
    assert_eq!(compact("en", CompactDisplay::Short, 1_150), "1.2K");
    assert_eq!(compact("en", CompactDisplay::Short, 15_500), "16K");
    // Rounding up reaches the next magnitude.
    assert_eq!(compact("en", CompactDisplay::Short, 999_950), "1M");
    assert_eq!(compact("en", CompactDisplay::Short, 99_950), "100K");
    assert_eq!(compact("en", CompactDisplay::Short, 999.5), "1K");
    // Below the first magnitude: two significant digits.
    assert_eq!(compact("en", CompactDisplay::Short, 12.345), "12");
    assert_eq!(compact("en", CompactDisplay::Short, 1.25), "1.3");
    assert_eq!(compact("en", CompactDisplay::Short, 0.0123), "0.012");
}

#[test]
fn long_compact_numbers() {
    assert_eq!(compact("en", CompactDisplay::Long, 1234), "1.2 thousand");
    assert_eq!(
        compact("en", CompactDisplay::Long, 3_400_000),
        "3.4 million"
    );
    assert_eq!(compact("de", CompactDisplay::Long, 1_000_000), "1 Million");
    assert_eq!(
        compact("de", CompactDisplay::Long, 3_400_000),
        "3,4 Millionen"
    );
}

#[test]
fn compact_numbers_follow_the_locale() {
    // German abbreviates from a million; a thousand is written out.
    assert_eq!(compact("de", CompactDisplay::Short, 1234), "1234");
    assert_eq!(
        compact("de", CompactDisplay::Short, 2_500_000),
        "2,5\u{a0}Mio."
    );
    // Japanese groups by ten thousands.
    assert_eq!(compact("ja", CompactDisplay::Short, 12_345), "1.2万");
    // Hindi has lakh and crore.
    assert_eq!(compact("en-IN", CompactDisplay::Short, 1_500_000), "15L");
    // A numbering system from the locale.
    assert_eq!(
        compact("ar-u-nu-arab", CompactDisplay::Short, 1234),
        "١٫٢\u{a0}ألف"
    );
}

#[test]
fn not_a_number() {
    let formatter =
        CompactNumberFormatter::new(&parse_locale("en").unwrap(), CompactDisplay::Short).unwrap();
    assert!(matches!(
        formatter.format(f64::NAN),
        Err(IntlError::InvalidNumber(_))
    ));
    assert!(matches!(
        formatter.format(f64::INFINITY),
        Err(IntlError::InvalidNumber(_))
    ));
}
