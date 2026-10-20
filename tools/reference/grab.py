#!/usr/bin/env python3
"""Capture a named window by backing pixmap, unoccluded, to stdout or a file.

The whole 1:1 pass measures from these images: the desktop composites every
window, so a window's pixels survive even when some other app covers it.
Usage: grab.py <name-substring> <out.png> [width height]
Giving width and height resizes the window first, so the reference and ours
can be put at identical pixel sizes for measurement.
"""
import sys
import time

import Xlib.display
from Xlib.ext import composite
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

def grab(win, path):
    geo = win.get_geometry()
    pix = win.composite_name_window_pixmap(composite.NameWindowPixmap)
    if not hasattr(pix, "get_image"):
        pix = pix.pixmap
    img = pix.get_image(0, 0, geo.width, geo.height, Xlib.X.ZPixmap, 0xFFFFFFFF)
    im = Image.frombuffer("RGB", (geo.width, geo.height), img.data, "raw", "BGRX", 0, 1)
    im.save(path)
    print(path, im.size)

def main():
    want, path = sys.argv[1], sys.argv[2]
    d = Xlib.display.Display(":0")
    root = d.screen().root
    win = find(root, want)
    if win is None:
        sys.exit(f"no window matching {want!r}")
    if len(sys.argv) > 4:
        w, h = int(sys.argv[3]), int(sys.argv[4])
        win.configure(width=w, height=h)
        d.sync()
        time.sleep(0.8)
    grab(win, path)

if __name__ == "__main__":
    main()
