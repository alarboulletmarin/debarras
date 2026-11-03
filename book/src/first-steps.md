# First steps

## The start screen

The first time, Débarras opens on three choices:

- **Scan** followed by your home folder: the usual place where space goes.
- **Choose a folder…** (`Ctrl+O`, `Cmd+O` on macOS): any folder, an external disk, a project.
- **See the demo**: a made-up home folder, to look around without waiting. Nothing on your disk is read.

You can also drop a folder on the window, or name it when you start Débarras from a terminal:

```bash
debarras ~/projects     # scan a folder
debarras --demo         # open the demo
debarras --help         # every option
```

The next times, it opens what you looked at last: the same folder, scanned again, or the demo. A folder gone since then brings back the start screen. To look elsewhere, click **Open…** at the bottom of the panel (`Ctrl+O`).

## While it scans

The window counts the files and the bytes as they come. **Stop** (or `Esc`) cancels and goes back to the start screen.

A folder Débarras cannot read does not stop the scan: it is skipped, and the panel says how many items were unreadable.

## Look, open, go back

- **Hover** a plot to see its name, size and share of the folder in the title block below the plan.
- **Click** a folder to open it. The path above the plan shows where you are; click any part of it to go back there.
- **Go back** with `Esc`, `Backspace` or a right click.
- **Press `O`** to show the hovered item in your file manager.

## Tick what can go

Click a hatched or dotted plot to tick it. The panel adds it to **Ticked to free**, with the share of the disk it represents. Click it again to untick it. What can go without loss is ticked from the start.

The panel has two lists:

- **Suggestions**: the largest plots Débarras marked, the safe ones first. Click one to tick or untick it.
- **Ticked**: everything you ticked, wherever it is in the folder, largest first. Click one to untick it. A folder ticked covers what it holds: an item inside it is not counted twice.

## Free it

Click **Free**. A summary lists every ticked item, its place on the disk and its size, with the total. Then choose:

- **Move to the trash**: the items go to the system's trash, where you can still get them back. The room comes back only once the trash is emptied.
- **Delete for good**: the room comes back at once. There is no way back.
- **Cancel** (or `Esc`): nothing happens.

The plan follows at once, without a new scan, and the panel says what was freed. An item that could not be freed, because it is gone or you may not delete it, stays ticked, and the panel says why.

The trash itself (`.local/share/Trash`, `.Trash`) can only be deleted for good: that is how a trash is emptied.

In the demo, freeing only changes the plan: nothing on your disk is touched.

## Scan again

After you freed space elsewhere, press `F5` (or `Ctrl+R`) to scan the same folder again.
