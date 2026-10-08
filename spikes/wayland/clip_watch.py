#!/usr/bin/env python3
"""Spike: can a hidden, unfocused GTK3 process observe clipboard changes?

Run once per GDK backend and copy text from native Wayland apps meanwhile:
    GDK_BACKEND=wayland python3 clip_watch.py
    GDK_BACKEND=x11     python3 clip_watch.py   # runs under XWayland

No window is ever shown, mimicking a Tauri app sitting in the tray.
"""
import os
import sys
import time

import gi

gi.require_version("Gtk", "3.0")
gi.require_version("Gdk", "3.0")
from gi.repository import Gdk, GLib, Gtk  # noqa: E402

backend = os.environ.get("GDK_BACKEND", "auto")
display = Gdk.Display.get_default()
print(f"[{backend}] display={type(display).__name__}", flush=True)

clipboard = Gtk.Clipboard.get(Gdk.SELECTION_CLIPBOARD)
count = 0


def on_text(_clip, text):
    preview = (text or "<none>").replace("\n", "\\n")[:80]
    print(f"[{backend}] {time.strftime('%H:%M:%S')} #{count} text={preview!r}", flush=True)


def on_owner_change(clip, _event):
    global count
    count += 1
    clip.request_text(on_text)


clipboard.connect("owner-change", on_owner_change)

# Fallback probe: also poll every 2s so we can tell "no event" apart from "no access".
last = [None]


def poll():
    text = clipboard.wait_for_text()
    if text != last[0]:
        last[0] = text
        preview = (text or "<none>").replace("\n", "\\n")[:80]
        print(f"[{backend}] {time.strftime('%H:%M:%S')} poll text={preview!r}", flush=True)
    return True


GLib.timeout_add_seconds(2, poll)
print(f"[{backend}] watching... copy something in another app", flush=True)
try:
    Gtk.main()
except KeyboardInterrupt:
    sys.exit(0)
