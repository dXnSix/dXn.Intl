---
title: Numbers
description: Number as an exact decimal, NumberFormatter and NumberOptions for decimal, percent and currency, fraction digits, grouping, sign display, and resolve.
group: Formatting
order: 40
---

`NumberFormatter` writes a number in a locale's digits, separators and patterns: `1,234.5` in English, `1.234,5` in German, `12,34,567` in Indian English. It also does percentages and amounts of money. You make one per locale and options, then call `format`.

```rust
use dxn_intl::{NumberFormatter, NumberOptions, parse_locale};

let de = NumberFormatter::new(&parse_locale("de-DE")?, NumberOptions::default())?;
assert_eq!(de.format(1234.5)?, "1.234,5");

let percent = NumberFormatter::new(&parse_locale("de-DE")?, NumberOptions::percent())?;
assert_eq!(percent.format(0.25)?, "25\u{a0}%");

let usd = NumberFormatter::new(&parse_locale("en-US")?, NumberOptions::currency("USD"))?;
assert_eq!(usd.format(1234.5)?, "$1,234.50");
```

`format` takes anything that converts to a `Number`: the integer types, `f32`, `f64` and `Number` itself. A `&str` does not convert; parse it into a `Number` first. A float that is `NaN` or infinite is `IntlError::InvalidNumber`.

## Locale differences

The same value in six locales, with the default options:

```rust
use dxn_intl::{NumberFormatter, NumberOptions, parse_locale};

let show = |tag: &str, value: f64| -> Result<String, dxn_intl::IntlError> {
    NumberFormatter::new(&parse_locale(tag)?, NumberOptions::default())?.format(value)
};

assert_eq!(show("en-US", 1234567.891)?, "1,234,567.891");
assert_eq!(show("de-DE", 1234567.891)?, "1.234.567,891");
assert_eq!(show("fr-FR", 1234.5)?, "1\u{202f}234,5");   // narrow no-break space
assert_eq!(show("en-IN", 1234567.0)?, "12,34,567");     // lakh grouping
assert_eq!(show("ar-EG", 12.0)?, "١٢");                 // Arabic-Indic digits
assert_eq!(show("ar-EG-u-nu-latn", 12.0)?, "12");       // the user asked for Latin digits
```

The numbering system comes from the locale, and a `-u-nu-` extension overrides it. A locale parsed from the user's own tag keeps that preference; see [Locales](./locales.md#unicode-extensions).

## Number

`Number` is an exact decimal: the digits you give it stay as written. It exists because a plural rule and a formatter must agree on what the user sees, and a float cannot say "1.50".

| From | Code | Notes |
|---|---|---|
| Integers | `Number::from(42)` | `i8` to `i128`, `isize`, `u8` to `u128`, `usize` |
| Text | `"1.50".parse::<Number>()?` | Keeps every digit, including trailing zeros |
| `f64` | `Number::try_from(1.5)?` | The shortest text that reads back as the same float |
| `f32` | `Number::try_from(0.1_f32)?` | Through its own shortest text: `0.1`, not `0.100000001490116` |
| ICU4X decimal | `Number::from(decimal)` | `fixed_decimal::Decimal`; `as_decimal()` gives it back |

```rust
use dxn_intl::Number;

let price: Number = "1.50".parse()?;
assert_eq!(price.to_string(), "1.50");
assert!(!price.is_integer());
assert_eq!(Number::from(3).to_string(), "3");

assert_eq!(Number::try_from(0.1)?.to_string(), "0.1");
assert_eq!(Number::try_from(0.1_f32)?.to_string(), "0.1");
assert_eq!(Number::try_from(0.1 + 0.2)?.to_string(), "0.30000000000000004");
assert_eq!(Number::try_from(-2.5)?.to_f64(), -2.5);
```

The text a number reads is ASCII only: an optional sign, digits, an optional `.` fraction and an optional exponent. Surrounding whitespace is ignored.

| Text | Result |
|---|---|
| `"1.50"`, `" 1.50 "` | `1.50` |
| `"007"` | `007` (leading zeros are kept) |
| `".5"`, `"+5"`, `"-0"` | `0.5`, `+5`, `-0` |
| `"1e3"` | `1000` |
| `"5."`, `"1,5"`, `"1_000"`, `""` | error (`InvalidNumber`) |
| `"١٢"` (Arabic-Indic digits) | error: localized input is not parsed |

