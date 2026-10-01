# Contributing to Quickcast

Thanks for your interest in contributing to Quickcast! This guide covers setting up your development environment, building, testing, and submitting changes.

## Getting Started

### Prerequisites

Quickcast is a Rust GTK4 application. You'll need:

- **Rust**: Install from [rustup.rs](https://rustup.rs/). Quickcast targets recent stable Rust (check Cargo.toml for MSRV).
- **GTK 4**: System libraries and development headers
  - **Linux (Debian/Ubuntu)**: `sudo apt install libgtk-4-dev libgst-dev libadwaita-1-dev libxdp-dev`
  - **Fedora**: `sudo dnf install gtk4-devel gstreamer1-devel libadwaita-devel`
- **GStreamer**: Media framework for audio/video capture
  - **Linux (Debian/Ubuntu)**: `sudo apt install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev`
  - **Fedora**: `sudo dnf install gstreamer1-devel gstreamer1-plugins-base-devel`
- **Flatpak SDK** (optional): For testing the Flatpak build
  - `flatpak install flathub org.gnome.Platform//50 org.gnome.Sdk//50`

### Clone and Build

```bash
git clone https://github.com/tuna-os/quickcast.git
cd quickcast
cargo build
cargo test
```

## Development Workflow

### Building

Build the native binary in debug mode:

```bash
cargo build
```

Build in release mode (optimized):

```bash
cargo build --release
```

### Running Locally

Run the development binary:

```bash
cargo run
```

Enable debug logging:

```bash
RUST_LOG=quickcast=debug cargo run
```

### Testing

Run the test suite:

```bash
cargo test
```

Run tests with output (useful for debugging):

```bash
cargo test -- --nocapture
```

Test a specific module:

```bash
cargo test recording::
```

### Linting and Formatting

Format code according to Rust conventions:

```bash
cargo fmt
```

Check for style issues and common mistakes:

```bash
cargo clippy
```

Fix clippy warnings automatically (where possible):

```bash
cargo clippy --fix
```

Run both fmt and clippy:

```bash
cargo fmt && cargo clippy
```

### Building for Flatpak

Build the Flatpak bundle locally:

```bash
flatpak-builder --user --install build org.tunaos.Quickcast.json
flatpak run org.tunaos.Quickcast
```

Test with GNOME 50 runtime:

```bash
flatpak-builder --user build org.tunaos.Quickcast.json --require-changes
```

### Desktop File and Metadata

After changes to the app's name, description, or capabilities:
1. Update `org.tunaos.Quickcast.desktop`
2. Update `org.tunaos.Quickcast.metainfo.xml` (AppStream metadata)
3. Test Flatpak rebuild to ensure metadata validation passes

## Code Standards

- **Rust edition**: Check Cargo.toml for the target edition (likely 2021).
- **Formatting**: All code must pass `cargo fmt`. Run this before submitting.
- **Linting**: All code must pass `cargo clippy` with no warnings.
- **Testing**: Add tests for new recording modes, audio handling, or capture logic.
- **Comments**: Document non-obvious logic, especially in media capture and GStreamer code.
- **Safety**: GTK and GStreamer bindings use unsafe Rust; review safety comments and bounds checking.

## Architecture

Quickcast has several key modules:

1. **Recording** (recording/): Manages video capture, encoding, window/region selection
2. **Audio**: Microphone and desktop audio capture via GStreamer
3. **Video**: Screen capture, region selection, encoding (MP4, WebM)
4. **Webcam**: Floating bubble overlay with webcam feed
5. **UI** (ui/): GTK4 application window, controls, preferences
6. **Clipboard**: File copying to clipboard for paste-as-attachment

When modifying capture logic:
- Test with different screen resolutions and refresh rates
- Verify audio sync with video
- Check CPU/GPU usage with top/nvtop
- Test region selection at various screen scales (100%, 125%, 150%)

## Upstream: Kooha

Quickcast is a fork of [Kooha](https://github.com/SeaDve/Kooha). When filing issues or PRs:

1. Check if the issue exists in upstream Kooha first
2. If your change improves capture or stability, consider contributing upstream
3. Reference Kooha commits in your PR description if relevant
4. For Kooha-inherited code, respect the original author attribution

## Submitting Changes

### Before You Push

1. Run `cargo fmt` to format your code.
2. Run `cargo clippy` and fix all warnings.
3. Run `cargo test` to verify tests pass.
4. Test with `cargo run` to verify the UI still works.
5. If you added a feature, test it with Flatpak: `flatpak-builder --user --install build org.tunaos.Quickcast.json`
6. Include a clear commit message explaining the "why" behind your change.
7. Sign your commits with DCO: `git commit -s`.

### Creating a Pull Request

1. Push your branch: `git push -u origin guide/your-branch-name`
2. Open a PR on GitHub. Link any related issues.
3. The CI suite will run automatically. If any check fails, review the details and fix the issue.

### PR Guidelines

- **Scope**: Keep PRs focused. One feature or fix per PR when possible.
- **Commits**: Use clear commit messages. If your PR fixes an issue, mention it: `Fixes #123`.
- **Tests**: Add tests for new recording modes, audio handling, or encoding logic.
- **Docs**: Update this CONTRIBUTING.md if adding new prerequisites or build steps.
- **Screenshots**: For UI changes, include before/after screenshots in the PR description.
- **Performance**: For capture logic changes, include CPU/memory impact notes.

## Known Constraints

- **GNOME 50**: Quickcast targets GNOME 50 Flatpak runtime. Code must work with GTK4 from that release.
- **Kooha compatibility**: Inherited code patterns may differ from TunaOS style. Refactor incrementally, not all at once.
- **GStreamer**: Audio and video capture depend on GStreamer availability and codec support. Test on systems with different codec packs.
- **Wayland**: Primary development target; X11 support is secondary (inherited from Kooha).

## Getting Help

- **Issues**: Use GitHub issues to report bugs or suggest improvements.
- **PR comments**: Ask questions about changes directly on PRs.
- **Kooha**: For capture or GStreamer questions, check Kooha's docs and upstream issues first.
- **GNOME**: For GTK4 questions, consult GNOME API documentation and examples.

## Testing Checklist

Before submitting a capture-related PR:

- [ ] Test recording on a 1080p monitor at 60 FPS
- [ ] Test microphone audio (check sync with video)
- [ ] Test desktop audio capture (if available)
- [ ] Test webcam overlay (position, size, visibility)
- [ ] Test region selection on scaled displays (125%, 150%)
- [ ] Test output formats (MP4, WebM if supported)
- [ ] Verify CPU/GPU usage is reasonable
- [ ] Test Flatpak build and runtime

## Recognition

All contributors are credited in the commit history. Commits must be signed with DCO (`git commit -s`) per project policy.
