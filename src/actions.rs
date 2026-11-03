//! The app's actions: what the shortcuts dispatch. The shell handles them.

use gpui::actions;

actions!(app, [Up, NextTheme, ToggleHelp, OpenFolder, Rescan, Reveal, Quit]);
