# Contributing to Quickcast

Thanks for helping. Quickcast is GPL-3.0-or-later (see `COPYING`).

The shared tuna-os process (pull requests, security reporting) is described in
the [organization contributor guide](https://github.com/tuna-os/.github/blob/main/CONTRIBUTING.md).
This file covers what is specific to this repository.

## Getting started

1. Pick or open an issue. Comment on the one you want to own.
   Issue templates exist for bugs and feature requests.
2. Fork the repository and branch from `main`.
3. Set up and build following [README.md](README.md#build-and-test) and
   [AGENTS.md](AGENTS.md).
4. Read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) before larger changes.

## Before opening a pull request

- For changes touching the Flatpak or recording path, run
  `./scripts/build-local.sh` and `./scripts/test-local.sh`.
- Follow the lint rules in [AGENTS.md](AGENTS.md); CI fails on clippy findings
  covered by the Meson tests.
- Keep pull requests focused. Reference the issue (for example `Closes #123`).
- Use a short Conventional Commit style title such as `fix: ...` or `docs: ...`.
- Sign off your commits (`git commit -s`). A formal DCO policy is not
  documented yet; check the organization guide.
- Do not include credentials, API keys or screencast restore tokens in issues,
  logs or pull requests.

## Translations

Translations live in `po/` and have been managed through Weblate. A
contributor workflow for them is not documented yet.

## Security

Do not report vulnerabilities in public issues. See [SECURITY.md](SECURITY.md).

## Roadmap

Planned work is tracked in [ROADMAP.md](ROADMAP.md).
