---
title: Relative times
description: RelativeTimeFormatter for "in 3 days", "2 hours ago" and "yesterday", in every unit from seconds to years, with long, short and narrow words.
group: Formatting
order: 55
---

`RelativeTimeFormatter` writes a time relative to now in one unit, as `Intl.RelativeTimeFormat` does. A positive value is in the future, a negative one in the past.

```rust
use dxn_intl::{RelativeTimeFormatter, RelativeTimeOptions, RelativeTimeStyle, RelativeTimeUnit, parse_locale};

let en = parse_locale("en")?;
let days = RelativeTimeFormatter::new(&en, RelativeTimeUnit::Day, RelativeTimeOptions::default())?;
assert_eq!(days.format(3)?, "in 3 days");
assert_eq!(days.format(-1)?, "1 day ago");

let words = RelativeTimeFormatter::new(
    &en,
    RelativeTimeUnit::Day,
    RelativeTimeOptions::auto(RelativeTimeStyle::Long),
)?;
assert_eq!(words.format(-1)?, "yesterday");
assert_eq!(words.format(0)?, "today");
assert_eq!(words.format(2)?, "in 2 days");
```

| Option | Does |
|---|---|
| `style` | `Long` ("in 3 months", the default), `Short` ("in 3 mo."), `Narrow`. |
| `numeric` | `Always` ("in 1 day", the default) or `Auto`: the locale's words for nearby times ("tomorrow", "last week", "now", German "vorgestern"). |

The units are `Second`, `Minute`, `Hour`, `Day`, `Week`, `Month`, `Quarter` and `Year`. Which unit fits a difference is yours to pick: compare day numbers for days (see [Cookbook](./cookbook.md#today-and-yesterday)), and use the same thresholds your design uses for "5 minutes ago" or "in 2 hours". The value is a `Number`, so `1.5` hours is "in 1.5 hours", and the locale's plural forms follow it (Croatian "za 3 dana", "za 5 dana").

Next: [Lists and names](./lists-and-names.md).
