# AGENTS.md — agent guide for quickcast

Quickcast is a GNOME screen recorder: record screen with microphone audio and optional webcam, then paste as attachment. Shipped as a Flatpak with an optional GNOME Shell extension.

## Build and test commands

```bash
meson setup build                  # configure the build
meson compile -C build             # build the app
meson test -C build                # run all tests (~30s)
meson test -C build --timeout 60   # with longer timeout for slow CI
cargo clippy --all-targets -- -D warnings  # lint (all warnings denied)
cargo fmt --all --check            # format check
./scripts/build-local.sh           # build Flatpak locally
./scripts/test-local.sh            # run Flatpak tests
```

## Architecture rules

**Keep logic in libraries, not in the GTK window.**

- `src/recorder/` — capture logic, no GTK dependency, fully unit-testable
- `src/camera/` — webcam enumeration via V4L2, no GTK
- `src/encode/` — FFmpeg encoding wrapper, no GTK
- `src/ui/` — GTK dialogs and widgets, signal routing only
- `src/window.rs` — main application window, coordinates UI and backend
- `extension/` — GNOME Shell extension in JavaScript, separate from main app

Window and UI modules route between backend logic and GTK signals. They should not duplicate state or own encoding/capture logic. New capture or encoding features go in `src/recorder/`, `src/camera/`, or `src/encode/`, not in `src/window.rs`.

## Testing expectations

### Unit tests

New logic in `recorder/`, `camera/`, `encode/` modules → unit test in the same file or module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // test here
    }
}
```

Run with `meson test -C build`.

### Synthetic recording tests

Changes to video encoding, format, or frame properties → add/update a synthetic test in `tests/`:
- Encode a test recording with known parameters
- Verify codec, duration, frame count, resolution
- Check overlay rendering (if webcam compositing changed)

See `src/encode/tests.rs` for examples.

### UI tests

Changes to window, button, dialog, or clipboard behavior → add a GTK UI test:
- Test runs under Xvfb with a private D-Bus session
- Verify clipboard formats offered (GTK, URI, GNOME copied-file)
- Check bubble transparency and positioning

Tests use the GTK test harness in `src/ui/tests/`.

### Portal and hardware tests

Portal integration (screen capture permission), physical audio/video sync, real camera enumeration, actual Wayland/X11 behavior — these still require manual testing on a real GNOME session. Automated tests cannot exercise the portal or real hardware reliably.

## CI gates

Every PR checks:
1. **Build**: `meson compile -C build` (app binary)
2. **Tests**: `meson test -C build` (all unit/synthetic/UI tests, ~30s)
3. **Lint**: `cargo clippy --all-targets -- -D warnings` (all warnings denied)
4. **Format**: `cargo fmt --all --check` (code style)
5. **Flatpak**: `flatpak-builder --state-dir=.flatpak-builder build-dir org.tunaos.Quickcast.json` (release build)

A green build means the app compiles, tests pass, lint is clean, and the Flatpak builds. It does NOT mean:
- Actual screen capture works (that needs a real display and permission)
- Audio sync is correct (that needs real audio hardware)
- The portal allows it (that needs a real session)

Local manual testing on a real GNOME session is required before claiming a capture feature works end-to-end.

## Workflow

1. Branch from `main`: `git checkout -b fix/issue-name` or `git checkout -b feat/feature-name`
2. Make changes and commit with clear messages: `fix(encode): handle EOF gracefully`
3. Run checks locally (see commands above) before pushing
4. Push and open a PR. CI runs all gates.
5. All checks must be green before merge.

## Known issues and limitations

- **Xwayland fallback for region selection:** On Wayland, region selection may fall back to X11 via Xwayland if the portal implementation does not support the screenshot interface.
- **Flatpak device access:** The Flatpak requires `--device=all` and read-only `/run/udev` to enumerate V4L2 cameras. Without this, camera enumeration fails.
- **Audio sync on some hardware:** PulseAudio and Pipewire behave differently for loopback recording. Tests use synthetic audio; real hardware may drift.

## Performance considerations

- **Encoding:** Hardware acceleration (VAAPI, NVENC) is not implemented; software x264 encodes during recording.
- **Memory:** Each frame buffer is allocated on the stack during capture. Very high-resolution monitors may need buffer pooling (future work).
- **Threads:** Capture runs on the main GTK thread to simplify synchronization; encoding runs in a separate thread to avoid blocking the UI.

## Dependencies

| Dependency | Version | Purpose |
|---|---|---|
| GTK | ≥4.12 | Application framework |
| libadwaita | ≥1.5 | GNOME design system |
| FFmpeg | latest stable | Video encoding |
| Pipewire/PulseAudio | latest | Audio capture |
| Rust | 1.90+ | Language toolchain |

No Cargo.toml pinning other than MSRV (Minimum Supported Rust Version) of 1.70, set in `Cargo.toml`.

## Releasing

Tag releases as `v0.x.y`. The **Publish Quickcast Flatpak** workflow:
1. Builds the Flatpak
2. Pushes to GHCR (`ghcr.io/tuna-os/quickcast`)
3. Updates the TunaOS Flatpak index

CI on `main` builds and tests without publishing.
