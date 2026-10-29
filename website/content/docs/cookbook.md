---
title: Cookbook
description: Recipes for a count with its plural word, money from minor units, a timestamp in the user's zone, today and yesterday labels, per-locale formatter sets, and testing formatted text.
group: Guides
order: 60
---

Self-contained recipes for problems that come up once the basics work. Each one compiles against `dxn_intl` v0.1.0; the one that needs a time zone database uses the `jiff` crate and says so.

## Count with a plural word

"1 item", "1.0 items", "2.5 items". The number on screen decides the form, so format first, read the plural category from what the formatter shows, and pick the word.

```rust
use dxn_intl::{IntlError, Locale, NumberFormatter, NumberOptions, PluralCategory, PluralKind, PluralRules, parse_locale};

fn count_label(
    locale: &Locale,
    forms: &[(PluralCategory, &str)],
    count: f64,
    fraction_digits: u8,
) -> Result<String, IntlError> {
    let options = NumberOptions::default().with_fraction_digits(fraction_digits);
    let numbers = NumberFormatter::new(locale, options)?;
    let plurals = PluralRules::new(locale, PluralKind::Cardinal)?;

    let category = plurals.category(numbers.resolve(count)?); // what the user reads
    let word = forms
        .iter()
        .find(|(c, _)| *c == category)
        .or_else(|| forms.iter().find(|(c, _)| *c == PluralCategory::Other))
        .map_or("", |(_, word)| *word);
    Ok(format!("{} {word}", numbers.format(count)?))
}

use PluralCategory::{Few, One, Other};
let en = parse_locale("en")?;
let en_forms = [(One, "item"), (Other, "items")];
assert_eq!(count_label(&en, &en_forms, 1.0, 0)?, "1 item");
assert_eq!(count_label(&en, &en_forms, 1.0, 1)?, "1.0 items");
assert_eq!(count_label(&en, &en_forms, 2.5, 1)?, "2.5 items");

let hr = parse_locale("hr")?;
let hr_forms = [(One, "datoteka"), (Few, "datoteke"), (Other, "datoteka")];
assert_eq!(count_label(&hr, &hr_forms, 3.0, 0)?, "3 datoteke");
assert_eq!(count_label(&hr, &hr_forms, 2.0, 1)?, "2,0 datoteka"); // 2.0 is not `few`
```

The raw `count` would pick `item` for `1.0`. `resolve` returns the rounded, padded value, and the same value is what `format` prints, so the word and the digits cannot disagree. The `forms` slice is where your translations live; a language that needs `few` or `many` just has more entries. For a count shown next to a currency amount, resolve with a plain decimal formatter that has the currency's digits, because `resolve` does not pad currencies (see [Numbers](./numbers.md#resolve)).

## Format money from minor units

Prices are stored as integers of the smallest unit (cents). Dividing by 100 in floating point reintroduces binary rounding, so build the decimal from the integer's digits.

```rust
use dxn_intl::{IntlError, Number, NumberFormatter, NumberOptions, parse_locale};

/// `minor` units of a currency with `exponent` fraction digits (2 for EUR, 0 for JPY).
fn from_minor_units(minor: i64, exponent: u32) -> Result<Number, IntlError> {
    let scale = 10_u64.pow(exponent);
    let sign = if minor < 0 { "-" } else { "" };
    let (whole, fraction) = (minor.unsigned_abs() / scale, minor.unsigned_abs() % scale);
    let text = if exponent == 0 {
        format!("{sign}{whole}")
    } else {
        format!("{sign}{whole}.{fraction:0width$}", width = exponent as usize)
    };
    text.parse()
}

let eur = NumberFormatter::new(&parse_locale("de-DE")?, NumberOptions::currency("EUR"))?;
assert_eq!(eur.format(from_minor_units(123_456, 2)?)?, "1.234,56\u{a0}€");
assert_eq!(eur.format(from_minor_units(5, 2)?)?, "0,05\u{a0}€");
assert_eq!(eur.format(from_minor_units(-199, 2)?)?, "-1,99\u{a0}€");

let jpy = NumberFormatter::new(&parse_locale("ja-JP")?, NumberOptions::currency("JPY"))?;
assert_eq!(jpy.format(from_minor_units(1500, 0)?)?, "￥1,500");
```

