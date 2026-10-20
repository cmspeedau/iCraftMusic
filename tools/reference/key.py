#!/usr/bin/env python3
"""Send a key chord to a named window (WM-activated first), then photograph what appeared.

Win32 menus and tab controls ignore synthetic mouse clicks — XTEST presses never
open them — but keyboard input reaches the app fine: F10/Right/Down/Return drive
the menu bar, Escape closes dialogs, Space toggles playback. This script sends
any chord (modifiers held across the rest) and shoots every newly-mapped small
window so the result is readable even when the WM never shows it.
Usage: key.py <window-substring> <keysym> [keysym...] <out-prefix>
   e.g. key.py iTunes F10 f10   then   key.py iTunes Down adv-menu
"""
import sys
import time

import Xlib.display
from Xlib import X, XK
from Xlib.ext import composite, xtest
from Xlib.protocol import event
from PIL import Image


def find(root, want):
    def rec(w):
        for c in w.query_tree().children:
            try:
                name = c.get_wm_name()
            except Exception:
                name = None
            if name and want.lower() in name.lower():
                return c
            hit = rec(c)
            if hit:
                return hit
    return rec(root)


def viewables(root):
    out = []

    def rec(w):
        for c in w.query_tree().children:
            try:
                if c.get_attributes().map_state == X.IsViewable:
                    g = c.get_geometry()
                    try:
                        name = c.get_wm_name()
                    except Exception:
                        name = None
                    out.append((c, g, name))
            except Exception:
                pass
            rec(c)

    rec(root)
    return out


def shoot(win, g, path):
    pix = win.composite_name_window_pixmap(composite.NameWindowPixmap)
    if not hasattr(pix, "get_image"):
        pix = pix.pixmap
    img = pix.get_image(0, 0, g.width, g.height, X.ZPixmap, 0xFFFFFFFF)
    Image.frombuffer("RGB", (g.width, g.height), img.data, "raw", "BGRX", 0, 1).save(path)


def main():
    want, keys, prefix = sys.argv[1], sys.argv[2:-1], sys.argv[-1]
    d = Xlib.display.Display(":0")
    root = d.screen().root
    win = find(root, want)
    before = {c.id for c, _, _ in viewables(root)}
    net_active = d.intern_atom("_NET_ACTIVE_WINDOW")
    ev = event.ClientMessage(window=win, client_type=net_active, data=(32, [2, X.CurrentTime, 0, 0, 0]))
    root.send_event(ev, event_mask=X.SubstructureNotifyMask)
    d.sync()
    time.sleep(1.0)
    codes = [d.keysym_to_keycode(XK.string_to_keysym(k)) for k in keys]
    if any(c == 0 for c in codes):
        sys.exit(f"unknown keysym in {keys}")
    mods = [c for k, c in zip(keys, codes) if k.endswith("_L") or k.endswith("_R")]
    rest = [c for k, c in zip(keys, codes) if c not in mods]
    for c in mods:
        xtest.fake_input(d, X.KeyPress, c)
    for c in rest:
        xtest.fake_input(d, X.KeyPress, c)
        xtest.fake_input(d, X.KeyRelease, c)
    for c in reversed(mods):
        xtest.fake_input(d, X.KeyRelease, c)
    d.sync()
    time.sleep(1.2)
    fresh = 0
    for c, g, name in viewables(root):
        if c.id in before:
            continue
        fresh += 1
        print(f"new 0x{c.id:x} {g.width}x{g.height} at {g.x},{g.y} name={name!r}")
        if 40 < g.width < 1100 and 40 < g.height < 1100:
            try:
                shoot(c, g, f"{prefix}-{c.id:x}.png")
                print(f"  shot {prefix}-{c.id:x}.png")
            except Exception as e:
                print(f"  shot failed: {e}")
    if not fresh:
        print("no new windows (the chord still landed — Space/Escape change state, not windows)")


if __name__ == "__main__":
    main()