`Display` gives plain ASCII text, never localized (`-1234.50`). `is_integer()` is false when there are fraction digits, trailing zeros included (`"2.0"`). `to_f64()` is the nearest float.

:::warning Floats lose trailing zeros
`Number::try_from(1.0_f64)` is `1`, and so is `100.0`. A float has no "1.0" to remember. If the digits matter to the user (a price, a measurement with a fixed precision), keep the value as text or as integers, and let the formatter pad it with fraction digits.
:::

## NumberOptions

`NumberOptions` is a plain struct with public fields, and `Default` is a decimal number with ECMA-402's defaults.

| Field | Type | Default | Effect |
|---|---|---|---|
| `style` | `NumberStyle` | `Decimal` | `Decimal`, `Percent` or `Currency(code)` |
| `minimum_integer_digits` | `u8` | `1` | Leading zeros pad the integer part (`007`) |
| `minimum_fraction_digits` | `Option<u8>` | `0` | Trailing zeros pad the fraction |
| `maximum_fraction_digits` | `Option<u8>` | `3`; `0` for percent | Half-away-from-zero rounding |
| `grouping` | `Grouping` | `Auto` | When grouping separators appear |
| `sign_display` | `SignDisplay` | `Auto` | When the sign appears |

Constructors: `NumberOptions::percent()`, `NumberOptions::currency("EUR")` and `.with_fraction_digits(n)`, which sets the minimum and maximum to `n`. Build anything else with struct update syntax:

```rust
use dxn_intl::{Grouping, NumberFormatter, NumberOptions, parse_locale};

let options = NumberOptions {
    minimum_integer_digits: 3,
    grouping: Grouping::Never,
    ..NumberOptions::default()
};
let formatter = NumberFormatter::new(&parse_locale("en")?, options)?;
assert_eq!(formatter.format(7)?, "007");
assert_eq!(formatter.options().minimum_integer_digits, 3);
```

The options are checked in `NumberFormatter::new`, not when you build the struct.

## Fraction digits

The fraction is rounded half away from zero to the maximum, trailing zeros are removed, and zeros are added up to the minimum.

```rust
use dxn_intl::{NumberFormatter, NumberOptions, parse_locale};

let en = parse_locale("en")?;
let format = |options: NumberOptions, value: f64| -> Result<String, dxn_intl::IntlError> {
    NumberFormatter::new(&en, options)?.format(value)
};

let two = NumberOptions::default().with_fraction_digits(2);
assert_eq!(format(two.clone(), 2.5)?, "2.50");
assert_eq!(format(two.clone(), 2.005)?, "2.01");   // half away from zero
assert_eq!(format(two, -1.005)?, "-1.01");

assert_eq!(format(NumberOptions::default(), 1.23456)?, "1.235"); // at most 3 by default

let whole = NumberOptions { maximum_fraction_digits: Some(0), ..NumberOptions::default() };
assert_eq!(format(whole, 2.5)?, "3");

let at_least_one = NumberOptions { minimum_fraction_digits: Some(1), ..NumberOptions::default() };
assert_eq!(format(at_least_one, 5.0)?, "5.0");
```

Rounding works on the number's decimal text, so `2.005` becomes `2.01`, where Rust's `format!("{:.2}", 2.005_f64)` prints `2.00` because the binary float is slightly below 2.005. The formatter reads the shortest text of the float, which is what you typed.

| You set | Result |
|---|---|
| Only a minimum above the default maximum | The maximum rises to the minimum (`Some(5)` shows five digits) |
| A minimum above the maximum | `IntlError::InvalidOptions` from `new` |
| More than 100 fraction or integer digits | `IntlError::InvalidOptions` from `new` |

## Percent

A percentage scales the value by 100 and shows no fraction digits by default. `0.25` is `25%`.

