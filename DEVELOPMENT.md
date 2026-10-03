# Development Guide

How to set up, build, test, and develop Quickcast locally.

## System requirements

- Rust 1.70 or newer (provided by rustup or your distribution)
- Meson 1.4.0 or newer
- GTK 4.12 or newer with development headers
- GLib 2.78 or newer with development headers
- libadwaita 1.5 or newer
- FFmpeg with development libraries for encoding
- Pipewire or PulseAudio for audio capture
- Xwayland (if using Wayland) for region selection and capture

### Install dependencies

**Fedora:**
```bash
sudo dnf install gtk4-devel libadwaita-devel libglib2-devel pipewire-devel \
  ffmpeg-devel meson cargo pulseaudio-devel
```

**Debian/Ubuntu:**
```bash
sudo apt-get install libgtk-4-dev libadwaita-1-dev libglib2.0-dev \
  libpipewire-0.3-dev libffmpeg-ocaml-dev meson cargo \
  libpulse-dev libcap-dev
```

**Arch:**
```bash
sudo pacman -S gtk4 libadwaita glib2 pipewire ffmpeg meson cargo
```

## Build from source

Initialize the build directory and compile:

```bash
meson setup build
meson compile -C build
```

On the first build, Meson will download Rust dependencies and compile them, which may take a few minutes.

## Run the application

Run directly from the build directory without installing:

```bash
./build/quickcast
```

Or install and run from your applications menu:

```bash
meson install -C build
quickcast
```

To uninstall:
```bash
meson uninstall -C build
```

## Running tests

Run the full test suite:

```bash
meson test -C build
```

Run a specific test:

```bash
meson test -C build test_name
```

The test suite includes:
- Unit tests for encode, capture, and utility modules
- GTK UI tests for clipboard and dialog behavior
- Synthetic recording tests that verify video output

## Development workflow

1. **Make your changes** in `src/` or `extension/`
2. **Recompile** with `meson compile -C build`
3. **Test** with `meson test -C build` and manual testing
4. **Run clippy** for style and common mistakes:
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```
5. **Format code** with rustfmt if you made changes:
   ```bash
   cargo fmt --all
   ```
6. **Commit** with clear messages and push to your fork

## Code organization

| Path | Purpose |
|---|---|
| `src/main.rs` | Application entry point and window setup |
| `src/window.rs` | Main GTK window and UI logic |
| `src/recorder/` | Screen, audio, and video capture backend |
| `src/camera/` | Webcam enumeration and V4L2 capture |
| `src/encode/` | FFmpeg integration for video encoding |
| `src/ui/` | GTK dialogs and custom widgets |
| `extension/` | GNOME Shell extension for floating bubble |
| `data/` | Application metadata, schemas, icons, UI files |

## GSettings schema

The application uses GSettings to persist user preferences. The schema is defined in `data/org.tunaos.Quickcast.gschema.xml` and is automatically compiled during the build.

To view or edit GSettings at runtime:

```bash
dconf dump /org/tunaos/Quickcast/
dconf write /org/tunaos/Quickcast/key 'value'
```

## Debugging

Enable verbose output by setting environment variables:

```bash
G_MESSAGES_DEBUG=all ./build/quickcast
```

For Rust backtrace on panic:

```bash
RUST_BACKTRACE=1 ./build/quickcast
```

Use GDB or lldb for stepping through code:

```bash
gdb --args ./build/quickcast
```

## Building for Flatpak

The project includes a Flatpak manifest at `org.tunaos.Quickcast.json`. To build and test the Flatpak:

```bash
flatpak install --user flathub org.gnome.Sdk//50
flatpak-builder --user --install build-dir org.tunaos.Quickcast.json
flatpak run --user org.tunaos.Quickcast
```

This builds in an isolated sandbox that matches the release environment, which is useful for testing portal integration and hardware access.

## Building the Shell extension

The GNOME Shell extension is in `extension/` and is built and installed by the main Meson build. To install it to GNOME manually:

```bash
./scripts/install-extension.sh
```

After installation, you may need to restart GNOME Shell (Alt+F2, type `r`, press Enter) or log out and back in for it to be discovered.

## Next steps

- Read [CONTRIBUTING.md](CONTRIBUTING.md) for code style and submission guidelines.
- Check open issues labeled `good-first-issue` for starting points.
- Look at [README.md](README.md) for user-facing feature documentation.
