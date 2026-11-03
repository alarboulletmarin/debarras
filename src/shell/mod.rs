//! The window: what is shown (start screen, scan in progress, the plan), the gestures, and the
//! scan run in the background. One `impl Shell` per subject: `plan.rs`, `panel.rs`, `start.rs`,
//! `help.rs`, `settings.rs`, `free.rs` (deleting what is ticked), `render.rs`; the shortcuts are in
//! `keymap.rs`.

mod free;
mod help;
mod keymap;
mod panel;
mod plan;
mod render;
mod settings;
mod start;
#[cfg(test)]
mod tests;

use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, atomic::Ordering::Relaxed},
    time::{Duration, Instant},
};

use gpui::{
    App, Application, Bounds, Context, Entity, FocusHandle, ResizeEdge, SharedString, TitlebarOptions, Window, WindowBackgroundAppearance, WindowBounds, WindowDecorations, WindowOptions,
    prelude::*, px, size,
};

use crate::{
    Start,
    actions::*,
    lang::{self, tr},
    logic::{
        layout::Tile,
        prefs::{Last, Prefs},
        sample,
        tree::{Node, Reclaim, outermost},
    },
    platform::{self, scan},
    ui::{assets::init_fonts, theme::Theme},
    views::plan::Plot,
};

pub use keymap::bind_keys;

/// Where the plan comes from.
#[derive(Clone, Debug, PartialEq)]
pub enum Source {
    Demo,
    Folder(PathBuf),
}

/// A disk read: the tree and what the scan met.
pub struct Disk {
    pub root: Node,
    pub source: Source,
    pub files: u64,
    /// What could not be read.
    pub errors: u64,
    pub took: Duration,
    /// What can be freed, worked out once per scan (`Node::suggestions` walks the whole tree).
    pub suggestions: Vec<Suggestion>,
}

/// An item worth freeing, as the panel lists it.
#[derive(Clone)]
pub struct Suggestion {
    pub path: Vec<usize>,
    pub name: String,
    pub size: u64,
    pub reclaim: Reclaim,
    pub why: Option<crate::logic::rules::Why>,
}

impl Disk {
    pub fn new(root: Node, source: Source, files: u64, errors: u64, took: Duration) -> Self {
        let suggestions = root
            .suggestions()
            .into_iter()
            .map(|(path, n)| Suggestion { name: root.short_name(&path), size: n.size, reclaim: n.reclaim, why: n.why, path })
            .collect();
        Disk { root, source, files, errors, took, suggestions }
    }
}

/// What the plan's drawing depends on: when it is the same as last frame, the plots are not laid out again.
#[derive(PartialEq)]
struct PlanKey {
    disk: u64,
    shown: Vec<usize>,
    map: crate::logic::layout::Rect,
    theme: &'static str,
    ticks: u64,
}

/// What is drawn over the window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Overlay {
    Help,
    Settings,
    /// The summary of what is ticked, before it is freed.
    Confirm,
}

/// The list of the panel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum List {
    Suggestions,
    Ticked,
}

/// What the window shows besides the plan.
pub enum Stage {
    /// Nothing read yet: choose a folder.
    Choose,
    Scanning { path: PathBuf, progress: Arc<scan::Progress>, started: Instant },
    Failed { path: PathBuf, reason: String },
    /// The plan of `Shell::disk`.
    Plan,
}

pub struct Shell {
    focus: FocusHandle,
    prefs: Prefs,
    theme: Theme,
    stage: Stage,
    disk: Option<Disk>,
    /// Path of the shown folder, from the root.
    shown: Vec<usize>,
    /// Items ticked to be freed, by their path from the root.
    ticked: HashSet<Vec<usize>>,
    /// The plots of the last frame, in window coordinates (hover and click).
    tiles: Vec<Tile>,
    hovered: Option<Tile>,
    overlay: Option<Overlay>,
    list: List,
    /// Ticked items are being deleted, in the background: the ticks stay as they are meanwhile.
    freeing: bool,
    lines_scroll: gpui::UniformListScrollHandle,
    /// A message under the suggestions (what a button did, what failed).
    status: Option<SharedString>,
    /// The window edge under the pointer, where the app draws its frame (Linux).
    edge: Option<ResizeEdge>,
    /// The plots, drawn cached (`views::plan::Plot`), and what they were drawn from.
    plot: Entity<Plot>,
    plot_key: Option<PlanKey>,
    /// Bumped when a disk is shown, and when a tick changes: part of `PlanKey`.
    disk_count: u64,
    tick_count: u64,
}

