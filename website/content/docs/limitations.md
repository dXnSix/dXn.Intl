---
title: Limitations
description: What dxn_intl does not format, options that are ignored or limited in v0.1.0, behavior that surprises, and what to do instead.
group: Reference
order: 80
---

What the crate leaves out, where it behaves differently from what you might expect, and what to do in each case. The version is v0.1.0; check the [releases](https://github.com/dXnSix/dXn.Intl/releases) for changes.

## Not implemented

ICU4X has most of these; they are added when a consumer needs them.

| Missing | Instead |
|---|---|
| Time zones: names ("CET"), offsets, conversion, daylight saving | Resolve the offset with a zone database and pass it to `DateTime::from_unix_millis`; see [Dates and times](./dates-and-times.md#time-zones) |
| Relative times ("in 3 days", "yesterday") | Compare day numbers and use your own words; see [Cookbook](./cookbook.md#today-and-yesterday) |
| Lists ("A, B and C") | Join with your translations, or use messages in [dXn.I18n](https://dxnsix.github.io/dXn.I18n/) |
| Units ("5 km", "3 hours") | Format the number, add the unit from your translations |
| Compact numbers ("1.2K"), scientific notation | Format the number and divide it yourself |
| Collation (locale-aware sorting) | Not available |
| Parsing localized input ("1.234,5", "27.10.2026") | `Number` reads ASCII only; normalize the input first |
| Date patterns and skeletons of your own, format to parts | Pick `DateFields`, `TimePrecision` and `Length` |
| Date ranges ("Oct 27 – 29") | Format both ends and join |
| Currency display styles (code, name, narrow symbol), accounting negatives | The locale's symbol is always used |
| Plural currency names ("5 US dollars") | Use plural categories with your own words |
| Rounding modes other than half away from zero; significant digits | Round the `Number` yourself before formatting |
| Choosing which locales' data ships | The data of every locale is compiled in; see [Platforms](./platforms.md#data-size) |

Messages with plural and gender selection, placeholders and a bundle per language belong one level up: [dXn.I18n](https://dxnsix.github.io/dXn.I18n/) builds on this crate for them.

## Behavior with caveats

### Formatters are not Send or Sync

`PluralRules`, `NumberFormatter` and `DateTimeFormatter` cannot cross threads: they contain `Rc` values from ICU4X's data handling. They cannot sit in a `static`, an `Arc` or state that a multi-threaded runtime moves between threads. Keep them on the thread that uses them or build one per thread; see [Cookbook](./cookbook.md#one-set-of-formatters-per-locale). `Number`, `DateTime`, `Locale` and `IntlError` are `Send + Sync`.

### Floats lose trailing zeros

`Number::try_from(1.0_f64)` is `1`, so the plural rules read it as `1`, not `1.0`. Use text or the formatter's `resolve`. See [Numbers](./numbers.md#number).

### Options that do not apply to currencies

A currency formatter uses the currency's own fraction digits and ignores `minimum_fraction_digits`, `maximum_fraction_digits` and `with_fraction_digits`. In v0.1.0 it also ignores `grouping`: `Grouping::Never` still shows `$12,345.00`. `NumberFormatter::resolve` does not round or pad a currency amount, so it is not the value shown. See [Numbers](./numbers.md#currency).

### Grouping::Always is Auto

ICU4X's decimal formatter treats `Always` like `Auto`, so it does not group four-digit numbers in locales that leave them ungrouped (Spanish, Polish, Portuguese in Portugal). The crate's own documentation says otherwise.

### Currency symbols fall back to the code

When the locale's data has no symbol for a currency, the ISO code is printed (`BHD 1,234.500`). ICU4X writes a `Missing data for identifier` line to standard error when this happens. A syntactically valid but unknown code, such as `ZZZ`, is not an error.

### POSIX modifiers are dropped

`sr_RS@latin` reads as `sr-RS` (Cyrillic), not `sr-Latn-RS`, and `C` and `POSIX` are errors. See [Locales](./locales.md#posix-tags).

### Negotiation drops the region

`negotiate` returns one of the locales you passed in `available`, with the user's Unicode extensions added. A user's `de-CH` becomes your `de`, and a formatter built from that answer uses German rather than Swiss conventions. Format with the user's own locale when it exists. See [Locales](./locales.md#the-users-preferences-survive).

### Calendars follow the locale

The input is always the ISO calendar, and the output follows the locale: `th` prints Buddhist years (2569) and `fa` Persian ones (1405). Add `-u-ca-gregory` to the locale when you need Gregorian dates. See [Dates and times](./dates-and-times.md#calendars).

### Dates have a year range and no zone

`DateTime` accepts the years -9999 to 9999. `from_unix_millis` can build a value outside that range and the failure comes at `format`. The documented range for `second` is 0 to 59, but 60 is accepted.

### Exact strings change with the data

The text comes from CLDR through ICU4X, which revises separators, abbreviations and joiners between releases (the narrow no-break space in `en` times is one such change). Dependencies are `icu` 2.4 or newer within 2.x and `icu_experimental` pinned to exactly 0.7.0, which provides percent and currency formatting and changes its API in any release. Pin the crate's tag, and rerun your tests when you bump it.

### It is a 0.x library

Under the 0.x policy a breaking change is released as a minor version (`0.1.x` to `0.2.0`) with a `Migration:` note. A fix, a performance change or a feature is a patch. See [Releases](./releasing.md).

## Not covered by tests

The test suite in `crates/dxn_intl/tests/` checks plurals, numbers, dates and locales for a handful of locales each, not every language. It runs on Linux, macOS and Windows in CI, and CI checks that the crate builds for `wasm32-unknown-unknown` but does not run the tests there. The plural rules, calendars and patterns come from ICU4X and CLDR, and their own test suites cover them; this crate's tests cover how it maps its options onto them.

## Next

[Platforms](./platforms.md) covers targets, threads and data size. [Cookbook](./cookbook.md) has recipes for several of the gaps above.
