---
title: Platforms
description: Where dxn_intl builds and runs, why it reads nothing from the system, the thread restrictions of its formatters, and how large the compiled-in locale data makes a WebAssembly build.
group: Reference
order: 90
---

`dxn_intl` is pure Rust over ICU4X's compiled data. It opens no files, reads no environment variable, asks for no clock and starts no thread, so the same code runs on every target Rust supports and in the browser. The caller supplies what the platform knows: the locale and the local wall time.

## Targets

| Target | Status | Proof |
|---|---|---|
| Linux, macOS, Windows | Supported | Tests run on all three in CI |
| `wasm32-unknown-unknown` | Supported | `cargo check --target wasm32-unknown-unknown` in CI |
| iOS, Android | Expected to work | Type-checks (`aarch64-apple-ios`, `aarch64-linux-android`); checked while writing these docs, not in CI |
| WASI and other `wasm32` targets | Not tested | |

The minimum Rust version is **1.97** (edition 2024); CI builds with 1.97 and develops on 1.99. There are no Cargo features, and `unsafe` code is forbidden in the crate (`unsafe_code = "forbid"`).

CI also fails when a UI or app framework appears in the dependency tree: the crate stays independent of NSH, GPUI, RayX, Leptos, Dioxus and similar, and frameworks wrap it in their own repositories.

## What the platform has to give you

| Input | Where it comes from | See |
|---|---|---|
| The user's locale | A setting, the `LANG` variable, `navigator.languages`, a header | [Locales](./locales.md#accept-language-and-the-system-locale) |
| The current time | `SystemTime`, `Date.now()` | [Dates and times](./dates-and-times.md#from-an-instant-to-a-wall-time) |
| The user's UTC offset | A zone database such as `jiff`, or `Date.getTimezoneOffset()` in the browser | [Dates and times](./dates-and-times.md#time-zones) |

None of this is read for you, so a unit test can run on any machine with fixed inputs and print the same text.

## Browser and WebAssembly

The crate builds for `wasm32-unknown-unknown` with no extra crates and no JavaScript glue. It exports no `wasm-bindgen` API of its own; call it from your own exported functions.

```bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
```

Because the formatting runs in Rust on the same data, a browser page shows the same text as the native app for the same locale and options, instead of depending on the browser's own `Intl` implementation.

## Data size

The locale data is compiled into the binary (`icu`'s compiled data, with every locale), so there is nothing to fetch at run time, and the linker keeps only the data of the formatters you use: a program that only picks plural categories does not carry the date patterns. There is no Cargo feature to restrict the set of locales.

These sizes were measured while writing these docs at v0.1.0, with Rust 1.99, for a `cdylib` that calls one formatter and is built for `wasm32-unknown-unknown` with `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `strip = true` and `panic = "abort"`:

| Calls | `.wasm` size |
|---|---|
| `PluralRules` only | 75 KiB |
| `NumberFormatter` (currency) only | 132 KiB |
| `DateTimeFormatter` (date and time) only | 856 KiB |
| All three | 911 KiB |

The date and time formatter is by far the largest because of the calendars and the names of months, weekdays and eras in every language. Reproduce the measurement for your own build:

```rust
use dxn_intl::{DateTimeFormatter, DateTimeOptions, DateTime, Length, parse_locale};

#[unsafe(no_mangle)]
pub extern "C" fn format_length(day: u32) -> usize {
    let Ok(locale) = parse_locale("de") else { return 0 };
    let options = DateTimeOptions::date_time(Length::Long);
    let Ok(formatter) = DateTimeFormatter::new(&locale, options) else { return 0 };
    let Ok(moment) = DateTime::date(2026, 1, (day % 28 + 1) as u8) else { return 0 };
    formatter.format(&moment).map_or(0, |text| text.len())
}
```

Put it in a `crate-type = ["cdylib"]` library, build with the profile above and look at the size of `target/wasm32-unknown-unknown/release/*.wasm`. Compression on the wire (gzip or Brotli) shrinks it considerably; measure with your server's settings.

:::tip Pay for what you format
If a page only needs plural forms and numbers, do not construct a `DateTimeFormatter` anywhere in that build. The size is decided by which formatters your code can reach, not by how many locales you use.
:::

## Threads

`PluralRules`, `NumberFormatter` and `DateTimeFormatter` are neither `Send` nor `Sync`. They hold ICU4X data behind `Rc`. In practice:

- A formatter stays on the thread that made it. In a multi-threaded runtime, build it inside the task that uses it, or keep it in a `thread_local!`.
- A GUI toolkit's main thread is the natural owner of a per-locale set.
- `Number`, `DateTime`, `Locale` and `IntlError` are `Send + Sync`, so you can compute values on any thread and format them on the right one.

[The cookbook](./cookbook.md#one-set-of-formatters-per-locale) shows a per-thread cache. Building a formatter took microseconds in a release build when measured while writing these docs, so building one per task is acceptable.

## ICU4X dependencies

| Crate | Version | Used for |
|---|---|---|
| `icu` | 2.4 or newer within 2.x | Plurals, decimals, dates, locales |
| `icu_provider` | 2.4 or newer within 2.x | Data errors |
| `icu_experimental` | exactly 0.7.0 | Percent and currency formatting |
| `fixed_decimal` | 0.7.3 or newer in 0.7, with `ryu` | `Number`, and reading floats as their shortest text |

`icu_experimental` is pinned with `=0.7.0` because ICU4X may change its API in any release; the crate bumps it deliberately and reruns every test. If your own code also uses `icu_experimental`, depend on the same version. `Locale` and `Number::as_decimal` expose ICU4X types; everything else in the public API is this crate's own.

## Next

[Limitations](./limitations.md) lists what is not covered. [Releases](./releasing.md) explains how versions are cut.