impl Shell {
    pub fn new(start: Start, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        focus.focus(window);
        let prefs = Prefs::parse(&platform::load_file("settings"));
        lang::set_language(prefs.language);
        let theme = Theme::of(&prefs, window.appearance());
        // The house themes follow the system's appearance while the user has chosen none.
        cx.observe_window_appearance(window, |this: &mut Self, window, cx| {
            this.theme = Theme::of(&this.prefs, window.appearance());
            cx.notify();
        })
        .detach();
        let mut shell = Shell {
            focus,
            prefs,
            theme,
            stage: Stage::Choose,
            disk: None,
            shown: Vec::new(),
            ticked: HashSet::new(),
            tiles: Vec::new(),
            hovered: None,
            overlay: None,
            list: List::Suggestions,
            freeing: false,
            lines_scroll: gpui::UniformListScrollHandle::new(),
            status: None,
            edge: None,
            plot: cx.new(|_| Plot::new()),
            plot_key: None,
            disk_count: 0,
            tick_count: 0,
        };
        // Without arguments, what was shown last opens again; a folder gone since then does not.
        let start = match (start, shell.prefs.last.clone()) {
            (Start::Choose, Some(Last::Demo)) => Start::Demo,
            (Start::Choose, Some(Last::Folder(path))) if path.is_dir() => Start::Folder(path),
            (start, _) => start,
        };
        match start {
            Start::Choose => {}
            Start::Demo => shell.show_demo(cx),
            Start::Folder(path) => shell.scan(path, cx),
        }
        shell
    }

    fn show_demo(&mut self, cx: &mut Context<Self>) {
        self.set_disk(Disk::new(sample::home(), Source::Demo, 0, 0, Duration::ZERO), cx);
    }

    /// Shows a disk: at its root, with what can go without loss ticked. It opens again at the next launch.
    fn set_disk(&mut self, disk: Disk, cx: &mut Context<Self>) {
        let last = match &disk.source {
            Source::Demo => Last::Demo,
            Source::Folder(path) => Last::Folder(path.clone()),
        };
        if self.prefs.last.as_ref() != Some(&last) {
            self.prefs.last = Some(last);
            self.save_prefs();
        }
        self.ticked = disk.suggestions.iter().filter(|s| s.reclaim == Reclaim::Safe).map(|s| s.path.clone()).collect();
        self.disk_count += 1;
        self.disk = Some(disk);
        self.shown.clear();
        self.hovered = None;
        self.status = None;
        self.list = List::Suggestions;
        self.stage = Stage::Plan;
        cx.notify();
    }

    /// Writes the settings; a failure is told in the panel.
    fn save_prefs(&mut self) {
        if let Err(e) = platform::save_file("settings", &self.prefs.to_text()) {
            self.status = Some(format!("{} {}", tr("Settings not saved:", "Réglages non enregistrés :"), lang::io_reason(&e)).into());
        }
    }

