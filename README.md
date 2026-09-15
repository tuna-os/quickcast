# Quickcast

A small GNOME screen recorder for quick explanations: record your screen with
microphone audio and an optional webcam in the corner, then paste the finished
video as an attachment.

Quickcast is a GPL-3.0-or-later fork of [Kooha](https://github.com/SeaDve/Kooha).
Kooha's authors and contributors built the capture, audio, region selection and
GNOME application foundation. This fork adds webcam composition, automatic file
copying, a compact MP4 preset and TunaOS packaging.

## Install

```sh
flatpak remote-add --user --if-not-exists tuna-os https://tunaos.org/flatpak/tuna-os.flatpakrepo
flatpak install --user tuna-os org.tunaos.Quickcast
flatpak run --user org.tunaos.Quickcast
```

The initial release targets x86_64 and the GNOME 50 Flatpak runtime.
Alternatively, install the `.flatpak` bundle from this repository's releases.

## Record and paste

1. Choose a monitor/window or a screen region.
2. Toggle microphone and desktop audio. Microphone is enabled by default;
   desktop audio is optional.
3. Enable **Include webcam in recording**, select a camera and corner, and use
   **Preview camera** to check the picture. Close the preview to release the
   camera before recording.
4. Press **Record**. Press **Stop** when finished.
5. The completion window appears and copies the video file. Paste in an app
   that accepts file attachments. **Copy video** copies the latest recording again;
   **Open video** plays it.

Files remain in `~/Videos/Quickcast` unless you change the destination in
Preferences. No upload, account or cloud storage is involved. Closing the window
keeps the app running to serve the clipboard; choose **Quit** or Ctrl+Q to exit.

### Global shortcut

In GNOME Settings → Keyboard → Custom Shortcuts, use:

```sh
gapplication action org.tunaos.Quickcast toggle-record
```

The same command starts and stops recording. `scripts/install-shortcut.py` installs
Super+Shift+R if it is free, preserving your existing custom shortcuts.

## Encoding and clipboard behavior

- Default: H.264 + AAC in MP4, 30 fps, software x264 quality 24 / veryfast.
- The output fits within 1920×1080 without upscaling. Webcam composition happens
  after region cropping, so it is included in window and region recordings.
- The first version has a rectangular 4:3 overlay, four corner positions, and
  camera preview. A circular bubble, live overlay dragging and automatic
  hardware encoder selection are not implemented.
- Video is encoded during recording. Copying occurs only after successful
  end-of-stream finalization, never after cancellation or an encoding error.
- GTK file-list, URI-list and GNOME copied-file clipboard formats are offered.
  Native apps and browser attachment fields differ in which formats they accept.
  GNOME Wayland may require focusing the completion window before copying can
  finish; the pending copy is retried when that window becomes active.
- The camera uses V4L2 (raw or MJPEG), so the Flatpak needs device access. Devices
  are enumerated when the app window is created; restart after attaching a camera.
- Screen permissions use the desktop portal. Previously granted capture sources
  are remembered where the portal permits it. No capture permission is bypassed.

## Build and test

Install the GNOME 50 SDK and Rust extension for your user:

```sh
flatpak install --user flathub org.gnome.Sdk//50 org.freedesktop.Sdk.Extension.rust-stable//25.08
./scripts/build-local.sh
flatpak install --user --reinstall ./Quickcast.flatpak
./scripts/test-local.sh
```

The build script produces `Quickcast.flatpak`. The Flatpak Builder manifest is
`org.tunaos.Quickcast.json`. CI builds that manifest and runs the Meson tests,
including a synthetic screen/webcam/audio MP4 recording and a GTK clipboard/UI
check. Local tests use Xvfb and a private D-Bus session, avoiding the desktop's
clipboard. `scripts/verify-video.py` decodes the synthetic result and checks its
codecs, duration and overlay pixels.

Real GNOME portal selection, physical audio/video synchronization, and pasting
into individual chat/browser apps still require interactive hardware testing.

## Publishing

Tag releases as `v0.1.0`, etc., or dispatch **Publish Quickcast Flatpak**. The
workflow calls the shared `tuna-os/.github` publisher, pushes
`ghcr.io/tuna-os/quickcast`, and updates the served TunaOS Flatpak index. It inherits
`FLATPAK_INDEX_TOKEN` from the organization/repository; that credential needs access
to the central index repository. CI on `main` builds and tests without publishing.
