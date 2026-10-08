# Quickcast Architecture

Quickcast is a GNOME screen recorder designed for quick video explanations. It records screen content, microphone/desktop audio, and an optional circular floating webcam bubble into an MP4 file, automatically copying the result to the clipboard.

## Component Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Quickcast GTK4 Application               │
│                                                             │
│  ┌───────────────────────┐       ┌───────────────────────┐  │
│  │   UI & Settings       │       │  Webcam Bubble Window │  │
│  │   (libadwaita / GTK)  │       │  (Transparent Window) │  │
│  └───────────┬───────────┘       └───────────┬───────────┘  │
│              │                               │              │
│              ▼                               ▼              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              GStreamer Capture Engine                 │  │
│  │  Screen / Monitor / Window + Webcam + Audio Mixer     │  │
│  └───────────────────────────┬───────────────────────────┘  │
└──────────────────────────────┼──────────────────────────────┘
                               │ DBus / Portal API
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                   GNOME Shell Companion                     │
│  (Window tracking, floating position relay, always-on-top)   │
└─────────────────────────────────────────────────────────────┘
```

## Core Subsystems

### 1. User Interface (`src/ui/`)
Built with GTK4 and `libadwaita`. Manages recording controls (monitor/window/region selection, microphone/desktop audio toggles, webcam bubble toggle, and size menu).

### 2. GStreamer Capture Engine (`src/capture/`)
Encodes video and audio using GStreamer:
- **Video Capture**: Uses the XDG Desktop Portal (`org.freedesktop.portal.ScreenCast`) for Wayland screen/window capture.
- **Webcam Overlay**: Composites V4L2 camera input as a circular overlay over the recorded screen or window area.
- **Audio Pipeline**: Mixes PulseAudio/PipeWire microphone and desktop audio streams.
- **Encoder**: Produces H.264 + AAC in MP4 container at 30 fps (quality x264 preset `veryfast`).

### 3. GNOME Shell Extension (`extension/`)
A host Shell extension companion providing:
- Floating webcam bubble position tracking across workspaces.
- Keeping the webcam overlay always-on-top.
- Start/stop recording actions from the GNOME Shell top panel menu.

### 4. Clipboard & Storage (`src/utils/`)
- Saves finished MP4 recordings to `~/Videos/Quickcast`.
- Offers GTK file-list, URI-list, and GNOME copied-file clipboard formats upon finalization.
