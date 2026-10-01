# AGENTS.md — agent guide for tuna-os/quickcast

**Quickcast** is a GNOME screen recorder for quick explanations: record screen with microphone and optional webcam overlay, then paste finished video as attachment.

Human docs: [`README.md`](README.md) (overview, installation, usage),
[`CONTRIBUTING.md`](CONTRIBUTING.md) (development setup and PR process).

## Build and Test

```bash
cargo build                        # build native binary
cargo test                         # run tests
cargo run                          # run locally
RUST_LOG=quickcast=debug cargo run # with debug logging
flatpak-builder --user --install build org.tunaos.Quickcast.json  # Flatpak build
```

## Key Facts

- **Language**: Rust (recent stable)
- **GUI**: GTK4 with libadwaita
- **Target**: GNOME 50 Flatpak runtime
- **Origin**: Fork of Kooha; adds webcam overlay, file copying, MP4 preset, TunaOS packaging
- **Media**: GStreamer for audio/video capture and encoding

## Repository Structure

- **src/recording/**: Video capture, encoding, region selection
- **src/audio/**: Microphone and desktop audio via GStreamer
- **src/video/**: Screen capture, encoding logic
- **src/ui/**: GTK4 application, controls, preferences
- **src/clipboard/**: File copying for paste-as-attachment
- **org.tunaos.Quickcast.json**: Flatpak manifest
- **Cargo.toml**: Rust dependencies (gtk4, gstreamer, etc.)

## Build Prerequisites

**System libraries** (on Debian/Ubuntu):
```bash
sudo apt install libgtk-4-dev libgst-dev libadwaita-1-dev libxdp-dev \
  libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev
```

**Flatpak SDK** (optional):
```bash
flatpak install flathub org.gnome.Platform//50 org.gnome.Sdk//50
```

## Architecture

Key components:

1. **Recording**: Manages capture session (window/region/monitor selection, start/stop)
2. **Audio**: Microphone and desktop audio capture (GStreamer audio sources)
3. **Video**: Screen capture and encoding (VP9/VP8 for WebM, H.264 for MP4)
4. **Webcam**: Floating overlay with live camera feed (optional, draggable)
5. **Output**: MP4 (default), WebM (optional), copied to clipboard

State flow: Select region → Configure audio/webcam → Record → Stop → Save file + copy to clipboard

## Testing Approach

- **Unit tests**: Cargo tests for recording logic, audio handling, output validation
- **Integration tests**: Full recording flow with mock GStreamer (if available)
- **Manual**: Test capture on different monitors, scales (100%/125%/150%), refresh rates
- **Flatpak**: Build and run via flatpak-builder to verify runtime environment

## Upstream: Kooha

Quickcast inherits architecture and components from Kooha (SeaDve/Kooha on GitHub):
- Screen capture and region selection
- GStreamer audio/video pipeline
- GTK4 UI foundation
- Flatpak packaging

Refactors to Kooha-inherited code should preserve original author attribution.

## Deployment

Quickcast is distributed as Flatpak via TunaOS remote:

```bash
flatpak remote-add --if-not-exists tuna-os https://tunaos.org/flatpak/tuna-os.flatpakrepo
flatpak install tuna-os org.tunaos.Quickcast
```

Desktop file and AppStream metadata:
- `org.tunaos.Quickcast.desktop`: Standard desktop entry
- `org.tunaos.Quickcast.metainfo.xml`: AppStream metadata for app stores

## Known Constraints

- **GNOME 50 only**: Uses APIs from GNOME 50 Flatpak runtime; older GNOME versions not supported
- **Codecs**: Availability depends on GStreamer plugins installed on the system
- **Wayland-primary**: Primary development target (X11 inherited from Kooha)

## DCO and Attribution

All commits must be signed with DCO: `git commit -s`. No special PR attribution needed beyond standard Git author.
