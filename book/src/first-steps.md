# First steps

## The start screen

The first time, Débarras says in three lines what it does (scan, choose, free) and what its two patterns mean, then offers three choices:

- **Scan my home folder**: the usual place where space goes.
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

- **Hover** a block to see, in the info bar below the plan, its name, size, share of the folder, family, verdict (**Safe to remove**, **Check first**, **Keep**), what freeing it does, and what a click would do.
- **Click** a folder to open it. The path above the plan shows where you are; click any part of it to go back there, or **← Up** to go back one level.
- **Go back** with `Esc`, `Backspace` or a right click.
- **Press `O`** to show the hovered item in your file manager.

## Choose what goes

The three steps, **Scan → Choose → Free**, stay at the top of the panel, the current one in ink. Under them, the panel answers first: **You can free**, the total selected, with a gauge of it (hatched for what is safe to remove, dotted for what to check first). Right after a scan it says how many safe items were selected for you; untick what you keep. The space used comes next.

Click a hatched or dotted block to select it; click it again to unselect it.

The panel has two lists:

- **What you can free**: every block Débarras marked.
- **Selected**: everything you selected, wherever it is in the folder. A folder ticked covers what it holds: an item inside it is not counted twice.

Both are trees, largest first: the items are grouped under the folders that hold them, closed, with their size and how many items they hold. Hovering a line outlines its block on the plan. Click a folder's line to open or close it. Its box ticks everything it holds, or unticks it all when everything is ticked; a dash in the box means some of it is. Click an item's line to tick or untick it.

**Why?**, by the lists, or a click on the legend above the plan, explains how the suggestions are chosen: every rule, and its reason. The [rules page](rules.md) lists them too.

## Free it

Click **Free**. A summary gives the total, then the items grouped by verdict, **Safe to remove** then **Check first**, each group with its subtotal, every item with its place on the disk and its size. Then choose:

- **Move to the trash**: the items go to the system's trash, where you can still get them back. The room comes back only once the trash is emptied.
- **Delete for good**: the room comes back at once. There is no way back.
- **Cancel** (or `Esc`): nothing happens.

The plan follows at once, without a new scan, and the panel says what was freed. An item that could not be freed, because it is gone or you may not delete it, stays selected, and the panel says why.

The trash itself (`.local/share/Trash`, `.Trash`) can only be deleted for good: that is how a trash is emptied.

In the demo, freeing only changes the plan: nothing on your disk is touched.

## Scan again

After you freed space elsewhere, press `F5` (or `Ctrl+R`) to scan the same folder again.
