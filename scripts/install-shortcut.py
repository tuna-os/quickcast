#!/usr/bin/python3
"""Add the Quickcast GNOME shortcut without replacing other custom bindings."""
import ast
import subprocess
from gi.repository import Gio

binding = "<Super><Shift>r"
root = Gio.Settings.new("org.gnome.settings-daemon.plugins.media-keys")
path = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/quickcast/"
existing = list(root.get_strv("custom-keybindings"))
if path not in existing:
    # Account for GTK's alternate spelling/order when comparing accelerators.
    def canonical(value):
        import re
        return (frozenset(re.findall(r"<([^>]+)>", value.lower())),
                re.sub(r"<[^>]+>", "", value.lower()))
    for line in subprocess.check_output(["gsettings", "list-recursively"], text=True).splitlines():
        parts = line.split(maxsplit=2)
        if len(parts) != 3:
            continue
        try:
            values = ast.literal_eval(parts[2])
        except (ValueError, SyntaxError):
            continue
        if isinstance(values, str):
            values = [values]
        if isinstance(values, list) and any(isinstance(v, str) and canonical(v) == canonical(binding) for v in values):
            raise SystemExit("Super+Shift+R is already assigned; choose a shortcut in GNOME Settings.")
    for item in existing:
        current = Gio.Settings.new_with_path("org.gnome.settings-daemon.plugins.media-keys.custom-keybinding", item)
        if canonical(current.get_string("binding")) == canonical(binding):
            raise SystemExit("Super+Shift+R is already assigned to a custom shortcut.")
settings = Gio.Settings.new_with_path("org.gnome.settings-daemon.plugins.media-keys.custom-keybinding", path)
settings.set_string("name", "Quickcast: start or stop recording")
settings.set_string("command", "gapplication action org.tunaos.Quickcast toggle-record")
settings.set_string("binding", binding)
if path not in existing:
    root.set_strv("custom-keybindings", existing + [path])
Gio.Settings.sync()
print("Quickcast shortcut: Super+Shift+R")
