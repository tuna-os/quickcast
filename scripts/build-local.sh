#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
project_dir="$PWD"
flatpak run --user --share=network --filesystem="$project_dir" --command=sh org.gnome.Sdk//50 -c '
    set -eu
    export PATH=/usr/lib/sdk/rust-stable/bin:$PATH
    cd "$1"
    if [ ! -d _build ]; then meson setup _build --prefix=/app -Dprofile=development; fi
    ninja -C _build
    DESTDIR="$PWD/_stage" meson install -C _build --no-rebuild
    glib-compile-schemas _stage/app/share/glib-2.0/schemas
' sh "$project_dir"
if [ -d _app ]; then
    rm -rf _app
fi
if [ ! -d _app ]; then
    flatpak build-init _app org.tunaos.Quickcast org.gnome.Sdk org.gnome.Platform 50
fi
cp -a _stage/app/. _app/files/
flatpak build-finish --command=quickcast --device=all --filesystem=xdg-videos --filesystem=/run/udev:ro --share=ipc --socket=fallback-x11 --socket=wayland --socket=pulseaudio --env=KOOHA_EXPERIMENTAL=window-recording _app
flatpak build-export _repo _app
flatpak build-bundle _repo Quickcast.flatpak org.tunaos.Quickcast