The formatter rounds to the currency's own digits, so a price with extra digits (a tax computed to four places) is rounded half away from zero when shown. The library does not know a currency's exponent; keep it next to the amount. `ja-JP` writes the yen sign full-width, `￥`.

## Format a timestamp for the user's zone

The crate takes an instant and an offset; a zone database turns the zone name into the offset at that instant. This recipe uses [`jiff`](https://docs.rs/jiff) (`jiff = "0.2"`), which reads the operating system's zone database by default; `chrono-tz` or any crate that gives a UTC offset for an instant works the same way.

```rust
use dxn_intl::{DateTime, DateTimeFormatter, DateTimeOptions, Length, parse_locale};
use jiff::{Timestamp, tz::TimeZone};

fn format_instant(millis: i64, zone: &str, locale: &str) -> Result<String, Box<dyn std::error::Error>> {
    let instant = Timestamp::from_millisecond(millis)?;
    let offset = TimeZone::get(zone)?.to_offset(instant); // the zone's offset at that instant
    let wall = DateTime::from_unix_millis(millis, offset.seconds())?;

    let options = DateTimeOptions::date_time(Length::Medium);
    let formatter = DateTimeFormatter::new(&parse_locale(locale)?, options)?;
    Ok(formatter.format(&wall)?)
}

// 2026-07-14T03:33:20Z: Zagreb is on summer time, UTC+2.
assert_eq!(format_instant(1_784_000_000_000, "Europe/Zagreb", "de")?, "14.07.2026, 05:33");
// 2026-01-09T23:06:40Z: winter time, UTC+1, which is already the next day.
assert_eq!(format_instant(1_768_000_000_000, "Europe/Zagreb", "de")?, "10.01.2026, 00:06");
// The same instant for a user in New York (UTC-5 in winter).
assert_eq!(
    format_instant(1_768_000_000_000, "America/New_York", "en-US")?,
    "Jan 9, 2026, 6:06\u{202f}PM"
);
```

Take the "current time" from your platform and pass it in: `SystemTime::now().duration_since(UNIX_EPOCH)` natively, `Date.now()` in the browser. If you already have a civil date and time (a calendar entry the user typed, say), build the `DateTime` directly with `DateTime::new` and skip the offset.

## Today and yesterday

The crate has no relative times, but "Today" and "Yesterday" for recent days are a comparison of day numbers. Convert both instants to the same zone's wall days, and fall back to a formatted date.

```rust
use dxn_intl::{DateTime, DateTimeFormatter, DateTimeOptions, IntlError, Length, parse_locale};

/// Days since 1970-01-01 in the zone with this UTC offset.
fn day_number(millis: i64, offset_seconds: i32) -> i64 {
    (millis + i64::from(offset_seconds) * 1000).div_euclid(86_400_000)
}

fn day_label(
    now: i64,
    then: i64,
    offset_seconds: i32,
    words: (&str, &str), // your translations of "Today" and "Yesterday"
    dates: &DateTimeFormatter,
) -> Result<String, IntlError> {
    match day_number(now, offset_seconds) - day_number(then, offset_seconds) {
        0 => Ok(words.0.to_string()),
        1 => Ok(words.1.to_string()),
        _ => dates.format(&DateTime::from_unix_millis(then, offset_seconds)?),
    }
}

let dates = DateTimeFormatter::new(&parse_locale("en-US")?, DateTimeOptions::date(Length::Medium))?;
let now = 1_793_112_330_000; // 2026-10-27T14:45:30Z
let hour = 3_600_000;
let words = ("Today", "Yesterday");

assert_eq!(day_label(now, now - hour, 3600, words, &dates)?, "Today");
assert_eq!(day_label(now, now - 24 * hour, 3600, words, &dates)?, "Yesterday");
assert_eq!(day_label(now, now - 72 * hour, 3600, words, &dates)?, "Oct 24, 2026");
```

This works in a fixed offset. Across a daylight-saving change, compute each instant's own offset (see the previous recipe) and compare the day numbers of the resulting wall times.

## One set of formatters per locale

A formatter loads the locale's data when it is made and is then cheap to call. Build the set an interface needs once per locale and keep it with the code that uses it. The formatters are not `Send` or `Sync`, so a `static`, an `Arc` or a task that moves between threads cannot hold them; keep the set on one thread, or per thread.

```rust
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use dxn_intl::{
    DateTimeFormatter, DateTimeOptions, IntlError, Length, Locale, NumberFormatter, NumberOptions,
    PluralKind, PluralRules, parse_locale,
};

struct Formatters {
    locale: Locale,
    count: PluralRules,
    price: NumberFormatter,
    date: DateTimeFormatter,
}

impl Formatters {
    fn new(locale: Locale, currency: &str) -> Result<Self, IntlError> {
        Ok(Self {
            count: PluralRules::new(&locale, PluralKind::Cardinal)?,
            price: NumberFormatter::new(&locale, NumberOptions::currency(currency))?,
            date: DateTimeFormatter::new(&locale, DateTimeOptions::date(Length::Long))?,
            locale,
        })
    }
}

thread_local! {
    static CACHE: RefCell<HashMap<String, Rc<Formatters>>> = RefCell::new(HashMap::new());
}

/// The formatters for a tag, built on first use on this thread.
fn formatters(tag: &str) -> Result<Rc<Formatters>, IntlError> {
    CACHE.with(|cache| {
        if let Some(found) = cache.borrow().get(tag) {
            return Ok(Rc::clone(found));
        }
        let built = Rc::new(Formatters::new(parse_locale(tag)?, "EUR")?);
        cache.borrow_mut().insert(tag.to_string(), Rc::clone(&built));
        Ok(built)
    })
}

let de = formatters("de-DE")?;
assert_eq!(de.price.format(9.9)?, "9,90\u{a0}€");
assert!(Rc::ptr_eq(&de, &formatters("de-DE")?)); // the second call reuses the first
assert_eq!(de.locale.to_string(), "de-DE");
```

Building one is fast, measured in microseconds in a release build when these docs were written, so rebuilding on a locale change is fine too. The pattern matters when the same set is used in a tight loop, such as rendering a table.

## Test formatted text

CLDR output contains non-breaking spaces (U+00A0 and U+202F) that look like spaces and compare unequal to them. Either write them as escapes, or normalize in a test helper so expected values stay readable.

```rust
use dxn_intl::{NumberFormatter, NumberOptions, parse_locale};

fn plain(text: &str) -> String {
    text.replace(['\u{a0}', '\u{202f}'], " ")
}

let fr = NumberFormatter::new(&parse_locale("fr-FR")?, NumberOptions::currency("EUR"))?;
let text = fr.format(1234.5)?;

assert_ne!(text, "1 234,50 €");          // ASCII spaces: not equal
assert_eq!(plain(&text), "1 234,50 €");  // normalized
assert_eq!(text, "1\u{202f}234,50\u{a0}€"); // exact
```

Expect CLDR to change between ICU4X releases: separators, short month names and joiners are revised. Assert exact text for strings your product depends on (a receipt format), and check structure, such as "contains the year" or "starts with the symbol", for the rest. The library's own tests pin exact strings and are the best source of what v0.1.0 prints for a locale.

## Set the page direction

For a web page or an app that lays out right to left, ask the locale and write the attribute.

```rust
use dxn_intl::{direction, parse_locale};

let locale = parse_locale("ar-EG")?;
let html = format!(r#"<html lang="{locale}" dir="{}">"#, direction(&locale).as_str());
assert_eq!(html, r#"<html lang="ar-EG" dir="rtl">"#);
```

## Next

[Errors](./errors.md) lists what each call can return. [Limitations](./limitations.md) covers what is not here, such as relative times and time zone names.
