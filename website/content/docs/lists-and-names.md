---
title: Lists and names
description: ListFormatter for "A, B, and C" and "A or B", and DisplayNames for the names of languages, regions, scripts and locales in a locale's language.
group: Formatting
order: 56
---

## Lists

`ListFormatter` joins items with the locale's separators and words, as `Intl.ListFormat` does:

```rust
use dxn_intl::{ListFormatter, ListStyle, ListType, parse_locale};

let and = ListFormatter::new(&parse_locale("en")?, ListType::And, ListStyle::Long)?;
assert_eq!(and.format(["Ana", "Iva", "Marko"]), "Ana, Iva, and Marko");

let hr = ListFormatter::new(&parse_locale("hr")?, ListType::And, ListStyle::Long)?;
assert_eq!(hr.format(["Ana", "Iva", "Marko"]), "Ana, Iva i Marko");

let or = ListFormatter::new(&parse_locale("es")?, ListType::Or, ListStyle::Long)?;
assert_eq!(or.format(["siete", "ocho"]), "siete u ocho");
```

- `ListType`: `And` (all of them), `Or` (one of them), `Unit` (parts of one measure: "3 feet, 7 inches").
- `ListStyle`: `Long` ("A, B, and C"), `Short` ("A, B, & C"), `Narrow` ("A, B, C").
- `format` takes anything that iterates over text: an array, a `Vec<String>`, an iterator. No items is the empty text, one item is itself.
- The words change with what follows where the language does: Spanish "y" becomes "e" before an "i" sound and "o" becomes "u" before an "o" sound.

## Display names

`DisplayNames` gives the names of languages, regions, scripts and whole locales in a locale's language, as `Intl.DisplayNames` does:

```rust
use dxn_intl::{DisplayNames, DisplayNamesOptions, parse_locale};

let en = DisplayNames::new(&parse_locale("en")?, DisplayNamesOptions::default())?;
assert_eq!(en.language("hr").as_deref(), Some("Croatian"));
assert_eq!(en.region("HR").as_deref(), Some("Croatia"));
assert_eq!(en.script("Cyrl").as_deref(), Some("Cyrillic"));
assert_eq!(en.locale(&parse_locale("sr-Latn-RS")?), "Serbian (Latin, Serbia)");

let hr = DisplayNames::new(&parse_locale("hr")?, DisplayNamesOptions::default())?;
assert_eq!(hr.region("DE").as_deref(), Some("Njemačka"));
```

- `language`, `region` and `script` take a code (`"hr"`, `"HR"` or `"419"`, `"Cyrl"`) and return `None` for text that is not such a code or that the locale has no name for. CLDR names even `Zzzz`: "Unknown Script".
- `locale` names a whole locale: its language's name with its script, region and variants in parentheses. A subtag with no name shows as its code.
- `DisplayNamesOptions::style` is `Long` (the default) or `Short` ("UK" for the United Kingdom, where the locale has a shorter name); `dialect: true` names a locale by its dialect where CLDR has one ("British English" rather than "English (United Kingdom)").

A language picker shows each language's name in that language (`DisplayNames::new(&locale, …)?.locale(&locale)`) or in the user's.

Next: [Sorting](./collation.md).
