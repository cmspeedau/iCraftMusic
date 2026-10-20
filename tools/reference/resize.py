#!/usr/bin/env python3
"""Resize a named window's client area, so reference and ours can be
compared at the same pixel size.
Usage: resize.py <name-substring> <width> <height>
"""
import sys
import time

import Xlib.display

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
    w, h = int(sys.argv[2]), int(sys.argv[3])
    d = Xlib.display.Display(":0")
    win = find(d.screen().root, want)
    if win is None:
        sys.exit(f"no window matching {want!r}")
    win.configure(width=w, height=h)
    d.sync()
    time.sleep(0.8)
    geo = win.get_geometry()
    print(f"{want!r} client now {geo.width}x{geo.height}")

if __name__ == "__main__":
    main()
