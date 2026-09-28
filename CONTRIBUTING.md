# Contributing to Quickcast

Thanks for your interest. Quickcast is a GNOME screen recorder for quick explanations — record your screen with microphone audio and an optional webcam, then paste as video.

This is a GPL-3.0-or-later fork of [Kooha](https://github.com/SeaDve/Kooha), extended with webcam composition, automatic file copying, and TunaOS Flatpak packaging.

## Before You Start

Read [README.md](README.md) for an overview. See [tuna-os/.github/CODE_OF_CONDUCT.md](https://github.com/tuna-os/.github/blob/main/CODE_OF_CONDUCT.md) for community guidelines.

## Set Up

You need:
- Flatpak with GNOME 50 SDK and Rust extension
- Python (for test verification)
- Xvfb (for headless testing)

```bash
flatpak install --user flathub org.gnome.Sdk//50 org.freedesktop.Sdk.Extension.rust-stable//25.08
git clone https://github.com/tuna-os/quickcast.git
cd quickcast
```

## Build and Test

```bash
# Local build with Flatpak
./scripts/build-local.sh

# Install the resulting Flatpak
flatpak install --user --reinstall ./Quickcast.flatpak

# Run tests (requires Xvfb and D-Bus)
./scripts/test-local.sh
```

The build produces `Quickcast.flatpak`. CI runs Meson tests including synthetic MP4 recording validation (`scripts/verify-video.py`) and GTK clipboard/UI checks.

## Code Conventions

- Follow Rust style via `cargo fmt`
- Code is in `src/` — UI, capture, audio
- Tests use Xvfb and a private D-Bus session
- GObject Introspection for screen capture via the desktop portal
- V4L2 for camera enumeration and capture

## Pull Requests

1. Open or find an issue for your change
2. Branch from `main`: `git checkout -b feature/your-feature`
3. Write commit messages: `feat: …`, `fix: …`, `refactor: …`, `docs: …`
4. Run tests locally: `./scripts/test-local.sh`
5. Push and open a PR against `main`

Interactive hardware testing (GNOME portal selection, audio/video sync, pasting into apps) still requires human validation.

## Publishing

Releases are tagged `v0.1.0`, etc. The **Publish Quickcast Flatpak** workflow publishes to GHCR and updates the TunaOS Flatpak index. See README Publishing section for details on fallback publishing when the index is unavailable.

---

By contributing, you agree your contributions are licensed under GPL-3.0-or-later.
