---
title: Locales
description: Read BCP 47 and POSIX language tags, pick the best available locale for a user's preferences with negotiate, and ask a locale for its writing direction.
group: Start
order: 20
---

A `Locale` says which language, script and region to format for, plus the user's Unicode preferences (clock, calendar, digits). Every formatter takes one. You get it from a tag with `parse_locale`, and from a list of the user's preferences with `negotiate`.

## Reading a tag

```rust
use dxn_intl::parse_locale;

assert_eq!(parse_locale("en-US")?.to_string(), "en-US");
assert_eq!(parse_locale("sr-Latn-RS")?.to_string(), "sr-Latn-RS");
assert_eq!(parse_locale("de-CH-u-nu-latn")?.to_string(), "de-CH-u-nu-latn");
assert!(parse_locale("not a tag").is_err());
```

`parse_locale` reads BCP 47 tags: language, optional script, region and variants, then Unicode extensions after `-u-`. Case is normalized (`EN-us` reads as `en-US`) and surrounding whitespace is trimmed.

`Locale` is `icu::locale::Locale`, re-exported from the crate root. Its fields are public, so `locale.id.language` and `locale.id.region` give the parts, and `to_string()` gives the canonical tag. The [ICU4X documentation](https://docs.rs/icu_locale_core/latest/icu_locale_core/struct.Locale.html) lists everything else.

### POSIX tags

Platforms report locales as `en_US.UTF-8` or `sr_RS@latin`. `parse_locale` accepts them: underscores become hyphens, and everything from the first `.` or `@` on is dropped.

| Input | Result |
|---|---|
| `en_US` | `en-US` |
| `pt_BR.UTF-8` | `pt-BR` |
| `zh_Hant_TW.UTF-8` | `zh-Hant-TW` |
| `en_US.UTF-8@euro` | `en-US` |
| `sr_RS@latin` | `sr-RS` |
| `C`, `POSIX`, empty text | error (`InvalidLocale`) |

:::warning A POSIX modifier is dropped, not translated
`sr_RS@latin` means Serbian in the Latin script, but the `@latin` modifier is discarded, so the result is `sr-RS`, which CLDR writes in Cyrillic. If your platform can report such modifiers, map them yourself (`sr-Latn-RS`). The `C` and `POSIX` locales are not language tags; treat the error as "no preference" and use your default.
:::

### Unicode extensions

Extensions after `-u-` carry the user's preferences, and the formatters honor them:

| Extension | Effect | Page |
|---|---|---|
| `-u-nu-latn` | Numbering system: `ar-EG` shows `١٢`, `ar-EG-u-nu-latn` shows `12` | [Numbers](./numbers.md) |
| `-u-hc-h12`, `-u-hc-h23` | Clock for times of day | [Dates and times](./dates-and-times.md#hour-cycles) |
| `-u-ca-gregory` | Calendar: `fa` is Persian by default, `fa-u-ca-gregory` is Gregorian | [Dates and times](./dates-and-times.md#calendars) |

## Choosing a locale

`negotiate` takes the user's preferred locales in order, the locales your app has translations or support for, and a default. It returns the best match from `available`, or the default.

```rust
use dxn_intl::{negotiate, parse_locale};

let available = [parse_locale("en")?, parse_locale("de")?, parse_locale("hr")?];
let default = &available[0];

let pick = |tags: &[&str]| -> Result<String, dxn_intl::IntlError> {
    let requested = tags.iter().map(|t| parse_locale(t)).collect::<Result<Vec<_>, _>>()?;
    Ok(negotiate(&requested, &available, default).to_string())
};

assert_eq!(pick(&["de-AT"])?, "de");        // de-AT falls back to de
assert_eq!(pick(&["fr", "hr-BA"])?, "hr");  // fr has nothing; the next entry does
assert_eq!(pick(&["ja"])?, "en");           // no match: the default
assert_eq!(pick(&[])?, "en");
```

Each requested locale is tried before the next one, along CLDR's fallback chain: `de-AT` then `de`, `zh-Hant-HK` then `zh-Hant`. An available locale matches a step when its language and script are the same and, if the step still names a region, its region too.

### Regions

At a step without a region, an available locale without a region is preferred; otherwise any region of that language matches. So a user asking for `fr-FR` gets `fr-CA` over your default when `fr-CA` is all you have:

```rust
use dxn_intl::{negotiate, parse_locale};

let available = [parse_locale("en")?, parse_locale("de")?, parse_locale("fr-CA")?];
let best = negotiate(&[parse_locale("fr-FR")?], &available, &available[0]);
assert_eq!(best.to_string(), "fr-CA");

let regional = [parse_locale("pt-PT")?, parse_locale("pt")?];
let best = negotiate(&[parse_locale("pt-BR")?], &regional, &available[0]);
assert_eq!(best.to_string(), "pt"); // the region-less locale wins over pt-PT
```

### Scripts must match

Serbian is written in Cyrillic and Latin, Chinese in Simplified and Traditional. A script mismatch is not a match, because the user could not read it:

| Requested | Available | Result |
|---|---|---|
| `sr-Latn` | `sr`, `en` | `en` (`sr` is Cyrillic) |
| `sr-Cyrl-RS` | `sr`, `en` | `sr` |
| `zh-TW` | `zh-Hant`, `en` | `zh-Hant` (Taiwan is Traditional) |
| `zh-CN` | `zh-Hant`, `en` | `en` |

When a tag names no script, its language's likely script is used.

### The user's preferences survive

The Unicode extensions of the requested locale are copied onto the answer, and the rest of the requested tag (such as its region) is not:

```rust
use dxn_intl::{negotiate, parse_locale};

let available = [parse_locale("en")?, parse_locale("de")?];
let best = negotiate(&[parse_locale("de-CH-u-hc-h12")?], &available, &available[0]);
assert_eq!(best.to_string(), "de-u-hc-h12");
```

The formatters then show a 12-hour clock even though German defaults to 24 hours. The answer has lost the `CH` region, so a formatter built from it uses German, not Swiss German, conventions; if you need the user's region for numbers or currency, format with their parsed locale and use `negotiate` only for choosing translations.

:::tip Negotiate for text, format for the region
Which translation to show is a choice among what you have, which is what `negotiate` answers. How to write `1234.5` depends on the user's own locale, so format with `parse_locale(user_tag)` directly when you have it, and keep the negotiated locale for looking up strings.
:::

## Accept-Language and the system locale

The crate has no parser for an `Accept-Language` header and reads no environment, because it builds for the browser. Both are a few lines on your side:

```rust
use dxn_intl::{Locale, parse_locale};

/// `fr-CH, fr;q=0.9, en;q=0.8, *;q=0.5` -> [fr-CH, fr, en], in the order sent.
fn accept_language(header: &str) -> Vec<Locale> {
    let mut entries: Vec<(f32, Locale)> = header
        .split(',')
        .filter_map(|part| {
            let mut pieces = part.split(';');
            let locale = parse_locale(pieces.next()?).ok()?; // `*` fails and is skipped
            let q = pieces
                .find_map(|p| p.trim().strip_prefix("q=")?.parse::<f32>().ok())
                .unwrap_or(1.0);
            (q > 0.0).then_some((q, locale))
        })
        .collect();
    entries.sort_by(|a, b| b.0.total_cmp(&a.0)); // stable: equal weights keep their order
    entries.into_iter().map(|(_, locale)| locale).collect()
}

/// The locale from `LC_ALL` or `LANG`, or `default` for `C`, `POSIX` and unset.
fn system_locale(default: &Locale) -> Locale {
    ["LC_ALL", "LANG"]
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .find_map(|value| parse_locale(&value).ok())
        .unwrap_or_else(|| default.clone())
}

let requested = accept_language("fr-CH, fr;q=0.9, en;q=0.8, *;q=0.5");
let tags: Vec<String> = requested.iter().map(|l| l.to_string()).collect();
assert_eq!(tags, ["fr-CH", "fr", "en"]);
```

`system_locale` reads the environment, so it belongs in native code; in the browser, read `navigator.languages` and pass the strings to `parse_locale`.

## Writing direction

`direction` answers whether a locale's script runs left to right or right to left, using the tag's script or the language's likely one.

```rust
use dxn_intl::{Direction, direction, parse_locale};

assert_eq!(direction(&parse_locale("ar")?), Direction::Rtl);
assert_eq!(direction(&parse_locale("he-IL")?), Direction::Rtl);
assert_eq!(direction(&parse_locale("az-Arab")?), Direction::Rtl); // Azerbaijani in Arabic script
assert_eq!(direction(&parse_locale("en")?), Direction::Ltr);

assert_eq!(Direction::Rtl.as_str(), "rtl"); // HTML's dir and CSS's direction
```

An unknown language is left to right. `Direction` is `Copy` and implements `Debug`, `Eq` and `Hash`, but not `Display`; use `as_str()` for text.

## Next

[Plurals](./plurals.md) turns a number into a category once you have a locale. [Cookbook](./cookbook.md) wires these pieces into a per-locale set of formatters.
