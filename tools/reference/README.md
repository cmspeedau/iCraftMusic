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

`reference-ui.html` is the reference itself redrawn from those numbers: an
HTML replica of the iTunes 7 window — the list, column-browser and
now-playing views, the real menus with their items and disabled states, and
the Preferences dialog — with a toggle that overlays every measurement where
it was taken. Open it beside a live capture to check an eye against the ruler.

Working captures land under `target/reference/` (a gitignored scratch area);
the ones worth keeping live in `captures/` beside these scripts, kept in git as
the measurements the fidelity pass was drawn from. They are photographs of a
third-party application and are here as reference only. The scripts are ours.
Captures of Store, account or sign-in surfaces are never kept, tracked or
untracked: measure from them, then delete them — which is why no such frame
appears here.
