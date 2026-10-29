---
title: Releases
description: How dxn_intl is versioned and released as git tags, how to depend on and upgrade a tag, and the 0.x versioning policy.
group: Project
order: 100
---

`dxn_intl` is released as git tags `vX.Y.Z`, each with a GitHub Release. Nothing goes to crates.io or any other registry. The manifest stays crates.io-ready (`cargo package --workspace` passes in CI), but no registry release is made. [release-plz](https://release-plz.dev) turns the Conventional Commits on `main` into versions, changelog sections, tags and GitHub Releases.

## Depend on a release

```toml
[dependencies]
dxn_intl = { git = "https://github.com/dXnSix/dXn.Intl", tag = "v0.1.0" }
```

`Cargo.lock` records the tagged commit. To upgrade, change the tag to a newer one from the [Releases page](https://github.com/dXnSix/dXn.Intl/releases); Cargo resolves it on the next build. Release tags are never moved or deleted, so a bad release is fixed forward with a new patch release.

The first release is `v0.1.0`. A change that spans this crate and a consumer is released here first, then the consumer bumps its tag; consumers never build against a local checkout of this repository.

## Versioning

The crate is 0.x, so the policy shifts one position: a breaking change is a minor bump.

| Commit | Release |
|---|---|
| `fix:`, `perf:`, `feat:` | patch (`0.1.0` to `0.1.1`) |
| Breaking: `!` after the type or scope, or a `BREAKING CHANGE:` footer | minor (`0.1.x` to `0.2.0`) |
| `docs:`, `test:`, `ci:`, `build:`, `refactor:`, `chore:` | none on their own; listed in the next release's changelog |

A breaking commit's body states the break, and a `Migration:` footer tells consumers what to change; the changelog prints it under the entry.

:::warning Read the changelog before bumping a minor version
In 0.x a `0.1.x` to `0.2.0` upgrade can break the API. Check the release notes for a `Migration:` line before changing the tag. Output text can also change with a CLDR update in ICU4X, so rerun tests that assert exact strings.
:::

## How a release happens

1. Work lands on `main` as Conventional Commits. Each push runs rustfmt, clippy, the tests on Linux, macOS and Windows, a `wasm32-unknown-unknown` check and `cargo package`; separate jobs build the rustdoc with `-D missing_docs` and check the minimum Rust version, 1.97.
2. release-plz keeps a release pull request open with the version bump and the new `CHANGELOG.md` section. Pull requests exist only for releases.
3. Before merging, check that the proposed version is the expected one (a minor bump only when a commit since the last tag is breaking), that the pull request changes only `Cargo.toml`, `Cargo.lock` and `CHANGELOG.md`, and that its CI is green.
4. The merge makes release-plz tag `vX.Y.Z` and create the GitHub Release with that version's changelog section as its notes. It tags whenever `Cargo.toml` holds a version that has no tag yet.

release-plz runs in git-only mode (`git_only = true`, `publish = false`), so versions are read from the `v*` tags. Its jobs use a fine-grained access token stored as the repository secret `RELEASE_PLZ_TOKEN`; when the token expires the release runs fail at checkout and releases stop until it is replaced.

## Next

[Get started](./getting-started.md) shows the install in context. [Limitations](./limitations.md) notes what changes to expect while the crate is 0.x.
