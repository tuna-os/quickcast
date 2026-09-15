#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
gnome-extensions pack --force --out-dir=build-aux extension
gnome-extensions install --force build-aux/quickcast@tunaos.org.shell-extension.zip
if gnome-extensions enable quickcast@tunaos.org; then
    gnome-extensions info quickcast@tunaos.org
else
    /usr/bin/python3 - <<'PY'
from gi.repository import Gio
settings = Gio.Settings.new('org.gnome.shell')
uuid = 'quickcast@tunaos.org'
enabled = list(settings.get_strv('enabled-extensions'))
if uuid not in enabled:
    settings.set_strv('enabled-extensions', enabled + [uuid])
Gio.Settings.sync()
print('Quickcast Desktop is installed and enabled for the next GNOME session. Log out and back in to load it.')
PY
fi
