#!/usr/bin/env python3
"""Spike: can a GNOME custom shortcut pop a pre-created window *with focus*?

Server (keeps a hidden window alive, like the Tauri app would):
    GDK_BACKEND=wayland python3 popup_toggle.py serve
    GDK_BACKEND=x11     python3 popup_toggle.py serve

Client (what the GNOME custom shortcut runs; forwards activation env):
    python3 popup_toggle.py toggle wayland|x11
"""
import json
import os
import socket
import sys
import time

def sock_path(backend):
    return os.path.join(os.environ.get("XDG_RUNTIME_DIR", "/tmp"), f"quickdesk-spike-{backend}.sock")


def toggle(backend):
    payload = {
        "t": time.time(),
        "XDG_ACTIVATION_TOKEN": os.environ.get("XDG_ACTIVATION_TOKEN"),
        "DESKTOP_STARTUP_ID": os.environ.get("DESKTOP_STARTUP_ID"),
    }
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
        s.connect(sock_path(backend))
        s.sendall(json.dumps(payload).encode())


def serve():
    import gi

    gi.require_version("Gtk", "3.0")
    from gi.repository import GLib, Gtk

    backend = os.environ.get("GDK_BACKEND", "auto")
    win = Gtk.Window(title="QuickDesk spike")
    win.set_default_size(420, 120)
    win.set_keep_above(True)
    win.set_position(Gtk.WindowPosition.CENTER)
    entry = Gtk.Entry(placeholder_text="What do you want to remember?")
    win.add(entry)
    win.connect("delete-event", lambda w, e: w.hide() or True)
    entry.connect("activate", lambda e: (print(f"[{backend}] saved: {e.get_text()!r}", flush=True), e.set_text(""), win.hide()))
    win.connect("key-press-event", lambda w, ev: ev.keyval == 0xFF1B and (w.hide() or True))  # Esc

    def report(sent_at):
        lat = (time.time() - sent_at) * 1000
        print(f"[{backend}] visible={win.get_visible()} active={win.is_active()} "
              f"has_focus={entry.has_focus()} latency={lat:.0f}ms", flush=True)
        return False

    def on_conn(srv, _cond):
        conn, _ = srv.accept()
        msg = json.loads(conn.recv(4096).decode())
        conn.close()
        print(f"[{backend}] toggle env: token={msg['XDG_ACTIVATION_TOKEN']!r} "
              f"startup_id={msg['DESKTOP_STARTUP_ID']!r}", flush=True)
        if win.get_visible() and win.is_active():
            win.hide()
            return True
        startup = msg["XDG_ACTIVATION_TOKEN"] or msg["DESKTOP_STARTUP_ID"]
        if startup:
            win.set_startup_id(startup)
        win.show_all()
        win.present()
        entry.grab_focus()
        GLib.timeout_add(300, report, msg["t"])
        return True

    SOCK = sock_path(backend)
    if os.path.exists(SOCK):
        os.unlink(SOCK)
    srv = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    srv.bind(SOCK)
    srv.listen(4)
    GLib.io_add_watch(srv.fileno(), GLib.IO_IN, lambda *_: on_conn(srv, _))
    # Pre-realize so the first show is fast.
    win.realize()
    print(f"[{backend}] serving on {SOCK}", flush=True)
    Gtk.main()


if __name__ == "__main__":
    if sys.argv[1] == "serve":
        serve()
    else:
        toggle(sys.argv[2])
