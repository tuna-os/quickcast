# Contributing to Quickcast

Thanks for your interest in contributing to Quickcast. This project is a GNOME screen recorder for quick explanations: record your screen with microphone audio and an optional webcam, then paste the video as an attachment.

## How to contribute

Quickcast welcomes bug reports, feature requests, and pull requests. Please file issues in this repository with clear descriptions of what you found or what you'd like to see.

## Reporting issues

When filing a bug report, include:
- The version of Quickcast you're using
- Your GNOME and GTK version (`gnome-shell --version`, `gtk-launch --version`)
- Steps to reproduce the issue
- What you expected to happen vs. what actually happened
- Relevant hardware details (monitor resolution, if a webcam is involved, audio setup)

## Development setup

See [DEVELOPMENT.md](DEVELOPMENT.md) for instructions on setting up a development environment, building locally, and running tests.

## Submitting changes

1. Fork the repository and create a branch from `main` for your changes.
2. Write commit messages that are clear and concise. Follow the existing style.
3. Build and test locally before pushing (see [DEVELOPMENT.md](DEVELOPMENT.md)):
   ```bash
   meson setup build
   meson compile -C build
   meson test -C build
   ```
4. Push your branch and open a pull request. Include a clear description of what your changes do.
5. CI will run the build, tests, and linting checks. All checks must pass before merge.

## Code style and conventions

- Follow the existing code style in the repository.
- The project uses Rust for the main application and JavaScript/TypeScript for the GNOME Shell extension.
- Use `cargo clippy --all-targets` to check for common mistakes and style issues.
- Comments should explain the "why", not just the "what".

## Architecture

The project is organized as follows:

| Directory | Purpose |
|---|---|
| `src/` | Main Quickcast application code in Rust |
| `extension/` | GNOME Shell extension for floating bubble integration |
| `data/` | Application metadata, schemas, icons, and UI definitions |
| `scripts/` | Helper scripts for building, testing, and installation |
| `build-aux/` | Build-time helpers and dependencies |

Key modules in `src/`:
- `recorder/` — Screen, audio, and video capture
- `camera/` — Webcam enumeration and V4L2 access
- `encode/` — Video encoding with FFmpeg
- `ui/` — GTK application windows and dialogs

For more details, see [DEVELOPMENT.md](DEVELOPMENT.md).

## Testing

The project includes:
- **Unit tests** for encode, capture, and utility modules
- **UI tests** for clipboard and dialog behavior using GTK
- **Synthetic recording tests** that verify codec output and overlay rendering

Run all tests with:
```bash
meson test -C build
```

Real hardware testing (physical audio/video sync, portal integration with actual apps) still requires manual verification on a real GNOME session.

## License

Quickcast is GPL-3.0-or-later. By contributing, you agree that your contributions are licensed under the same license. See [COPYING](COPYING) for the full license text.

## Getting help

- Check the [README.md](README.md) for usage instructions and troubleshooting.
- Look at existing issues and pull requests to see if your question has been asked before.
- Feel free to ask questions in issues — maintainers and contributors are happy to help.
