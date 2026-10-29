---
title: Plurals
description: Cardinal and ordinal plural categories for every language CLDR has, how the visible fraction digits decide the category, and how to pick a word from one.
group: Formatting
order: 30
---

A plural category tells you which form of a word goes with a number: "1 file", "2 files" in English; "1 datoteka", "2 datoteke", "5 datoteka" in Croatian. `PluralRules` gives the category for a locale, and your text or translation table maps it to a word. You reach for it whenever a sentence contains a count.

```rust
use dxn_intl::{PluralCategory, PluralKind, PluralRules, parse_locale};

let hr = PluralRules::new(&parse_locale("hr")?, PluralKind::Cardinal)?;

assert_eq!(hr.category(1), PluralCategory::One);
assert_eq!(hr.category(2), PluralCategory::Few);
assert_eq!(hr.category(5), PluralCategory::Other);
assert_eq!(hr.category(11), PluralCategory::Other);
assert_eq!(hr.category(21), PluralCategory::One);
assert_eq!(hr.category(23), PluralCategory::Few);
```

`PluralRules::new(&locale, kind)` builds the rules; `category(number)` is infallible. The rules come from CLDR for every language it has. A language with no rules uses the root's, where everything is `other`, and a regional tag falls back to its language (`de-AT` uses German's), except where CLDR gives the region its own rules (`pt-PT` and `pt-BR` differ for `0`).

## Categories

CLDR defines six categories. A language uses a subset, always including `other`:

| Category | Keyword | Typical use |
|---|---|---|
| `PluralCategory::Zero` | `zero` | Arabic, Latvian, Welsh |
| `PluralCategory::One` | `one` | Most languages, for 1 |
| `PluralCategory::Two` | `two` | Arabic, Welsh, Slovenian |
| `PluralCategory::Few` | `few` | Polish, Czech, Croatian, Russian: 2 to 4 and similar |
| `PluralCategory::Many` | `many` | Polish and Russian: 5 and up; French and Spanish: millions |
| `PluralCategory::Other` | `other` | Everything else; the only one in Japanese |

Do not read the names as meanings. They are labels for sets of numbers that a language groups together, so `many` in French means 1,000,000, not "a lot". The categories a locale uses, in order from `zero` to `other`:

```rust
use dxn_intl::{PluralCategory, PluralKind, PluralRules, parse_locale};

let categories = |tag: &str| -> Result<Vec<PluralCategory>, dxn_intl::IntlError> {
    let rules = PluralRules::new(&parse_locale(tag)?, PluralKind::Cardinal)?;
    Ok(rules.categories().collect())
};

use PluralCategory::*;
assert_eq!(categories("ja")?, [Other]);
assert_eq!(categories("en")?, [One, Other]);
assert_eq!(categories("hr")?, [One, Few, Other]);
assert_eq!(categories("fr")?, [One, Many, Other]);
assert_eq!(categories("pl")?, [One, Few, Many, Other]);
assert_eq!(categories("ar")?, [Zero, One, Two, Few, Many, Other]);
```

`PluralCategory` has `as_str()` and `Display` for the CLDR keyword, `FromStr` to read one back (an unknown word is `IntlError::InvalidOptions`), and `PluralCategory::ALL` for all six in order. It derives `Ord`, `Hash` and `Copy`, so it works as a map key.

```rust
use dxn_intl::PluralCategory;

assert_eq!(PluralCategory::Few.to_string(), "few");
assert_eq!("many".parse::<PluralCategory>()?, PluralCategory::Many);
assert!("several".parse::<PluralCategory>().is_err());
```

## Choosing a word

Match on the category. `PluralCategory` is not `#[non_exhaustive]`, so a match can list all six, but a `_` arm for `other` is the usual shape:

```rust
use dxn_intl::{PluralCategory, PluralKind, PluralRules, parse_locale};

fn files(rules: &PluralRules, count: u32) -> String {
    let word = match rules.category(count) {
        PluralCategory::One => "datoteka",
        PluralCategory::Few => "datoteke",
        _ => "datoteka",
    };
    format!("{count} {word}")
}

let hr = PluralRules::new(&parse_locale("hr")?, PluralKind::Cardinal)?;
assert_eq!(files(&hr, 1), "1 datoteka");
assert_eq!(files(&hr, 3), "3 datoteke");
assert_eq!(files(&hr, 5), "5 datoteka");
assert_eq!(files(&hr, 21), "21 datoteka");
```

Never write `if n == 1`. Croatian uses `one` for 21 and 101 too, Polish has four forms, and Arabic has six. The rules say which numbers fall in which category; your table needs one entry per category the locale uses, with `other` as the fallback.

