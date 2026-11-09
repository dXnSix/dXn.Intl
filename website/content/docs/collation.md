---
title: Sorting
description: Collator compares and sorts text in the locale's alphabetical order, with sensitivity, numeric, case-first and punctuation options and the locale's -u-co- collations.
group: Formatting
order: 57
---

`Collator` compares strings as the locale's alphabet orders them, as `Intl.Collator` does: `č` after `c` in Croatian, `ä` with `a` in German and after `z` in Swedish, capitals with their lowercase letters rather than before every lowercase one.

```rust
use dxn_intl::{Collator, CollatorOptions, parse_locale};

let hr = Collator::new(&parse_locale("hr")?, CollatorOptions::default())?;
let mut towns = vec!["Zagreb", "Čakovec", "Cres", "Ćuk", "Split", "Šibenik"];
hr.sort(&mut towns);
assert_eq!(towns, ["Cres", "Čakovec", "Ćuk", "Split", "Šibenik", "Zagreb"]);

let numeric = CollatorOptions { numeric: true, ..CollatorOptions::default() };
let mut files = vec!["file10", "file2", "file1"];
Collator::new(&parse_locale("en")?, numeric)?.sort(&mut files);
assert_eq!(files, ["file1", "file2", "file10"]);
```

| Option | Does |
|---|---|
| `sensitivity` | Which differences count: `Base` (letters only: `a` = `á` = `A`), `Accent` (`a` ≠ `á`), `Case` (`a` ≠ `A`), `Variant` (both, the default). |
| `numeric` | Digits compare as numbers: `2` before `10`. |
| `case_first` | `Locale` (the default), `Upper` (`A` before `a`) or `Lower`. |
| `ignore_punctuation` | Spaces and punctuation are ignored: `co-op` = `coop`. |

- `compare(a, b)` gives an `Ordering`; `sort` and `sort_by_key` sort a slice and keep equal items in their order.
- The locale's `-u-co-` collation is honored (`de-u-co-phonebk` sorts `ä` as `ae`), and so are `-u-kn-` and `-u-kf-` unless the options set them.
- For a search that ignores case and accents, compare with `Sensitivity::Base`.

The collation data is large; a program carries it only when it makes a `Collator`.

Next: [Cookbook](./cookbook.md).
