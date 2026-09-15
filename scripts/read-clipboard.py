#!/usr/bin/env python3
"""Read file clipboard data from a different process in the isolated test desktop."""
import sys
import gi
gi.require_version("Gtk", "4.0")
gi.require_version("Gdk", "4.0")
from gi.repository import Gdk, GLib, Gtk

Gtk.init()
clipboard = Gdk.Display.get_default().get_clipboard()
mode, expected = sys.argv[1:]
loop = GLib.MainLoop()
errors = []

def verify(actual):
    if actual != expected:
        errors.append(repr((mode, actual, expected)))
    loop.quit()


def ready(clipboard, result):
    try:
        if mode == "files":
            files = clipboard.read_value_finish(result).get_files()
            verify(files[0].get_uri())
        else:
            stream, _mime = clipboard.read_finish(result)
            chunks = []
            def read_chunk(stream, result):
                try:
                    chunk = stream.read_bytes_finish(result).get_data()
                    if chunk:
                        chunks.append(chunk)
                        stream.read_bytes_async(8192, GLib.PRIORITY_DEFAULT, None, read_chunk)
                    else:
                        text = b"".join(chunks).decode()
                        verify(text.splitlines()[1 if mode == "x-special/gnome-copied-files" else 0])
                except Exception as error:
                    errors.append(str(error))
                    loop.quit()
            stream.read_bytes_async(8192, GLib.PRIORITY_DEFAULT, None, read_chunk)
    except Exception as error:
        errors.append(str(error))
        loop.quit()


def timeout():
    errors.append("Clipboard transfer timed out")
    loop.quit()
    return GLib.SOURCE_REMOVE

if mode == "files":
    clipboard.read_value_async(Gdk.FileList, GLib.PRIORITY_DEFAULT, None, ready)
else:
    clipboard.read_async([mode], GLib.PRIORITY_DEFAULT, None, ready)
GLib.timeout_add_seconds(5, timeout)
loop.run()
if errors:
    raise SystemExit("; ".join(errors))
print("Transferred " + mode)
