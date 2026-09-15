#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p test-artifacts
xvfb-run -a flatpak run --user --socket=x11 --nosocket=wayland \
    --share=network --app-path="$PWD/_app/files" --filesystem="$PWD" \
    --command=dbus-run-session org.gnome.Sdk//50 -- sh -c '
    export PATH=/usr/lib/sdk/rust-stable/bin:$PATH
    export GSETTINGS_BACKEND=memory GSK_RENDERER=cairo
    cd "$1"
    export QUICKCAST_TEST_DIR="$PWD/test-artifacts"
    meson test -C _build --print-errorlogs
    python3 scripts/verify-video.py
' sh "$PWD"
