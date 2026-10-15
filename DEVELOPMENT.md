# Development Setup Guide

This guide explains how to set up a development environment for Quickcast and build it from source.

## System Prerequisites

### Debian / Ubuntu

```sh
sudo apt install flatpak flatpak-builder python3 meson ninja-build pkg-config \
  libgtk-4-dev libadwaita-1-dev libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
  libpipewire-0.3-dev
```

### Fedora

```sh
sudo dnf install flatpak flatpak-builder python3 meson ninja-build pkg-config \
  gtk4-devel libadwaita-devel gstreamer1-devel gstreamer1-plugins-base-devel \
  pipewire-devel
```

### Arch

```sh
sudo pacman -S flatpak flatpak-builder python3 meson ninja pkg-config gtk4 \
  libadwaita gstreamer gst-plugins-base pipewire
```

### macOS

Quickcast targets GNOME on Linux. macOS builds are not supported.

## Rust Toolchain

Quickcast uses Rust. Install via [rustup](https://rustup.rs):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Build and Run Locally

### Quick Build (Direct Meson)

For faster iteration without Flatpak overhead:

```sh
# Configure and build
meson setup builddir
meson compile -C builddir

# Run the development build
./builddir/src/quickcast
```

Changes to source are picked up on the next build.

### Flatpak Build (Full Testing)

For testing the packaged Flatpak and CI behavior:

```sh
# Install the GNOME 50 SDK and Rust extension (one-time setup)
flatpak install --user flathub org.gnome.Sdk//50 \
  org.freedesktop.Sdk.Extension.rust-stable//25.08

# Build the Flatpak bundle
./scripts/build-local.sh

# Install and run the bundle
flatpak install --user --reinstall ./Quickcast.flatpak
flatpak run --user org.tunaos.Quickcast
```

The Flatpak build is slower but tests the full distribution setup, portal permissions, and sandbox behavior.

## Running Tests

### Unit and UI Tests

```sh
./scripts/test-local.sh
```

This runs:
- **Meson tests**: Unit tests and GTK integration tests
- **Synthetic recording test**: Records a synthetic screen with webcam overlay and audio
- **Clipboard test**: Verifies file transfer in all three clipboard formats

### Synthetic Recording Verification

The test creates a synthetic MP4 recording. To verify it manually:

```sh
python3 scripts/verify-video.py /path/to/video.mp4
```

This checks:
- Video codec (H.264)
- Audio codec (AAC)
- Duration
- Circular webcam overlay with transparent corners

### Individual Test Suites

Run Meson tests only:

```sh
meson test -C builddir
```

Run without the synthetic recording and clipboard tests:

```sh
meson test -C builddir --suite quick
```

## Code Organization

| Directory | Purpose |
|-----------|---------|
| `src/` | Rust source code |
| `src/ui/` | GTK user interface |
| `src/recording/` | GStreamer recording pipeline |
| `src/portal/` | GNOME desktop portal integration |
| `src/gstreamer/` | GStreamer pipeline components |
| `data/` | Desktop entry, schemas, UI definitions |
| `extension/` | GNOME Shell companion extension |
| `scripts/` | Build, test, and deployment helpers |
| `po/` | Translations |

See [AGENTS.md](AGENTS.md) for architecture rules and code separation.

## Debugging

### View Logs

The Flatpak logs to systemd journal when run as a service. For interactive runs:

```sh
G_MESSAGES_DEBUG=all flatpak run --user org.tunaos.Quickcast
```

### Debug with GDB (Flatpak)

```sh
flatpak run --user --devel --command=gdb org.tunaos.Quickcast
```

### Screen Recording Without UI

Test the recording pipeline directly:

```sh
# Direct build only
./builddir/src/quickcast --help
```

## Development Workflow

### Making Changes

1. Edit source files in `src/`.
2. Rebuild: `meson compile -C builddir`
3. Test: `./builddir/src/quickcast`

### Before Submitting a PR

1. Run tests: `./scripts/test-local.sh`
2. Verify the Flatpak build: `./scripts/build-local.sh`
3. Test on a real GNOME desktop if possible (portal permissions, pasting behavior)

### Format and Lint

Rust code is checked by:
- `cargo fmt` (formatting — run automatically in CI)
- `cargo clippy` (linting)

Run before committing:

```sh
cargo fmt --manifest-path=src/Cargo.toml
cargo clippy --manifest-path=src/Cargo.toml
```

## Common Issues

### `flatpak install` fails: runtime not found

The build script requires the GNOME 50 runtime:

```sh
flatpak install --user flathub org.gnome.Sdk//50
```

### Portal permissions dialog doesn't appear

When running directly (not via Flatpak), portal selection may not work. Use the Flatpak build for testing portal behavior:

```sh
./scripts/build-local.sh
flatpak run --user org.tunaos.Quickcast
```

### Webcam not detected

The app enumerates cameras when the window opens. If you attach a camera after starting Quickcast:

1. Quit the app (Ctrl+Q)
2. Restart it

For the Flatpak, verify it has device access:

```sh
flatpak run --user --device=all org.tunaos.Quickcast
```

### Audio sync issues in recordings

Audio/video synchronization depends on hardware capabilities and GStreamer plugins. If you encounter sync problems:

1. Verify both audio inputs work: microphone + desktop audio
2. Test with different framerate settings
3. Report the issue with: GNOME version, hardware, and a sample recording

## Publishing and Release

See [ROADMAP.md](ROADMAP.md) for publishing workflow and credential requirements.

## Getting Help

- [CONTRIBUTING.md](CONTRIBUTING.md): How to contribute
- [AGENTS.md](AGENTS.md): Architecture and CI guidelines
- [ROADMAP.md](ROADMAP.md): Project priorities and status
- [GitHub Issues](https://github.com/tuna-os/quickcast/issues): Report bugs or discuss features
