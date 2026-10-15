# Contributing to Quickcast

Thank you for your interest in contributing to Quickcast! This document explains how to report issues, suggest features, and submit code changes.

## Reporting Issues

Before opening an issue, please:

1. Check the [existing issues](https://github.com/tuna-os/quickcast/issues) to avoid duplicates.
2. Include the full output of your system information: GNOME version, Flatpak version (if applicable), and your distribution.
3. For bugs related to recording, include:
   - Your screen setup (monitor count and resolution)
   - Whether you're using a webcam, and its model if available
   - Your audio input/output devices
   - Steps to reproduce the issue
   - Expected vs. actual behavior

## Suggesting Features

Feature requests are welcome. When suggesting a feature:

1. Describe the use case and why it would be valuable.
2. Explain how it fits Quickcast's mission: making short GNOME screen recordings easy to capture and paste locally.
3. Note any privacy or architectural concerns.

## Code Contributions

### Getting Started

1. [Fork](https://github.com/tuna-os/quickcast/fork) the repository.
2. Clone your fork: `git clone https://github.com/your-username/quickcast.git`
3. Read [DEVELOPMENT.md](DEVELOPMENT.md) for setup instructions.

### Development Process

1. Create a branch for your change: `git checkout -b fix/issue-name` or `git checkout -b feature/feature-name`
2. Make your changes and test them locally (see [DEVELOPMENT.md](DEVELOPMENT.md) for test commands).
3. Commit with clear, descriptive messages: `git commit -s`
4. Push to your fork and open a pull request.

### Commit Messages

- Use imperative mood: "Add feature" not "Added feature"
- Keep the first line under 50 characters
- Reference related issues: "Fixes #123" or "Refs #123"
- Sign off your commits with `-s` (DCO — Developer Certificate of Origin)

Example:
```
Fix webcam positioning in window-only recordings

The bubble now correctly follows relative position changes
when the window moves between monitors.

Fixes #15
```

### Pull Request Process

- Keep pull requests focused on a single issue or feature.
- Reference the GitHub issue your PR addresses.
- Ensure all CI checks pass (build, tests, and lint).
- Keep the PR description clear and concise.
- Larger architectural changes should be discussed in an issue first.

### Code Style

- Rust code follows standard Rust conventions. Run `cargo fmt` before committing.
- GTK/UI code follows GNOME conventions.
- The [Architecture Rules](AGENTS.md) document the codebase structure.

### Testing

Before submitting a PR:

```sh
./scripts/build-local.sh    # Build the Flatpak
./scripts/test-local.sh     # Run all tests
```

Tests include:
- **Unit tests**: Meson test suite
- **UI tests**: GTK clipboard and bubble transparency
- **Synthetic recording**: Screen, webcam, and audio encoding verification
- **Codec validation**: Checks video codecs, duration, and visual content

See [DEVELOPMENT.md](DEVELOPMENT.md) for more details on running individual test suites.

### Real Hardware Testing

Some aspects require interactive testing:

- GNOME portal screen/webcam/audio selection UI
- Audio and video synchronization
- Pasting into specific applications (browsers, chat apps, etc.)

If you're making changes to these areas, please note it in your PR and test on real hardware if possible.

## License

Quickcast is GPL-3.0-or-later. By contributing, you agree that your contributions will be licensed under the same license. The DCO sign-off confirms this.

## Questions?

Open an issue or check [DEVELOPMENT.md](DEVELOPMENT.md) for setup help.

## Code of Conduct

Be respectful, inclusive, and constructive in all interactions. This project is part of the TunaOS community and follows the same standards for respectful collaboration.
