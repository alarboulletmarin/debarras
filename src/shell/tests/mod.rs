//! Tests that drive the whole app (GPUI's test platform, no screen): keystrokes and clicks go in,
//! the shell's state, what is drawn and the files on disk are checked. Pure functions are tested
//! in their own modules instead.

mod benches;
mod journeys;
mod shortcuts_page;
mod window;

use std::{
    fs,
    path::{Path, PathBuf},
};

use gpui::{Bounds, Entity, Modifiers, Pixels, TestAppContext, VisualTestContext, point, px, size};

use super::*;

static CONFIG: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// The layers only depend downwards: `logic` and `platform` never see gpui nor the layers above,
/// `ui` never sees the views, and the views never see the shell. See ARCHITECTURE.md.
#[test]
fn layers_only_depend_downwards() {
    let rules: &[(&str, &[&str])] = &[
        ("src/logic", &["gpui", "crate::platform", "crate::ui", "crate::views", "crate::shell"]),
        ("src/platform", &["gpui", "crate::ui", "crate::views", "crate::shell"]),
        ("src/ui", &["crate::views", "crate::shell", "impl Shell"]),
        ("src/views", &["crate::shell", "impl Shell"]),
    ];
    for (dir, banned) in rules {
        for path in files(Path::new(dir)) {
            let code = fs::read_to_string(&path).unwrap();
            // Comments may name anything; only code counts.
            let code: String = code.lines().map(|l| l.split("//").next().unwrap()).collect::<Vec<_>>().join("\n");
            for word in *banned {
                assert!(!code.contains(word), "{} uses `{word}`", path.display());
            }
        }
    }
}

/// The French interface speaks to the reader as « vous »: no « tu » form in a text of the app.
#[test]
fn french_says_vous() {
    let tu = [" tes ", " ton ", " ta ", "Tes ", "Ton ", " tu ", "Tu ", "-toi", " t'", "Survole ", "Choisis ", "Clique ", "Ouvre ", " ton "];
    for path in files(Path::new("src")).into_iter().filter(|p| !p.starts_with("src/shell/tests")) {
        let code = fs::read_to_string(&path).unwrap();
        let code = code.split("#[cfg(test)]\nmod tests").next().unwrap();
        for line in code.lines().map(|l| l.split("//").next().unwrap()) {
            for text in line.split('"').skip(1).step_by(2) {
                if let Some(word) = tu.iter().find(|w| text.contains(*w)) {
                    panic!("{}: « {text} » says « {} »", path.display(), word.trim());
                }
            }
        }
    }
}

/// No file grows past 1,500 lines: it gets split by subject first.
#[test]
fn files_stay_readable() {
    for path in files(Path::new("src")) {
        let lines = fs::read_to_string(&path).unwrap().lines().count();
        assert!(lines <= 1_500, "{} has {lines} lines", path.display());
    }
}

fn files(dir: &Path) -> Vec<PathBuf> {
    let entries = fs::read_dir(dir).unwrap().map(|entry| entry.unwrap().path());
    entries.flat_map(|path| if path.is_dir() { files(&path) } else { vec![path] }).filter(|p| p.extension().is_some_and(|e| e == "rs")).collect()
}

/// The config folder is global to the process (environment variables): a test that moves it under
/// `root` holds the lock until it ends.
fn isolated_config(root: &Path) -> std::sync::MutexGuard<'static, ()> {
    let guard = CONFIG.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    for key in ["XDG_CONFIG_HOME", "HOME", "APPDATA", "USERPROFILE"] {
        unsafe { std::env::set_var(key, root.join(".config")) };
    }
    guard
}

/// A temporary folder for one test, empty.
pub(super) fn temp(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("debarras-e2e-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

/// A window of 1440 × 900 on a fresh configuration, showing `start`.
pub(super) fn open<'a>(cx: &'a mut TestAppContext, name: &str, start: Start) -> (PathBuf, Entity<Shell>, &'a mut VisualTestContext, std::sync::MutexGuard<'static, ()>) {
    let root = temp(name);
    let config = isolated_config(&root);
    cx.update(bind_keys);
    cx.update(init_fonts);
    let (shell, cx) = cx.add_window_view(|window, cx| Shell::new(start, window, cx));
    cx.simulate_resize(size(px(1440.), px(900.)));
    cx.run_until_parked();
    (root, shell, cx, config)
}

/// The middle of a rectangle of the plan, in window coordinates.
pub(super) fn middle(r: crate::logic::layout::Rect) -> gpui::Point<Pixels> {
    point(px(r.x + r.w / 2.), px(r.y + r.h / 2.))
}

pub(super) fn click(cx: &mut VisualTestContext, at: gpui::Point<Pixels>) {
    cx.simulate_mouse_move(at, None, Modifiers::none());
    cx.simulate_click(at, Modifiers::none());
    cx.run_until_parked();
}

pub(super) fn click_on(cx: &mut VisualTestContext, selector: &'static str) {
    let bounds: Bounds<Pixels> = cx.debug_bounds(selector).unwrap_or_else(|| panic!("{selector} is not drawn"));
    click(cx, bounds.center());
}
