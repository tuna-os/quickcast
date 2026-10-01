# Contributing to Quickcast

Thank you for your interest in contributing to Quickcast! This document outlines guidelines and instructions for submitting contributions to the project.

## Prerequisites & Environment Setup

Before building or testing Quickcast locally, ensure you have Flatpak installed on your host Linux system along with the required GNOME 50 SDK runtimes:

```sh
flatpak install --user flathub org.gnome.Sdk//50 org.freedesktop.Sdk.Extension.rust-stable//25.08
```

## Development Workflow

### Building Locally

Quickcast uses Meson and Rust inside the GNOME 50 Sdk Flatpak container. A local build helper script is provided:

```sh
./scripts/build-local.sh
```

This compiles the Rust application, compiles GSettings schemas, packages the app bundle into `Quickcast.flatpak`, and prepares local staging files in `_app/`.

To test installation of your local build bundle:

```sh
flatpak install --user --reinstall ./Quickcast.flatpak
```

### Running Tests

Execute the automated test suite locally:

```sh
./scripts/test-local.sh
```

The test runner launches a synthetic headless session using `Xvfb` and a isolated D-Bus session, running Meson tests and video verification (`scripts/verify-video.py`).

### Companion Shell Extension

If your changes involve the GNOME Shell companion extension for floating webcam composition:

```sh
./scripts/install-extension.sh
```

Note that GNOME Shell may require a session restart or logout/login for newly installed local extensions to take effect.

## Repository Layout

- `src/`: Core Rust application source code (GTK4 / libadwaita UI, GStreamer capture pipelines, clipboard handler).
- `extension/`: GNOME 50 Shell companion extension for window positioning and always-on-top behavior.
- `data/`: Desktop entry files, AppStream metadata, GSettings schemas, icons, and UI resource definitions.
- `po/`: Gettext translation files.
- `scripts/`: Development, build, verification, and installation scripts.
- `org.tunaos.Quickcast.json`: Flatpak Builder manifest specification.

## Submitting Pull Requests

1. **Sign-off Commits (DCO)**: All commits must include a Developer Certificate of Origin (DCO) sign-off line. Use `git commit -s` when committing your changes.
2. **Commit Messages**: Write concise, descriptive commit subjects using Conventional Commits formatting (e.g., `docs: update build steps`, `fix(ui): adjust webcam bubble drag bounds`).
3. **Pull Request Review**: Ensure local tests pass (`./scripts/test-local.sh`) before submitting your PR. Open PRs against the `main` branch.
