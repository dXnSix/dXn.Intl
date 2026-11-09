//! Times relative to now in one unit: numbers or the locale's words for nearby times, in the
//! past and the future.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dxn_intl::{
    IntlError, Number, RelativeTimeFormatter, RelativeTimeNumeric, RelativeTimeOptions,
    RelativeTimeStyle, RelativeTimeUnit, parse_locale,
};

fn relative(
    tag: &str,
    unit: RelativeTimeUnit,
    options: RelativeTimeOptions,
    value: impl TryInto<Number, Error: Into<IntlError>>,
) -> String {
    RelativeTimeFormatter::new(&parse_locale(tag).unwrap(), unit, options)
        .unwrap()
        .format(value)
        .unwrap()
}

#[test]
fn numbers_in_every_unit() {
    let always = RelativeTimeOptions::default();
    use RelativeTimeUnit::*;
    let cases = [
        (Second, 5, "in 5 seconds"),
        (Minute, -5, "5 minutes ago"),
        (Hour, 2, "in 2 hours"),
        (Day, 1, "in 1 day"),
        (Day, -1, "1 day ago"),
        (Week, 3, "in 3 weeks"),
        (Month, -2, "2 months ago"),
        (Quarter, 1, "in 1 quarter"),
        (Year, -10, "10 years ago"),
    ];
    for (unit, value, text) in cases {
        assert_eq!(
            relative("en", unit, always, value),
            text,
            "{unit:?} {value}"
        );
    }
    assert_eq!(relative("en", Day, always, 1200), "in 1,200 days");
    assert_eq!(relative("en", Hour, always, 1.5), "in 1.5 hours");
}

#[test]
fn words_for_nearby_times() {
    use RelativeTimeUnit::*;
    let auto = RelativeTimeOptions::auto(RelativeTimeStyle::Long);
    assert_eq!(relative("en", Day, auto, -1), "yesterday");
    assert_eq!(relative("en", Day, auto, 0), "today");
    assert_eq!(relative("en", Day, auto, 1), "tomorrow");
    assert_eq!(relative("en", Day, auto, 2), "in 2 days");
    assert_eq!(relative("en", Week, auto, -1), "last week");
    assert_eq!(relative("en", Year, auto, 1), "next year");
    assert_eq!(relative("en", Second, auto, 0), "now");
    assert_eq!(relative("de", Day, auto, -2), "vorgestern");
    assert_eq!(relative("hr", Day, auto, 1), "sutra");
    assert_eq!(
        RelativeTimeOptions::auto(RelativeTimeStyle::Short).numeric,
        RelativeTimeNumeric::Auto
    );
}

#[test]
fn shorter_styles_and_other_locales() {
    use RelativeTimeUnit::*;
    let short = RelativeTimeOptions {
        style: RelativeTimeStyle::Short,
        ..RelativeTimeOptions::default()
    };
    let narrow = RelativeTimeOptions {
        style: RelativeTimeStyle::Narrow,
        ..RelativeTimeOptions::default()
    };
    assert_eq!(relative("en", Month, short, 3), "in 3 mo.");
    assert_eq!(relative("en", Minute, short, -5), "5 min. ago");
    assert!(!relative("en", Minute, narrow, -5).is_empty());
    // The locale's plural forms.
    let always = RelativeTimeOptions::default();
    assert_eq!(relative("hr", Day, always, 1), "za 1 dan");
    assert_eq!(relative("hr", Day, always, 3), "za 3 dana");
    assert_eq!(relative("hr", Day, always, 5), "za 5 dana");
    assert_eq!(relative("hr", Year, always, -5), "prije 5 godina");
    assert_eq!(relative("de", Hour, always, -3), "vor 3 Stunden");
    assert_eq!(relative("ja", Day, always, 3), "3 日後");
}

#[test]
fn not_a_number() {
    let formatter = RelativeTimeFormatter::new(
        &parse_locale("en").unwrap(),
        RelativeTimeUnit::Day,
        RelativeTimeOptions::default(),
    )
    .unwrap();
    assert!(matches!(
        formatter.format(f64::NAN),
        Err(IntlError::InvalidNumber(_))
    ));
}
