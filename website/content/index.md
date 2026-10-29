---
title: dXn.Intl
description: Locale-aware plurals, numbers, dates and locale negotiation for Rust on ICU4X's CLDR data, with no UI framework and a build for wasm32-unknown-unknown.
eyebrow: Locale-aware formatting for Rust
headline: Plurals, numbers and dates, the same everywhere.
lede: >-
  dxn_intl formats the things an app shows in the user's language: plural forms for every language
  CLDR has, decimal, percent and currency numbers, and dates and times in the locale's calendar and
  clock. It runs on ICU4X's compiled-in data and reads nothing from the system, so a native app, a
  server and a WebAssembly page print the same text.
install: |-
  [dependencies]
  dxn_intl = { git = "https://github.com/dXnSix/dXn.Intl", tag = "v0.1.0" }
facts:
  - { label: Data, value: "ICU4X 2.4, CLDR compiled in" }
  - { label: Rust, value: "1.97+ (edition 2024)" }
  - { label: Targets, value: "native + wasm32-unknown-unknown" }
  - { label: System access, value: "none: no files, clock or env" }
  - { label: Unsafe code, value: "forbidden" }
  - { label: License, value: "MIT OR Apache-2.0" }
heroTitle: main.rs
heroLang: rust
heroCode: |
  use dxn_intl::{
      DateTime, DateTimeFormatter, DateTimeOptions, Length, NumberFormatter, NumberOptions,
      PluralCategory, PluralKind, PluralRules, negotiate, parse_locale,
  };

  fn main() -> Result<(), dxn_intl::IntlError> {
      // The best of the app's locales for what the user prefers.
      let available = [parse_locale("en")?, parse_locale("de")?, parse_locale("hr")?];
      let locale = negotiate(&[parse_locale("de-AT")?], &available, &available[0]);
      assert_eq!(locale.to_string(), "de");

      let plurals = PluralRules::new(&parse_locale("hr")?, PluralKind::Cardinal)?;
      assert_eq!(plurals.category(23), PluralCategory::Few);

      let money = NumberFormatter::new(&locale, NumberOptions::currency("EUR"))?;
      assert_eq!(money.format(1234.5)?, "1.234,50\u{a0}€");

      let dates = DateTimeFormatter::new(&locale, DateTimeOptions::date(Length::Long))?;
      assert_eq!(dates.format(&DateTime::date(2026, 10, 27)?)?, "27. Oktober 2026");
      Ok(())
  }
features:
  - title: Plural categories for every language
    text: Cardinal and ordinal categories from CLDR, from English's two to Arabic's six. The rules read the visible fraction digits, so 1.0 is not 1.
    link: ./docs/plurals.md
  - title: Numbers as exact decimals
    text: Number keeps "1.50" as written and reads floats as their shortest text, so what the plural rules see is what the user sees.
    link: ./docs/numbers.md
  - title: Decimal, percent and currency
    text: The locale's digits, separators and patterns, with fraction digits, integer digits, grouping and sign options, and the currency's own digits and symbol.
    link: ./docs/numbers.md#currency
  - title: Dates and times in the locale's calendar
    text: Fields and length you choose, shown in the locale's words, order, calendar (Buddhist, Persian, Japanese) and clock, or a 12 or 24 hour clock you pick.
    link: ./docs/dates-and-times.md
  - title: Locales read, negotiated and directed
    text: Parse BCP 47 and POSIX tags (pt_BR.UTF-8), pick the best available locale for a user's list with CLDR fallback, and ask whether it is written left to right.
    link: ./docs/locales.md
  - title: No system access, builds for the browser
    text: The caller passes the locale and the wall time. Nothing reads the OS, so the crate builds for wasm32-unknown-unknown, which CI checks on every push.
    link: ./docs/platforms.md
next:
  - { title: Get started, text: "Add the dependency and format a plural, a price and a date.", link: ./docs/getting-started.md }
  - { title: Plurals, text: "Categories, ordinals and why 1.0 is not 1.", link: ./docs/plurals.md }
  - { title: Cookbook, text: "Localized prices, a timestamp in the user's zone, 1.0 items.", link: ./docs/cookbook.md }
  - { title: Limitations, text: "What is left out and what to do instead.", link: ./docs/limitations.md }
---

## Plural forms that follow the visible digits

A plural category is a property of how the number is shown. In English "1 item" takes `one` and "1.0 items" takes `other`, so `Number` keeps the digits you give it.

```rust
use dxn_intl::{Number, PluralCategory, PluralKind, PluralRules, parse_locale};

let en = PluralRules::new(&parse_locale("en")?, PluralKind::Cardinal)?;
assert_eq!(en.category(1), PluralCategory::One);
assert_eq!(en.category("1.0".parse::<Number>()?), PluralCategory::Other);

let pl = PluralRules::new(&parse_locale("pl")?, PluralKind::Cardinal)?;
assert_eq!(pl.category(3), PluralCategory::Few);
assert_eq!(pl.category(5), PluralCategory::Many);
```

[Plurals](./docs/plurals.md) covers ordinals, the categories each language uses and how to pick a word from a category.

## Money in the user's locale

The same amount reads differently per locale: symbol position, separators, spacing and the currency's own number of digits.

```rust
use dxn_intl::{NumberFormatter, NumberOptions, parse_locale};

let de = NumberFormatter::new(&parse_locale("de-DE")?, NumberOptions::currency("EUR"))?;
assert_eq!(de.format(1234.5)?, "1.234,50\u{a0}€"); // a non-breaking space before €

let us = NumberFormatter::new(&parse_locale("en-US")?, NumberOptions::currency("USD"))?;
assert_eq!(us.format(1234.5)?, "$1,234.50");
```

[Numbers](./docs/numbers.md) covers percent, fraction digits, grouping, sign display and `resolve`.

## A timestamp in the user's zone

Dates are wall times on the ISO calendar. There is no time zone database in the crate: you give the instant and the zone's offset at that instant, and it shows the locale's words and clock.

```rust
use dxn_intl::{DateTime, DateTimeFormatter, DateTimeOptions, Length, parse_locale};

let millis = 1_793_112_330_000; // 2026-10-27T14:45:30Z
let wall = DateTime::from_unix_millis(millis, 3600)?; // UTC+1
let options = DateTimeOptions::date_time(Length::Medium);

let de = DateTimeFormatter::new(&parse_locale("de")?, options)?;
assert_eq!(de.format(&wall)?, "27.10.2026, 15:45");
let en = DateTimeFormatter::new(&parse_locale("en-US")?, options)?;
assert_eq!(en.format(&wall)?, "Oct 27, 2026, 3:45\u{202f}PM");
```

[Dates and times](./docs/dates-and-times.md) explains the options, hour cycles and how to get the offset.

## Pick the locale the user asked for

`negotiate` walks the user's list in order, trying each entry's CLDR fallback chain (`de-AT` then `de`) before the next entry, and keeps the user's Unicode extensions such as `-u-hc-h23`.

```rust
use dxn_intl::{direction, negotiate, parse_locale};

let available = [parse_locale("en")?, parse_locale("de")?, parse_locale("ar")?];
let requested = [parse_locale("de-CH-u-hc-h12")?, parse_locale("fr")?];
let locale = negotiate(&requested, &available, &available[0]);
assert_eq!(locale.to_string(), "de-u-hc-h12");

assert_eq!(direction(&parse_locale("ar")?).as_str(), "rtl");
```

[Locales](./docs/locales.md) covers POSIX tags, script matching and writing direction.
