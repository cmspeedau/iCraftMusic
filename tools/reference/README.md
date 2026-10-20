# Reference capture

The 1:1 pass measures against a reference window (iTunes 7 under Wine here)
photographed pixel-exact, so these numbers are measurements, not memories.

The desktop composites every window, so a window's own pixels survive even
when another app covers it: `grab.py` reads a window's backing pixmap and
never needs it raised. `click.py` is the exception — a click is real input at
screen coordinates, so it raises the window, clicks, and sends it back down
before capturing the result. `resize.py` puts a window at a chosen client
size, so reference and ours can be measured at the same pixels.

```sh
python3 grab.py  iTunes itunes-browser.png      # photograph, wherever it is
python3 grab.py  iTunes it.png 1180 780         # resize while capturing
python3 click.py iTunes 470 122 after-click.png # real click, then capture
python3 resize.py iTunes 1180 780               # just resize
```

The reference is a Win32 app, and its menu bar and tab controls ignore synthetic
mouse clicks — XTEST presses never open them. Keyboard input arrives fine, so
drive those surfaces with `key.py`: it asks the window manager to activate the
window, sends a key chord (modifiers held), and photographs every window the
chord made appear. `F10` arms the menu bar, `Right`/`Down`/`Return` walk and
open menus, `Escape` closes dialogs.

Images and this directory's helpers land under `target/reference/` when used
from there; keep captured images out of git — they are screenshots of
someone else's application. The scripts are ours.
