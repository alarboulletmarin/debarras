# First steps

## The start screen

Débarras opens on three choices:

- **Scan** followed by your home folder: the usual place where space goes.
- **Choose a folder…** (`Ctrl+O`, `Cmd+O` on macOS): any folder, an external disk, a project.
- **See the demo**: a made-up home folder, to look around without waiting. Nothing on your disk is read.

You can also drop a folder on the window, or name it when you start Débarras from a terminal:

```bash
debarras ~/projects     # scan a folder
debarras --demo         # open the demo
debarras --help         # every option
```

## While it scans

The window counts the files and the bytes as they come. **Stop** (or `Esc`) cancels and goes back to the start screen.

A folder Débarras cannot read does not stop the scan: it is skipped, and the panel says how many items were unreadable.

## Look, open, go back

- **Hover** a plot to see its name, size and share of the folder in the title block below the plan.
- **Click** a folder to open it. The path above the plan shows where you are; click any part of it to go back there.
- **Go back** with `Esc`, `Backspace` or a right click.
- **Press `O`** to show the hovered item in your file manager.

## Tick what can go

Click a hatched or dotted plot to tick it. The panel adds it to **Ticked to free**, with the share of the disk it represents. Click it again to untick it.

The **Suggestions** list on the right gathers the largest plots Débarras marked, the safe ones first. Click a suggestion to tick it too.

**Move to the trash** does not delete anything yet in this version: it tells you how many items you ticked and their total. Delete those folders yourself, from your file manager or with the tool that made them (`cargo clean`, `docker system prune`…).

## Scan again

After you freed space, press `F5` (or `Ctrl+R`) to scan the same folder again.