```rust
use dxn_intl::{NumberFormatter, NumberOptions, parse_locale};

let en = parse_locale("en")?;
let percent = NumberFormatter::new(&en, NumberOptions::percent())?;
assert_eq!(percent.format(0.25)?, "25%");
assert_eq!(percent.format(0.1234)?, "12%");

let precise = NumberFormatter::new(&en, NumberOptions::percent().with_fraction_digits(1))?;
assert_eq!(precise.format(0.1234)?, "12.3%");

let turkish = NumberFormatter::new(&parse_locale("tr")?, NumberOptions::percent())?;
assert_eq!(turkish.format(0.256)?, "%26"); // the sign goes first in Turkish
```

German writes `25 %` with a non-breaking space; Turkish puts the sign in front. Pass the fraction (0.25), not the percentage (25), or you get `2,500%`.

## Currency

`NumberOptions::currency("EUR")` takes an ISO 4217 code. The symbol, its position and spacing come from the locale, and the number of fraction digits from the currency:

```rust
use dxn_intl::{NumberFormatter, NumberOptions, parse_locale};

let show = |tag: &str, code: &str, value: f64| -> Result<String, dxn_intl::IntlError> {
    NumberFormatter::new(&parse_locale(tag)?, NumberOptions::currency(code))?.format(value)
};

assert_eq!(show("en-US", "USD", 1234.5)?, "$1,234.50");
assert_eq!(show("de-DE", "EUR", 1234.5)?, "1.234,50\u{a0}€");
assert_eq!(show("fr", "USD", 1234.5)?, "1\u{202f}234,50\u{a0}$US");
assert_eq!(show("en", "CAD", 1234.5)?, "CA$1,234.50");   // another country's dollar
assert_eq!(show("en-CA", "CAD", 1234.5)?, "$1,234.50");  // the user's own
assert_eq!(show("en-US", "JPY", 1234.5)?, "¥1,235");     // yen have no fraction
assert_eq!(show("en", "BHD", 1234.5)?, "BHD\u{a0}1,234.500"); // dinars have three
```

The code is trimmed and case-insensitive (`" eur "` works). Anything that is not three ASCII letters is `IntlError::InvalidCurrency`. Any other three letters are accepted, known or not, and unknown codes show as the code (`ZZZ 1,234.50`).

When the locale's data has no symbol for a currency, the ISO code is shown (`BHD` above, and `RSD` in `sr`). In that case ICU4X also prints a line starting `ICU4X data error: Missing data for identifier` to standard error, and the output is still the code.

:::warning Some options are ignored for currencies
A currency always shows its own digits, so `minimum_fraction_digits`, `maximum_fraction_digits` and `with_fraction_digits` have no effect on it. The `grouping` option is not applied either in v0.1.0: `Grouping::Never` still groups `12,345.00`, and the locale decides. `minimum_integer_digits` and `sign_display` do apply.
:::

