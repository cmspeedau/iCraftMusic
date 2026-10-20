#!/usr/bin/env python3
"""Click inside a named window and capture the result, unoccluded.

The click is real XTEST input at the window's screen position, so the window
has to come forward for the click; it is sent back down afterwards. The
capture reads the backing pixmap, so it never needs the window on top.
Usage: click.py <name-substring> <x> <y> <out.png>
x,y are inside the window's client area as seen in a grab.py image.
"""
import sys
import time

import Xlib.display
from Xlib.ext import composite, xtest
from PIL import Image

def find(root, want):
    for c in root.query_tree().children:
        try:
            name = c.get_wm_name()
        except Exception:
            name = None
        if name and want.lower() in name.lower():
            return c
        hit = find(c, want)
        if hit is not None:
            return hit
    return None

def main():
    want = sys.argv[1]
    x, y = int(sys.argv[2]), int(sys.argv[3])
    path = sys.argv[4]
    d = Xlib.display.Display(":0")
    root = d.screen().root
    win = find(root, want)
    if win is None:
        sys.exit(f"no window matching {want!r}")
    geo = win.get_geometry()
    if not (0 <= x < geo.width and 0 <= y < geo.height):
        sys.exit(f"click {x},{y} outside {geo.width}x{geo.height} — refused")
    t = win.translate_coords(root, 0, 0)
    gx, gy = t.x + x, t.y + y
    win.raise_window()
    d.sync()
    time.sleep(0.4)
    xtest.fake_input(d, Xlib.X.MotionNotify, x=gx, y=gy)
    xtest.fake_input(d, Xlib.X.ButtonPress, 1, root=root.id, x=gx, y=gy)
    xtest.fake_input(d, Xlib.X.ButtonRelease, 1, root=root.id, x=gx, y=gy)
    d.sync()
    time.sleep(0.6)
    win.configure(stack_mode=Xlib.X.BottomIf)
    d.sync()
    pix = win.composite_name_window_pixmap(composite.NameWindowPixmap)
    if not hasattr(pix, "get_image"):
        pix = pix.pixmap
    img = pix.get_image(0, 0, geo.width, geo.height, Xlib.X.ZPixmap, 0xFFFFFFFF)
    im = Image.frombuffer("RGB", (geo.width, geo.height), img.data, "raw", "BGRX", 0, 1)
    im.save(path)
    print(path, im.size)

if __name__ == "__main__":
    main()
