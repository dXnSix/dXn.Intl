---
title: Errors
description: The IntlError variants, which call returns which, what their messages look like, and how to handle bad input from users and bad options from code.
group: Reference
order: 70
---

Every fallible call returns `Result<T, IntlError>`. The crate also exports the alias `dxn_intl::Result<T, E = IntlError>`. `IntlError` is `Clone`, `Debug`, `PartialEq` and `Eq`, is `Send + Sync`, and implements `std::error::Error`, so it works with `?`, `Box<dyn std::error::Error>`, `anyhow` and `thiserror`. Formatting itself (`PluralRules::category`) cannot fail.

```rust
use dxn_intl::{IntlError, NumberFormatter, NumberOptions, parse_locale};

fn price(tag: &str, code: &str, amount: f64) -> Result<String, IntlError> {
    let formatter = NumberFormatter::new(&parse_locale(tag)?, NumberOptions::currency(code))?;
    formatter.format(amount)
}

assert_eq!(price("en-US", "USD", 5.0)?, "$5.00");
assert!(matches!(price("en-US", "EURO", 5.0), Err(IntlError::InvalidCurrency(_))));
assert!(matches!(price("not a tag", "USD", 5.0), Err(IntlError::InvalidLocale(_))));
assert!(matches!(price("en-US", "USD", f64::NAN), Err(IntlError::InvalidNumber(_))));
```

## Variants

Every variant carries one `String`: the offending input or a reason.

| Variant | Meaning | Message |
|---|---|---|
| `InvalidLocale(tag)` | The text is not a BCP 47 (or POSIX) language tag | `` `not a tag` is not a valid language tag `` |
| `InvalidNumber(text)` | Malformed number text, or a `NaN` or infinite float | `` `abc` is not a number that can be formatted `` |
| `InvalidOptions(reason)` | Options that do not go together, or an unknown keyword | `invalid options: minimum fraction digits 3 is above the maximum 1` |
| `InvalidDateTime(reason)` | A date or time outside the calendar | `invalid date or time: The day = 30 argument is out of range 1..=28` |
| `InvalidCurrency(code)` | Not three ASCII letters | `` `EURO` is not an ISO 4217 currency code `` |
| `Data(reason)` | ICU4X could not provide the locale data | `locale data: ...` |

`Data` comes from ICU4X's `DataError`. The crate compiles its data in, so you should not meet it in normal use; handle it as an internal failure.

The text inside `InvalidDateTime` and some `InvalidOptions` comes from ICU4X and is not stable. A date option that cannot be shown reads `invalid options: InvalidDateFields` or `invalid options: MissingDateFields`. Match on the variant, show your own message to users, and log the text.

## Which call returns what

| Call | Errors |
|---|---|
| `parse_locale` | `InvalidLocale` |
| `Number::from_str` (`"1.5".parse()`) | `InvalidNumber` |
| `Number::try_from(f64 or f32)` | `InvalidNumber` for `NaN` and infinities |
| `NumberFormatter::new` | `InvalidOptions` (minimum above maximum, more than 100 digits), `InvalidCurrency`, `Data` |
| `NumberFormatter::format`, `resolve` | `InvalidNumber` when the argument does not convert (`NaN`, infinity) |
| `PluralRules::new` | `Data` |
| `PluralCategory::from_str` | `InvalidOptions` |
| `DateTime::date`, `new`, `from_unix_millis` | `InvalidDateTime` |
| `HourCycle::from_str` | `InvalidOptions` |
| `DateTimeFormatter::new` | `InvalidOptions` (month, year-month or year with a time; neither date nor time), `Data` |
| `DateTimeFormatter::format` | `InvalidDateTime` for a `DateTime` outside the calendar or the year range |

## Handling them

Errors split into two groups by who caused them. Input from a user (a typed amount, a tag from a header) fails at the edge and deserves a message or a default. Options and codes from your own code fail on the first call during development and then never again, so `?` is enough.

```rust
use dxn_intl::{IntlError, Locale, Number, parse_locale};

/// The user's locale from a header or setting, or the app default when it is not a tag.
fn locale_or(default: &Locale, tag: &str) -> Locale {
    parse_locale(tag).unwrap_or_else(|_| default.clone())
}

/// A typed amount: `Ok(None)` for empty input, a message for bad input.
fn parse_amount(input: &str) -> Result<Option<Number>, String> {
    if input.trim().is_empty() {
        return Ok(None);
    }
    match input.parse::<Number>() {
        Ok(number) => Ok(Some(number)),
        Err(IntlError::InvalidNumber(text)) => Err(format!("`{text}` is not a number")),
        Err(other) => Err(other.to_string()),
    }
}

let default = parse_locale("en")?;
assert_eq!(locale_or(&default, "C").to_string(), "en");
assert_eq!(locale_or(&default, "de_AT.UTF-8").to_string(), "de-AT");

assert!(parse_amount("")?.is_none());
assert_eq!(parse_amount("12.50")?.map(|n| n.to_string()), Some("12.50".to_string()));
assert_eq!(parse_amount("12,50"), Err("`12,50` is not a number".to_string()));
```

:::warning Localized input is not parsed
`Number` reads ASCII digits and a `.` only, so a German user typing `12,50` gets `InvalidNumber`. The crate formats locale-specific text and does not read it back. Normalize what the user types (replace the locale's decimal separator, strip grouping) before parsing, or use a field that gives you a number.
:::

`IntlError` is not `#[non_exhaustive]`, so a `match` can list all six variants. Add a `_` arm anyway if you want a later 0.x release to compile unchanged when a variant is added.

## Next

[Limitations](./limitations.md) lists what the crate does not do. [Cookbook](./cookbook.md) has recipes that handle errors at the edge.
