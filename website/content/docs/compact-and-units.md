---
title: Compact numbers and units
description: CompactNumberFormatter for "1.2K" and "3.4 million", and UnitFormatter for amounts of CLDR's units of length, area, volume, mass and duration.
group: Formatting
order: 45
---

## Compact numbers

`CompactNumberFormatter` writes a number with its magnitude as an abbreviation or a word, as CLDR's compact patterns for the locale say:

```rust
use dxn_intl::{CompactDisplay, CompactNumberFormatter, parse_locale};

let short = CompactNumberFormatter::new(&parse_locale("en")?, CompactDisplay::Short)?;
assert_eq!(short.format(1234)?, "1.2K");
assert_eq!(short.format(1_250_000)?, "1.3M");
assert_eq!(short.format(999_950)?, "1M");

let long = CompactNumberFormatter::new(&parse_locale("de")?, CompactDisplay::Long)?;
assert_eq!(long.format(3_400_000)?, "3,4 Millionen");
```

- The magnitudes are the locale's: German abbreviates from a million (`1234`, `2,5 Mio.`), Japanese counts in ten thousands (`1.2万`), Indian English in lakhs (`15L`).
- The number is rounded as `Intl.NumberFormat`'s compact notation rounds it: half away from zero, to whole units of the magnitude from two digits on (`15K`), else to two significant digits (`1.3M`, `0.012`). Rounding up can reach the next magnitude (`999,950` is `1M`).
- A `-u-nu-` numbering system in the locale is honored.

## Units

`UnitFormatter` writes amounts of a unit of measure, named by its CLDR id:

```rust
use dxn_intl::{UnitDisplay, UnitFormatter, UnitOptions, parse_locale};

let en = parse_locale("en")?;
assert_eq!(UnitFormatter::new(&en, "kilometer", UnitOptions::default())?.format(5.25)?, "5.25 km");

let miles = UnitFormatter::new(&en, "mile", UnitOptions::new(UnitDisplay::Long))?;
assert_eq!(miles.format(1)?, "1 mile");
assert_eq!(miles.format(26.2)?, "26.2 miles");

let litres = UnitFormatter::new(&parse_locale("hr")?, "liter", UnitOptions::new(UnitDisplay::Long))?;
assert_eq!(litres.format(3)?, "3 litre");
assert_eq!(litres.format(5)?, "5 litara");
```

| Option | Does |
|---|---|
| `display` | `Short` (`5 km`, the default), `Long` (`5 kilometers`) or `Narrow` (`5km`, `6′`). |
| `minimum_fraction_digits` | Pads the fraction with zeros (`0` by default). |
| `maximum_fraction_digits` | Rounds the fraction half away from zero (`3` by default). |

The units are CLDR's of length, area, volume, mass and duration (`kilometer`, `foot`, `square-meter`, `hectare`, `liter`, `gallon`, `kilogram`, `pound`, `hour`, `week`, …); `UnitFormatter::units()` lists them. The amount is shown as given, never converted. Another id, including units of other kinds such as `celsius` or `kilometer-per-hour`, is an `InvalidOptions` error: ICU4X 0.7's unit data has no names for them. In that data some locales' long names of metric units are their symbols (`en`: "5 km" for "5 kilometers").

Next: [Dates and times](./dates-and-times.md).
