# AGENTS.md — Quickcast Project Guide

**Quickcast** is a compact GNOME screen recorder for quick explanations: record screen with microphone audio and optional webcam, then paste the finished video as an attachment.

This document guides hives and agents exploring the repository.

## Repository Overview

- **Type**: GNOME application (Rust + GTK4 + Libadwaita)
- **Distribution**: Flatpak (`org.tunaos.Quickcast`)
- **Origin**: Fork of [Kooha](https://github.com/SeaDve/Kooha) (GPL-3.0-or-later)
- **Target Platforms**: x86_64, GNOME 50 Flatpak runtime
- **Key Features**: Screen recording, microphone + desktop audio, webcam composition, MP4 preset, automatic file copying

## Key Files and Structure

```
quickcast/
├── Cargo.toml                  # Rust dependency manifest and build config
├── Cargo.lock                  # Locked dependency versions
├── meson.build                 # Meson build configuration
├── meson_options.txt           # Build options (features, flags)
├── org.tunaos.Quickcast.json   # Flatpak manifest
├── org.tunaos.Quickcast.json.in # Flatpak manifest template
├── README.md                   # User documentation
├── COPYING                     # GPL-3.0 license
├── src/                        # Rust source code
│   ├── main.rs                 # Application entry point
│   ├── recording/              # Recording logic
│   ├── ui/                     # GTK4 UI components
│   └── ...
├── data/                       # Application data (icons, desktop files, etc.)
├── po/                         # Localization files
├── scripts/                    # Build and utility scripts
├── build-aux/                  # Build auxiliary files
└── extension/                  # GNOME Shell extension (optional)
```

## Build and Development

### Prerequisites

- Rust toolchain (stable, edition 2021)
- Meson and Ninja build system
- GTK 4 and Libadwaita development libraries
- GStreamer development headers (for audio/video)
- FFmpeg libraries (encoding)

### Build Commands

```bash
# Standard Rust build
cargo build --release

# Meson build (used for Flatpak)
meson setup builddir
meson compile -C builddir

# Build as Flatpak
flatpak-builder --user --install builddir org.tunaos.Quickcast.json
```

### Running

```bash
# From cargo
cargo run

# From Flatpak
flatpak run org.tunaos.Quickcast
```

## Testing and Quality

### Test Execution

```bash
# Run all tests
cargo test

# Run specific test module
cargo test recording::tests

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

### Validation

```bash
# Format code
cargo fmt

# Lint with clippy
cargo clippy --all-targets

# Check dependencies for vulnerabilities
cargo audit
```

## CI/CD Pipeline

The repository uses GitHub Actions for:

- **Format & Lint** — `cargo fmt`, `cargo clippy`
- **Tests** — `cargo test` on Ubuntu
- **Flatpak Build** — Build and validate Flatpak artifact
- **Release** — Tag-based releases with artifact publishing

Key checks that must pass before merge:

- Code formatting (`cargo fmt --check`)
- Clippy linting (`cargo clippy`)
- Unit and integration tests
- Flatpak build validation

## Feature Flags

Key Cargo features:

- Default features are typically minimal; Flatpak build adds platform-specific features
- Check `Cargo.toml` `[features]` section for available options
- Some features may require additional system dependencies

## Dependency Management

### Adding Dependencies

1. Add to `Cargo.toml` with exact or conservative versions
2. Justify in PR description
3. Test build in Flatpak context (dependencies must be available in Flatpak runtime or bundled)

### Updating Dependencies

```bash
# Check for outdated dependencies
cargo outdated

# Update carefully and test thoroughly
cargo update -p <crate-name>
```

### Important Constraints

- GStreamer version must match Flatpak runtime version (GNOME 50)
- FFmpeg APIs must be compatible with packaged version
- Audio library dependencies must be available in the runtime

## Localization

The project uses GNU gettext for translations:

- `.po` files in `po/` directory
- Wrapping UI strings with `_("string")` in Rust
- Build extracts strings to `.pot` template
- Translators update `.po` files for each locale

Add new locales by creating `.po` file and adding to `po/LINGUAS`.

## Releasing

Release process (see COPYING and manifest for exact steps):

1. Update version in `Cargo.toml` and `meson.build`
2. Update CHANGELOG
3. Tag commit: `git tag vX.Y.Z`
4. Push tag to trigger CI/CD release workflow
5. CI builds Flatpak and creates GitHub Release
6. Flatpak is published to the tuna-os Flatpak repository

Version follows semver (X.Y.Z).

## Known Constraints

- **Platform**: x86_64 only (initial release target)
- **Runtime**: GNOME 50 Flatpak runtime minimum
- **Audio Input**: Requires PulseAudio or PipeWire
- **Webcam**: Optional; requires V4L2-compatible device

## Extension (Optional)

Directory `extension/` contains optional GNOME Shell extension code (if applicable). Build and test separately from main app.

## Troubleshooting Build Issues

### "Dependency X not found"

Usually means development library is not installed. Reinstall via package manager.

### Meson configuration fails

```bash
# Reconfigure
meson setup --reconfigure builddir

# Clean and start fresh
rm -rf builddir
meson setup builddir
meson compile -C builddir
```

### Flatpak build fails

Check that all dependencies are available in the Flatpak runtime. See `org.tunaos.Quickcast.json` manifest for runtime version and bundled modules.

## Related Documentation

- `README.md` — User guide and feature overview
- `COPYING` — GPL-3.0 license text
- `org.tunaos.Quickcast.json` — Flatpak build configuration
- Upstream: [Kooha](https://github.com/SeaDve/Kooha)

## Development Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/name`
3. Make changes and test locally
4. Run format, lint, and tests
5. Commit with DCO sign-off: `git commit -s`
6. Push and open pull request
7. Ensure CI checks pass
8. Await review and merge

All commits must be signed off with the Developer Certificate of Origin (DCO).