:::tip Check a translation table against the locale
`rules.categories()` lists exactly the categories a language needs, so a build step or test can verify that a translator supplied a form for each one. A missing `few` shows up as a failed check instead of a wrong word on screen.
:::

[dXn.I18n](https://dxnsix.github.io/dXn.I18n/) builds on these rules for whole messages: its plural selection is written in the message text and uses this crate for the categories.

## The visible fraction digits count

`category` takes anything that converts to a `Number`: integers directly, and `Number` values you parse or build. A `Number` keeps the digits it was written with, and the plural rules read them, because a language can treat "1" and "1.0" differently:

```rust
use dxn_intl::{Number, PluralCategory, PluralKind, PluralRules, parse_locale};

let en = PluralRules::new(&parse_locale("en")?, PluralKind::Cardinal)?;

assert_eq!(en.category(1), PluralCategory::One);                        // 1 item
assert_eq!(en.category("1.0".parse::<Number>()?), PluralCategory::Other); // 1.0 items
assert_eq!(en.category("1.5".parse::<Number>()?), PluralCategory::Other); // 1.5 items

let hr = PluralRules::new(&parse_locale("hr")?, PluralKind::Cardinal)?;
assert_eq!(hr.category("1.1".parse::<Number>()?), PluralCategory::One);  // 1,1 datoteka
assert_eq!(hr.category("21.0".parse::<Number>()?), PluralCategory::Other);
```

A float cannot carry a trailing zero: `Number::try_from(1.0_f64)` is the integer `1`, because floats are read as the shortest text that round-trips. A plain `f64` has no `Into<Number>`, so `rules.category(1.5)` does not compile; convert it explicitly:

```rust
use dxn_intl::{Number, PluralCategory, PluralKind, PluralRules, parse_locale};

let en = PluralRules::new(&parse_locale("en")?, PluralKind::Cardinal)?;

let one = Number::try_from(1.0_f64)?;
assert_eq!(one.to_string(), "1");
assert_eq!(en.category(one), PluralCategory::One); // not "1.0 items"
```

When the text on screen shows a fixed number of fraction digits ("1.0 items", "2.50 kg"), do not choose the category from the raw value. Format the number first and give the category what the formatter showed: [Numbers: resolve](./numbers.md#resolve) explains how, and the [cookbook](./cookbook.md#count-with-a-plural-word) has the full recipe.

The sign is ignored: `-1` is `one` in Polish, as `1` is.

## Ordinals

`PluralKind::Ordinal` is for position: "1st", "2nd", "3rd", "4th". The categories differ from the cardinal ones, and many languages have only `other`.

```rust
use dxn_intl::{PluralCategory, PluralKind, PluralRules, parse_locale};

let en = PluralRules::new(&parse_locale("en")?, PluralKind::Ordinal)?;
assert_eq!(en.kind(), PluralKind::Ordinal);

fn ordinal(rules: &PluralRules, n: u32) -> String {
    let suffix = match rules.category(n) {
        PluralCategory::One => "st",
        PluralCategory::Two => "nd",
        PluralCategory::Few => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

let labels: Vec<String> = [1, 2, 3, 4, 11, 12, 13, 21, 22, 23, 101].map(|n| ordinal(&en, n)).into();
assert_eq!(
    labels,
    ["1st", "2nd", "3rd", "4th", "11th", "12th", "13th", "21st", "22nd", "23rd", "101st"]
);

let hr = PluralRules::new(&parse_locale("hr")?, PluralKind::Ordinal)?;
assert_eq!(hr.categories().collect::<Vec<_>>(), [PluralCategory::Other]);
```

`PluralKind` is `Cardinal` by default. The suffixes belong to your translations: the rules only say which of `one`, `two`, `few` and `other` applies, so 11, 12 and 13 correctly get `other` in English.

## Reference

| Item | Description |
|---|---|
| `PluralRules::new(&Locale, PluralKind) -> Result<PluralRules>` | Rules of that kind for the locale. Fails only with `IntlError::Data`. |
| `category(impl Into<Number>) -> PluralCategory` | The category of the number, with its visible fraction digits. |
| `categories() -> impl Iterator<Item = PluralCategory>` | The categories the locale uses, `zero` to `other`. |
| `kind() -> PluralKind` | The kind the rules were made with. |
| `PluralKind::{Cardinal, Ordinal}` | Counting things or ordering them; `Cardinal` is the default. |
| `PluralCategory::{Zero, One, Two, Few, Many, Other}` | With `ALL`, `as_str`, `Display` and `FromStr`. |

## Next

[Numbers](./numbers.md) explains `Number` and how to get the value that is shown. [Cookbook](./cookbook.md) combines a count, its formatting and its plural word.
