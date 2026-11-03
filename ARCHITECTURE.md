# Architecture

Débarras is one Rust crate built on [GPUI](https://www.gpui.rs/) (pinned at 0.2.2). This page is
the map: where things live, which way the dependencies go, and the rules the code keeps.

## Layers

```
src/
  main.rs      entry point: reads the command line (`Start`), then shell::run()
  lang.rs      tr("English", "Français"), language detection, sizes, counts and shares as written
  actions.rs   the app's actions, dispatched by the shortcuts
  logic/       pure functions: no gpui, no disk, no window
  platform/    the outside world: the configuration folder and the scan of the disk (no gpui)
  ui/          the look shared by every view: theme, fonts and pictures, small drawn pieces
  views/       what the window draws from data alone: the plots of the plan
  shell/       the window: what is shown, the gestures, the scan in the background
    tests/     the whole app driven by simulated keystrokes and clicks
```

A layer only uses the layers below it:

```
shell ──► views ──► ui ──► logic
  │                         ▲
  └──────► platform ────────┘        lang, actions: usable everywhere
```

The test `layers_only_depend_downwards` fails when a file breaks this rule. A file that passes
1,500 lines is split by subject (`files_stay_readable`).

## Where to look

| To change… | Go to |
|---|---|
| the tree of sizes, suggestions, names and paths of items | `logic/tree.rs` |
| what can be freed, and why | `logic/rules.rs` (`classify_dir`, `classify_file`, `review_file`, `Why`) |
| how plots are laid out (squarified treemap, « others » plot, hit test) | `logic/layout.rs` |
| the demo disk | `logic/sample.rs` |
| the settings file | `logic/prefs.rs` (parsing), `platform/mod.rs` (config folder, atomic write) |
| reading a folder from the disk | `platform/scan.rs` |
| colours, themes, contrast | `ui/theme.rs` |
| fonts, the dot tile | `ui/assets.rs`, `assets/` |
| patterns, legend swatches, buttons, the window's buttons | `ui/widgets.rs` |
| how one plot is drawn, the hover marker | `views/plan.rs` |
| the window: state, scans, gestures | `shell/mod.rs` |
| the plan's surroundings: path, dimension line, title block, pointer | `shell/plan.rs` |
| the side panel | `shell/panel.rs` |
| the start screen, the scan in progress, a failure | `shell/start.rs` |
| the help panel | `shell/help.rs` |
| a shortcut | `shell/keymap.rs` (`defaults`, `help_sections`) |
| the window's frame on Linux | `shell/render.rs` |
| the test of a feature | `shell/tests/` (`journeys.rs`, `window.rs`; `benches.rs` are measures run by hand) |
| the user guide | `book/src/` (an mdBook); `shortcuts.md` is written from `help_sections` by the test `shortcuts_page_is_current` |

## How the main parts work

- **Scan** (`platform/scan.rs`): one rayon task per folder, so a disk is read on every core. The scan
  stays on the file system it starts on, never follows a link, and counts what it cannot read
  instead of stopping. Sizes are the blocks really taken on disk. Files under 1 MB are summed per
  folder into one plot (`Kind::Rest`): a tree of millions of files stays a few megabytes, and
  folders stay whole, however deep, so the plan can open any of them. The window polls the
  progress ten times a second; a scan replaced or stopped has its result dropped.
- **Rules** (`logic/rules.rs`): a folder is judged by its name and its parent's (`.cargo/registry`,
  `Library/Caches`), a file by its extension and size. Each rule carries a `Why`, worded in the
  interface's language when it is shown. A suggestion is the highest item of the tree a rule
  marks: a cache inside a cache is not counted twice.
- **Plan** (`logic/layout.rs`): squarified treemap (Bruls, Huizing & van Wijk), applied level by
  level. Items too small to show are gathered into one « others » plot; a folder large enough
  opens in place, up to three levels. The output is a flat list of tiles in drawing order.
- **Drawing** (`views/plan.rs`, `shell/plan.rs`): the plots are one view, `Plot`, drawn cached:
  moving the pointer redraws only the red marker and the title block, never the plots. The view
  is laid out again only when its key changes (`PlanKey`: the disk, the shown folder, the room,
  the theme, the ticks). Hatching is a shader pattern; dots are tiles of one SVG, rasterized once.
  Both cost a handful of primitives whatever the plot's size.
- **Themes** (`ui/theme.rs`): each theme gives six colours; the tint of each family of files is
  derived from it (the family's hue laid thinly over the paper), then the ink and the secondary
  text are moved until they read on every tint (WCAG 4.5:1). The test `every_theme_reads` checks
  all of them.
- **Window**: on Linux Débarras draws its own buttons, rounded frame and resize edges
  (client-side decorations); on macOS and Windows the system does.

## Rules

- **Nothing is deleted without the user's say**, and nothing is written outside the configuration
  folder; that folder is written atomically.
- **Interface text** always goes through `tr("English", "Français")`. Tests run in French; the
  French interface says « vous ».
- **Pure first**: logic that can be a pure function lives in `logic/` and is unit-tested there;
  every user path is tested end to end with simulated keystrokes and clicks (GPUI's test platform).
- **No new dependency** without asking: `Cargo.lock` comes from GPUI 0.2.2's and is fragile. The
  ones used are already pulled by GPUI, at the same version.