Money should not travel as `f64`. Build a `Number` from text or from whole minor units; [the cookbook](./cookbook.md#format-money-from-minor-units) shows both.

## Grouping

| `Grouping` | Behavior |
|---|---|
| `Auto` | As the locale does |
| `Never` | No separators: `1234567` |
| `Always` | In v0.1.0 the same as `Auto` |
| `Min2` | Only when there are at least two digits in the first group: from five digits on |

Locales differ in when `Auto` starts grouping. English groups from four digits (`1,234`); Spanish, Polish and Portuguese (Portugal) start at five:

```rust
use dxn_intl::{Grouping, NumberFormatter, NumberOptions, parse_locale};

let show = |tag: &str, grouping: Grouping, value: i32| -> Result<String, dxn_intl::IntlError> {
    let options = NumberOptions { grouping, ..NumberOptions::default() };
    NumberFormatter::new(&parse_locale(tag)?, options)?.format(value)
};

assert_eq!(show("en", Grouping::Auto, 1234)?, "1,234");
assert_eq!(show("es", Grouping::Auto, 1234)?, "1234");
assert_eq!(show("es", Grouping::Auto, 12345)?, "12.345");
assert_eq!(show("en", Grouping::Min2, 1234)?, "1234");
assert_eq!(show("en", Grouping::Min2, 12345)?, "12,345");
assert_eq!(show("en", Grouping::Never, 1234567)?, "1234567");
```

`Always` is documented as grouping "even where the locale would leave four digits ungrouped", but ICU4X's decimal formatter treats it as `Auto`, so Spanish `1234` stays `1234` (checked with icu_decimal 2.4). Use `Auto` and expect the locale's own rule.

## Sign display

Each value is the result of `format` for the number in the first row, with `en`:

| `SignDisplay` | `-5` | `5` | `0` | `-0` | `-0.0001` |
|---|---|---|---|---|---|
| `Auto` | `-5` | `5` | `0` | `-0` | `-0` |
| `Never` | `5` | `5` | `0` | `0` | `0` |
| `Always` | `-5` | `+5` | `+0` | `-0` | `-0` |
| `ExceptZero` | `-5` | `+5` | `0` | `0` | `0` |
| `Negative` | `-5` | `5` | `0` | `0` | `0` |

```rust
use dxn_intl::{NumberFormatter, NumberOptions, SignDisplay, parse_locale};

let change = NumberOptions {
    sign_display: SignDisplay::ExceptZero,
    ..NumberOptions::percent().with_fraction_digits(1)
};
let formatter = NumberFormatter::new(&parse_locale("en")?, change)?;
assert_eq!(formatter.format(0.125)?, "+12.5%");
assert_eq!(formatter.format(-0.125)?, "-12.5%");
assert_eq!(formatter.format(0)?, "0.0%");
```

Rounding can leave a negative zero: `-0.0001` at three digits is `-0` under `Auto`. `SignDisplay::Negative` hides the sign when the shown value is zero; `ExceptZero` also drops `+` from it.

## resolve

`NumberFormatter::resolve` returns the `Number` the formatter would print, before localization: scaled for percentages, rounded and padded to the fraction digits, padded to the integer digits, with the sign display applied. `format` is `resolve` plus the locale.

```rust
use dxn_intl::{NumberFormatter, NumberOptions, parse_locale};

let en = parse_locale("en")?;

let one_decimal = NumberFormatter::new(&en, NumberOptions::default().with_fraction_digits(1))?;
assert_eq!(one_decimal.resolve(1)?.to_string(), "1.0");
assert_eq!(one_decimal.resolve(2.449)?.to_string(), "2.4");

let percent = NumberFormatter::new(&en, NumberOptions::percent())?;
assert_eq!(percent.resolve(0.256)?.to_string(), "26");
```

The point is plural selection. The category must come from the value the user reads, not from the value you started with. A count of `1` shown with one fraction digit reads "1.0 items", which is `other` in English; the raw `1` would be `one`:

```rust
use dxn_intl::{NumberFormatter, NumberOptions, PluralCategory, PluralKind, PluralRules, parse_locale};

let en = parse_locale("en")?;
let numbers = NumberFormatter::new(&en, NumberOptions::default().with_fraction_digits(1))?;
let plurals = PluralRules::new(&en, PluralKind::Cardinal)?;

assert_eq!(plurals.category(1), PluralCategory::One);
assert_eq!(plurals.category(numbers.resolve(1)?), PluralCategory::Other);
```

Percentages are the same: `0.01` is `1` after resolving, so the plural of "1 percent" is read from `1`.

:::warning resolve does not round or pad currencies
For `NumberStyle::Currency`, `resolve` leaves the number as it is, because the currency formatter rounds and pads at print time. `resolve(1)` is `1`, although the text is `$1.00`. For plural selection next to an amount, resolve with a decimal formatter that has the fraction digits the text shows (`NumberOptions::default().with_fraction_digits(2)`), as in the [cookbook](./cookbook.md#count-with-a-plural-word).
:::

## Reference

| Item | Description |
|---|---|
| `NumberFormatter::new(&Locale, NumberOptions) -> Result<NumberFormatter>` | Checks the options and the currency code and loads the locale's data. |
| `format(impl TryInto<Number>) -> Result<String>` | The localized text. |
| `resolve(impl TryInto<Number>) -> Result<Number>` | The value as shown, not localized. |
| `options() -> &NumberOptions` | The options the formatter was made with. |
| `NumberStyle::{Decimal, Percent, Currency(String)}` | What the number stands for. |
| `Grouping::{Auto, Never, Always, Min2}` | Grouping separators. |
| `SignDisplay::{Auto, Never, Always, ExceptZero, Negative}` | The sign. |

## Next

[Dates and times](./dates-and-times.md) is the third formatter. [Cookbook](./cookbook.md) has prices from minor units and the count-plus-word recipe.
