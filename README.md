# dXn.Intl

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

**Documentation:** https://dxnsix.github.io/dXn.Intl/

**Locale-aware plurals, numbers and dates for Rust**, on [ICU4X](https://icu4x.unicode.org)'s CLDR
data, independent of any UI framework and building for `wasm32-unknown-unknown`. The formatting
happens in Rust, so an app shows the same text on every platform, browser or not.

- **Plurals**: a number's cardinal or ordinal category (`zero`, `one`, `two`, `few`, `many`,
  `other`) for every language CLDR has, reading the visible fraction digits (`1.0` is not `1`).
- **Numbers**: decimal numbers, percentages and amounts of money in the locale's digits,
  separators and patterns, with fraction digits, grouping and sign options.
- **Dates and times**: the fields and length you ask for, in the locale's calendar, words, order
  and clock, or a clock you choose.
- **Locales**: reading BCP 47 and POSIX tags, picking the best available locale for a user's
  preferred list, and a locale's writing direction.

## Use

```toml
[dependencies]
dxn_intl = { git = "https://github.com/dXnSix/dXn.Intl", tag = "v0.1.0" }
```

```rust
use dxn_intl::{
    DateTime, DateTimeFormatter, DateTimeOptions, Length, NumberFormatter, NumberOptions,
    PluralCategory, PluralKind, PluralRules, negotiate, parse_locale,
};

// The best of the app's locales for the user's list.
let available = [parse_locale("en")?, parse_locale("de")?, parse_locale("hr")?];
let locale = negotiate(&[parse_locale("de-AT")?], &available, &available[0]);
assert_eq!(locale.to_string(), "de");

let plurals = PluralRules::new(&parse_locale("hr")?, PluralKind::Cardinal)?;
assert_eq!(plurals.category(23), PluralCategory::Few);

let money = NumberFormatter::new(&locale, NumberOptions::currency("EUR"))?;
assert_eq!(money.format(1234.5)?, "1.234,50\u{a0}€");

let dates = DateTimeFormatter::new(&locale, DateTimeOptions::date(Length::Long))?;
assert_eq!(dates.format(&DateTime::date(2026, 10, 27)?)?, "27. Oktober 2026");
# Ok::<(), dxn_intl::IntlError>(())
```

Numbers are exact decimals (`Number`): made from integers, from text (`"1.50"` keeps its digits) or
from finite floats (the shortest text that reads back as the same float). `NumberFormatter::resolve`
gives the value as it will be shown (scaled, rounded, padded), which is the value plural selection
should read: "1.0 items" takes the form for `1.0`.

Dates are wall times on the ISO calendar (`DateTime`); there is no time zone database here. Take an
instant's local time in the user's zone first, or use `DateTime::from_unix_millis` with the zone's
offset.

## What is left out

Time zone names, relative times ("in 3 days"), lists, units, compact numbers and collation. ICU4X
has them; they come here when a consumer needs them.

## Development

See [AGENTS.md](AGENTS.md) for validation and [docs/releasing.md](docs/releasing.md) for releases.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT)
at your option. ICU4X and its CLDR data are under the [Unicode License v3](https://www.unicode.org/license.txt).
