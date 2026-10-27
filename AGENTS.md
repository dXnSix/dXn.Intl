# dXn.Intl contributor guidance

Locale-aware plurals, numbers and dates for Rust (the `dxn_intl` crate in `crates/`), on ICU4X's
CLDR data: plural rules for every language CLDR has, number formatting (decimal, percent,
currency) and date and time formatting, all in Rust. What it offers is in `README.md`.

## Boundaries

- No UI or app framework: not NSH, GPUI, RayX or any web framework. CI fails when one appears in
  the dependency tree. Frameworks wrap this crate (RayX's i18n service does, through `dxn_i18n`);
  their glue lives in their own repositories.
- It builds for `wasm32-unknown-unknown`: no threads, files, system locale or clock. The caller
  passes the locale and the local wall time; reading them from the platform is the framework's job.
- Locale data is ICU4X's compiled data. `icu_experimental` (percent and currency) is pinned
  exactly, since its API changes in any release; bump it on purpose and rerun every test.
- Public types are this crate's own; ICU4X types appear only where they are the natural
  vocabulary (`Locale`, and `Number::as_decimal`).

## Validation

Run before committing:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo check --workspace --target wasm32-unknown-unknown
cargo package --workspace
```

Tests are grouped by feature, one file per feature in `crates/dxn_intl/tests/` (`plurals`,
`numbers`, `datetime`, `locales`; select one with `cargo test -p dxn_intl --test <feature>`). A new
feature or behavior change gets a new group, or tests in the group of its feature. Test only what
you changed while iterating, and run the full list above once before committing. Raw diagnostic
output goes to the ignored `artifacts-temp/`, never into commits.

## Consumers

Consumers depend on release tags only:
`dxn_intl = { git = "https://github.com/dXnSix/dXn.Intl", tag = "vX.Y.Z" }`. Never build or
validate a consumer to test a change here; mimic its use in this repository's tests.

## Commits

Use [Conventional Commits](https://www.conventionalcommits.org/) with meaningful scopes
(`feat(number): …`, `fix(plural): …`). Never add Claude co-author or other AI attribution lines.

## Releases

release-plz tags `vX.Y.Z` (Git tags only, no crates.io) and writes `CHANGELOG.md`, which is never
edited by hand. 0.x policy: `fix`, `perf` and `feat` → patch; a breaking change (`!` or a
`BREAKING CHANGE:` footer) → minor, with the break stated in the body and a `Migration:` footer.
A release is merging release-plz's pull request after checking its version and that it changes
only `Cargo.toml`, `Cargo.lock` and `CHANGELOG.md`. Push straight to `main` with
`git fetch origin && git rebase origin/main`, then `git push --force-with-lease origin main`; pull
requests exist only for releases. See `docs/releasing.md`.
