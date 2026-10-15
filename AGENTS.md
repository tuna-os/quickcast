# Architecture and CI Guidelines

This document explains Quickcast's code organization, architecture rules, and CI pipeline for contributors and automated agents.

## Code Organization

| Component | Location | Purpose |
|-----------|----------|---------|
| Main application | `src/main.rs` | Application entry point and initialization |
| GTK UI | `src/ui/` | GNOME user interface (windows, widgets) |
| Recording pipeline | `src/recording/` | GStreamer recording and encoding logic |
| GNOME portals | `src/portal/` | Desktop portal integration (screen capture, permissions) |
| GStreamer components | `src/gstreamer/` | Custom GStreamer pipeline elements |
| i18n and schemas | `data/` | Desktop entry, GSettings schemas, UI definitions |
| Shell extension | `extension/` | GNOME Shell 50 companion extension |
| Build scripts | `scripts/` | Build, test, and deployment helpers |
| Translations | `po/` | Locale files |

## Architecture Rules

### Separation of Concerns

- **UI (`src/ui/`)**: GTK widgets and window management. No business logic.
- **Recording (`src/recording/`)**: GStreamer pipeline orchestration. No UI dependencies.
- **Portals (`src/portal/`)**: GNOME desktop portal abstractions. Called by recording logic.
- **Main (`src/main.rs`)**: Application lifecycle and wiring of subsystems.

### Known Architectural Debt

These are recorded in [ROADMAP.md](ROADMAP.md) as technical debt items:

| Issue | Impact | Effort |
|-------|--------|--------|
| Recording lifecycle couples portal state, GStreamer, and file output | [#29](https://github.com/tuna-os/quickcast/issues/29) | Large |
| Window code couples presentation, state, and recording orchestration | [#20](https://github.com/tuna-os/quickcast/issues/20) | Large |
| GStreamer input depends on UI-layer selection type | [#21](https://github.com/tuna-os/quickcast/issues/21) | Medium |
| Portal abstraction exposes D-Bus types to callers | [#22](https://github.com/tuna-os/quickcast/issues/22) | Medium |

When making changes, try to move toward the separation of concerns described above rather than deepening these couplings.

## Build System

Quickcast uses **Meson** as its build system:

```sh
meson setup builddir              # Configure the build directory
meson compile -C builddir         # Build
meson test -C builddir            # Run tests
meson install -C builddir         # Install locally
```

Key build configuration is in `meson.build` at the root and in each `src/` subdirectory.

### Rust Components

The Rust source is in `src/Cargo.toml`. Meson orchestrates the Rust build:

```sh
cargo build --manifest-path=src/Cargo.toml  # Direct Rust build
cargo fmt --manifest-path=src/Cargo.toml    # Format
cargo clippy --manifest-path=src/Cargo.toml # Lint
```

## Continuous Integration

### Workflow Files

Located in `.github/workflows/`:

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `build.yml` | PR, push to main | Build, lint, test |
| `publish.yml` | Release tag (v*), dispatch | Build Flatpak, publish to GHCR and central index |

**Note**: This agent cannot modify workflow files due to GitHub App permissions. See the [CONTRIBUTING.md](CONTRIBUTING.md) for how to report CI issues.

### CI Gates

Every PR must pass:

1. **Build**: `meson setup builddir && meson compile -C builddir`
2. **Unit tests**: `meson test -C builddir --suite quick` (logic tests only)
3. **Synthetic recording test**: Full record + encode + file verification
4. **Clipboard test**: GTK clipboard formats and file transfer
5. **Lint**: Rust code style via `cargo fmt` and `cargo clippy`

See [DEVELOPMENT.md](DEVELOPMENT.md) for how to run these locally.

### Test Suites

**Quick suite** (logic only):
```sh
meson test -C builddir --suite quick
```

**Full suite** (includes synthetic recording):
```sh
meson test -C builddir
```

## Commit Guidelines

- Use imperative mood: "Add feature" not "Added feature"
- Keep the first line under 50 characters
- Reference issues: "Fixes #123" or "Refs #123"
- Sign off with DCO: `git commit -s`

Example:
```
Fix webcam positioning in window recordings

The bubble now correctly follows relative position changes
when the window moves between monitors.

Fixes #15
```

## Code Style

### Rust

- Run `cargo fmt --manifest-path=src/Cargo.toml` before committing
- Linting: `cargo clippy --manifest-path=src/Cargo.toml`
- Follow standard Rust idioms and naming conventions

### GTK/UI

- Use GTK 4 and libadwaita patterns
- Follow GNOME HIG for UI consistency
- Minimize code in UI layer; push logic to recording module

## Dependencies

### Runtime

- GNOME 50 (libgtk-4, libadwaita)
- GStreamer 1.0 (core, plugins-base, plugins-good)
- PipeWire 0.3 (audio capture)
- Portal services (xdg-desktop-portal-gnome)

### Build

- Rust (1.70+)
- Meson 1.0+
- Ninja
- pkg-config

### Test

- Python 3 (for verification scripts)
- FFmpeg (for video decoding in tests)

See [DEVELOPMENT.md](DEVELOPMENT.md) for installation instructions per distribution.

## Release Process

1. Tag: `git tag v0.1.0` and push
2. Dispatch **Publish Quickcast Flatpak** workflow (or it runs automatically)
3. Workflow builds Flatpak, publishes to GHCR and central TunaOS index
4. Release bundle is created on GitHub releases

See [ROADMAP.md](ROADMAP.md) and [README.md](README.md) for details on credentials and fallback publishing.

## Testing Checklist

Before opening a PR:

- [ ] Code compiles: `meson compile -C builddir`
- [ ] Tests pass: `./scripts/test-local.sh`
- [ ] Rust formatted: `cargo fmt --manifest-path=src/Cargo.toml`
- [ ] No clippy warnings: `cargo clippy --manifest-path=src/Cargo.toml`
- [ ] Commit is signed: `git commit -s`

## Getting Help

- [DEVELOPING.md](DEVELOPMENT.md): Setup and build instructions
- [CONTRIBUTING.md](CONTRIBUTING.md): How to contribute
- [ROADMAP.md](ROADMAP.md): Project priorities and technical debt
- [GitHub Issues](https://github.com/tuna-os/quickcast/issues): Report bugs or discuss features