    /// Reads `path` in the background; the window shows the count growing until it is done.
    pub fn scan(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.stop_scan();
        let progress = Arc::new(scan::Progress::default());
        self.stage = Stage::Scanning { path: path.clone(), progress: progress.clone(), started: Instant::now() };
        self.hovered = None;
        cx.notify();
        let read = cx.background_spawn({
            let (path, progress) = (path.clone(), progress.clone());
            async move { scan::scan(&path, &progress) }
        });
        cx.spawn(async move |this, cx| {
            let result = read.await;
            this.update(cx, |this, cx| this.scanned(path, progress, result, cx)).ok();
        })
        .detach();
        // The count on screen, ten times a second while the scan runs.
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_millis(100)).await;
                let running = this.update(cx, |this, cx| {
                    cx.notify();
                    matches!(this.stage, Stage::Scanning { .. })
                });
                if !matches!(running, Ok(true)) {
                    break;
                }
            }
        })
        .detach();
    }

    fn scanned(&mut self, path: PathBuf, progress: Arc<scan::Progress>, result: std::io::Result<Node>, cx: &mut Context<Self>) {
        // A scan stopped or replaced by another one: its result is not wanted any more.
        let started = match &self.stage {
            Stage::Scanning { progress: current, started, .. } if Arc::ptr_eq(current, &progress) => *started,
            _ => return,
        };
        match result {
            Ok(root) => {
                let (files, errors) = (progress.files.load(Relaxed), progress.errors.load(Relaxed));
                self.set_disk(Disk::new(root, Source::Folder(path), files, errors, started.elapsed()), cx);
            }
            Err(e) => {
                self.stage = Stage::Failed { path, reason: lang::io_reason(&e) };
                cx.notify();
            }
        }
    }

    /// Stops the running scan, if any: back to the plan shown before, or to the start screen.
    fn stop_scan(&mut self) {
        if let Stage::Scanning { progress, .. } = &self.stage {
            progress.cancel.store(true, Relaxed);
            self.stage = if self.disk.is_some() { Stage::Plan } else { Stage::Choose };
        }
    }

    /// The system's folder picker, then a scan of the folder chosen.
    fn choose_folder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let paths = cx.prompt_for_paths(gpui::PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(tr("Scan this folder", "Analyser ce dossier").into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(paths))) = paths.await
                && let Some(path) = paths.into_iter().next()
            {
                this.update(cx, |this, cx| this.scan(path, cx)).ok();
            }
        })
        .detach();
    }

    fn scan_home(&mut self, cx: &mut Context<Self>) {
        match platform::home_dir() {
            Some(home) => self.scan(home, cx),
            None => {
                self.stage = Stage::Failed { path: PathBuf::from("~"), reason: tr("no home folder", "aucun dossier personnel").into() };
                cx.notify();
            }
        }
    }

    /// Reads the same folder again (after cleaning up elsewhere, for instance).
    fn rescan(&mut self, cx: &mut Context<Self>) {
        let path = match (&self.stage, &self.disk) {
            (Stage::Failed { path, .. }, _) => path.clone(),
            (Stage::Plan, Some(Disk { source: Source::Folder(path), .. })) => path.clone(),
            (Stage::Plan, Some(Disk { source: Source::Demo, .. })) => return self.show_demo(cx),
            _ => return,
        };
        self.scan(path, cx);
    }

    /// The shown folder.
    fn shown_node(&self) -> Option<&Node> {
        self.disk.as_ref().map(|d| d.root.at(&self.shown).unwrap_or(&d.root))
    }

    /// The path from the root of an item given by its path from the shown folder.
    fn rooted(&self, rel: &[usize]) -> Vec<usize> {
        self.shown.iter().chain(rel).copied().collect()
    }

    /// What freeing would delete: the ticked items not inside another ticked one, largest first.
    fn to_free(&self) -> Vec<(Vec<usize>, &Node)> {
        let Some(disk) = &self.disk else { return Vec::new() };
        let mut items: Vec<_> = outermost(&self.ticked).into_iter().filter_map(|p| disk.root.at(&p).map(|n| (p, n))).collect();
        items.sort_by(|a, b| b.1.size.cmp(&a.1.size).then_with(|| a.0.cmp(&b.0)));
        items
    }

    fn ticked_total(&self) -> u64 {
        self.to_free().iter().map(|(_, n)| n.size).sum()
    }

    fn toggle(&mut self, path: Vec<usize>, cx: &mut Context<Self>) {
        if self.freeing {
            return;
        }
        if !self.ticked.remove(&path) {
            self.ticked.insert(path);
        }
        self.tick_count += 1;
        self.status = None;
        cx.notify();
    }

    /// Shows the folder at `path` (from the root).
    fn open(&mut self, path: Vec<usize>, cx: &mut Context<Self>) {
        self.shown = path;
        self.hovered = None;
        cx.notify();
    }

    /// Back to the parent folder; what is drawn over the window closes first.
    fn up(&mut self, cx: &mut Context<Self>) {
        if self.overlay.is_some() {
            self.overlay = None;
        } else if matches!(self.stage, Stage::Scanning { .. }) {
            self.stop_scan();
        } else if self.shown.pop().is_none() {
            return;
        }
        self.hovered = None;
        cx.notify();
    }

    /// A click on a plot: one that can be freed is ticked or unticked, a folder opens.
    fn press(&mut self, tile: Tile, cx: &mut Context<Self>) {
        let Some(rel) = &tile.path else { return };
        let path = self.rooted(rel);
        if tile.reclaim != Reclaim::No {
            self.toggle(path, cx);
        } else if tile.is_dir() {
            self.open(path, cx);
        }
    }

    fn next_theme(&mut self, window: &Window, cx: &mut Context<Self>) {
        self.prefs.theme = self.theme.next_name().to_string();
        self.theme = Theme::of(&self.prefs, window.appearance());
        self.save_prefs();
        cx.notify();
    }

    /// Opens what is drawn over the window, or closes it when it is the one shown.
    fn toggle_overlay(&mut self, overlay: Overlay, cx: &mut Context<Self>) {
        self.overlay = if self.overlay == Some(overlay) { None } else { Some(overlay) };
        cx.notify();
    }

    /// Shows the hovered item in the system's file manager.
    fn reveal(&mut self, cx: &mut Context<Self>) {
        let Some(disk) = &self.disk else { return };
        let Some(rel) = self.hovered.as_ref().and_then(|t| t.path.clone()) else { return };
        if disk.source == Source::Demo {
            self.status = Some(tr("The demo is made up: there is nothing to show on disk.", "La démo est inventée : il n'y a rien à montrer sur le disque.").into());
        } else if let Some(path) = disk.root.disk_path(&self.rooted(&rel)) {
            cx.reveal_path(&path);
        }
        cx.notify();
    }
}

pub fn run(start: Start) {
    #[cfg(target_os = "linux")]
    crate::platform::skip_absent_nvidia_driver();
    Application::new().with_assets(crate::ui::assets::Assets).run(move |cx: &mut App| {
        bind_keys(cx);
        init_fonts(cx);
        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.on_window_closed(|cx| cx.quit()).detach();
        let bounds = Bounds::centered(None, size(px(1440.), px(900.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions { title: Some("Débarras".into()), ..Default::default() }),
                // Must match the .desktop file name for the desktop to associate the icon.
                app_id: Some("dev.andrea.Debarras".into()),
                // On Linux the app draws its frame: GNOME on Wayland provides none. macOS and Windows
                // ignore the request and keep their native bar.
                window_decorations: Some(WindowDecorations::Client),
                window_background: WindowBackgroundAppearance::Transparent,
                window_min_size: Some(size(px(900.), px(600.))),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| Shell::new(start, window, cx)),
        )
        .expect("the window opens");
        cx.activate(true);
    });
}
