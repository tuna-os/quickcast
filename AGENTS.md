# AGENTS.md

Guidance for people and tools working in this repository. Facts here come from
`README.md`, `meson.build`, `src/meson.build`, `clippy.toml`, `scripts/` and
`.github/workflows/`.

## Overview

Quickcast is a GNOME screen recorder (Rust, GTK4/libadwaita, GStreamer),
packaged as a Flatpak (`org.tunaos.Quickcast.json`). It is a fork of Kooha.
See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the code map and
[README.md](README.md) for user-facing behavior.

## Layout

- `src/`: Rust application (`src/recording/` holds the recording boundaries).
- `extension/`: GNOME Shell companion extension.
- `data/`, `po/`: app metadata, resources and translations.
- `scripts/`: local build and test helpers.
- `build-aux/`, `meson.build`, `org.tunaos.Quickcast.json`: build and packaging.

## Build and test

Prerequisites (from the README): the GNOME 50 SDK and Platform and the Rust
stable SDK extension, installed with Flatpak:

```sh
flatpak install --user flathub org.gnome.Sdk//50 org.gnome.Platform//50 org.freedesktop.Sdk.Extension.rust-stable//25.08
```

Then:

```sh
./scripts/build-local.sh      # produces Quickcast.flatpak
flatpak install --user --reinstall ./Quickcast.flatpak
./scripts/test-local.sh       # Xvfb + private D-Bus session
```

CI (`.github/workflows/ci.yml`) runs the same two scripts on every pull request.
The Meson tests defined in `src/meson.build` are `cargo-test`, `cargo-clippy`
and `cargo-clippy-temp-dir`.

## Lint rules

- `clippy.toml` disallows `std::env::temp_dir`. Use a private directory
  (for example `tempfile`) or allow it with a comment when the name is unique.
- `clippy::too_many_lines` is enforced with a threshold of 150 lines. Split a
  long function instead of raising the limit.
- `typos.toml` configures spell checking (excludes `po/*.po` and icon SVGs).
- Workflows are checked by `actions-security.yml` (lockfile verify and zizmor).
  Keep `.github/workflows/actions.lock` in sync with workflow changes.

A standalone formatter command is not documented yet.

## Conventions

- Recent commits use Conventional Commit style prefixes (`fix:`, `docs:`,
  `ci:`, `refactor:`), often with the PR number in parentheses.
- Never log screencast restore tokens or portal responses.
- Real portal selection, hardware A/V sync and per-app paste behavior need
  manual testing; CI covers synthetic recordings only.
