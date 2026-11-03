//! The user's paths, from the first screen to a plan, through the gestures on it.

use std::sync::atomic::Ordering::Relaxed;

use super::*;
use crate::logic::{layout::Tile, tree::Kind};

fn tile_named(shell: &Entity<Shell>, cx: &mut VisualTestContext, name: &str) -> Tile {
    shell.read_with(cx, |s, _| s.tiles.iter().find(|t| t.name == name).cloned()).unwrap_or_else(|| panic!("no plot named {name}"))
}

#[gpui::test]
fn the_demo_opens_folders_ticks_and_goes_back(cx: &mut TestAppContext) {
    let (_root, shell, cx, _config) = open(cx, "demo", Start::Demo);
    // What can go without loss is ticked from the start.
    let (ticked, safe) = shell.read_with(cx, |s, _| (s.ticked_total(), s.disk.as_ref().unwrap().root.safe_total()));
    assert!(ticked > 0 && ticked == safe);

    // Hovering a plot names it in the title block.
    let videos = tile_named(&shell, cx, "Vidéos");
    cx.simulate_mouse_move(middle(videos.rect), None, Modifiers::none());
    cx.run_until_parked();
    let hovered = shell.read_with(cx, |s, _| s.hovered.clone()).expect("a hovered plot");
    assert!(hovered.depth >= 1);

    // A folder that is to keep opens on a click; Escape goes back.
    click(cx, point(px(videos.rect.x + 4.), px(videos.rect.y + 4.)));
    assert_eq!(shell.read_with(cx, |s, _| s.shown_node().unwrap().name.clone()), "Vidéos");
    cx.simulate_keystrokes("escape");
    assert!(shell.read_with(cx, |s, _| s.shown.is_empty()));

    // A plot that can go is unticked, then ticked again, by a click.
    let cache = tile_named(&shell, cx, ".cache");
    let before = shell.read_with(cx, |s, _| s.ticked_total());
    click(cx, point(px(cache.rect.x + 4.), px(cache.rect.y + 4.)));
    assert_eq!(shell.read_with(cx, |s, _| s.ticked_total()), before - cache.size);
    click(cx, point(px(cache.rect.x + 4.), px(cache.rect.y + 4.)));
    assert_eq!(shell.read_with(cx, |s, _| s.ticked_total()), before);

    // A suggestion of the panel does the same.
    click_on(cx, "suggestion-0");
    assert!(shell.read_with(cx, |s, _| s.ticked_total()) < before);

    // Escape at the root does nothing; the right button goes back too.
    cx.simulate_keystrokes("escape");
    assert!(shell.read_with(cx, |s, _| s.shown.is_empty()));
    let projects = tile_named(&shell, cx, "projets");
    click(cx, point(px(projects.rect.x + 4.), px(projects.rect.y + 4.)));
    assert_eq!(shell.read_with(cx, |s, _| s.shown.len()), 1);
    cx.simulate_mouse_down(middle(Shell::map_rect((1440., 900.))), gpui::MouseButton::Right, Modifiers::none());
    cx.run_until_parked();
    assert!(shell.read_with(cx, |s, _| s.shown.is_empty()));

    // Freeing sums up first; in the demo, what is ticked leaves the plan and nothing else happens.
    let (total, picked, items) = shell.read_with(cx, |s, _| (s.disk.as_ref().unwrap().root.size, s.ticked_total(), s.to_free().len()));
    click_on(cx, "free");
    assert!(cx.debug_bounds("confirm").is_some());
    click_on(cx, "confirm-trash");
    assert_eq!(overlay(&shell, cx), None);
    let (after, ticked, status) = shell.read_with(cx, |s, _| (s.disk.as_ref().unwrap().root.size, s.ticked.len(), s.status.clone().unwrap()));
    assert_eq!((after, ticked), (total - picked, 0));
    assert!(status.contains(&format!("{items} éléments")), "{status}");
    // Nothing ticked any more: the button says what to do.
    click_on(cx, "free");
    assert_eq!(overlay(&shell, cx), None);
    assert!(shell.read_with(cx, |s, _| s.status.clone().unwrap()).contains("Rien n'est coché"));
}

#[gpui::test]
fn the_path_above_the_plan_goes_back_several_levels(cx: &mut TestAppContext) {
    let (_root, shell, cx, _config) = open(cx, "chemin", Start::Demo);
    let local = tile_named(&shell, cx, ".local/share");
    click(cx, point(px(local.rect.x + 4.), px(local.rect.y + 4.)));
    let flatpak = tile_named(&shell, cx, "flatpak");
    click(cx, point(px(flatpak.rect.x + 4.), px(flatpak.rect.y + 4.)));
    assert_eq!(shell.read_with(cx, |s, _| s.shown.len()), 2);
    click_on(cx, "crumb-0");
    assert!(shell.read_with(cx, |s, _| s.shown.is_empty()));
}

#[gpui::test]
fn a_real_folder_is_scanned_with_its_rules(cx: &mut TestAppContext) {
    let dir = temp("dossier");
    fs::create_dir_all(dir.join("appli/node_modules/paquet")).unwrap();
    fs::write(dir.join("appli/node_modules/paquet/index.js"), vec![1u8; 3_000_000]).unwrap();
    fs::write(dir.join("film.mkv"), vec![1u8; 2_000_000]).unwrap();
    fs::write(dir.join("note.md"), b"court").unwrap();
    let (_root, shell, cx, _config) = open(cx, "scan", Start::Folder(dir.clone()));
    let (stage_is_plan, files, suggestions) = shell.read_with(cx, |s, _| {
        let disk = s.disk.as_ref().unwrap();
        (matches!(s.stage, Stage::Plan), disk.files, disk.suggestions.len())
    });
    assert!(stage_is_plan);
    assert_eq!((files, suggestions), (3, 1));
    // The small file is summed (too small to be drawn beside the others).
    assert!(shell.read_with(cx, |s, _| s.disk.as_ref().unwrap().root.children.iter().any(|c| c.kind == Kind::Rest(1))));

    // Scanned again after a change on disk: the plan follows.
    fs::remove_dir_all(dir.join("appli")).unwrap();
    cx.simulate_keystrokes("f5");
    cx.run_until_parked();
    assert_eq!(shell.read_with(cx, |s, _| s.disk.as_ref().unwrap().suggestions.len()), 0);
    fs::remove_dir_all(&dir).unwrap();
}

#[gpui::test]
fn a_folder_that_cannot_be_read_says_why(cx: &mut TestAppContext) {
    let absent = std::env::temp_dir().join("debarras-e2e-absent-ici");
    let (_root, shell, cx, _config) = open(cx, "absent", Start::Folder(absent));
    let reason = shell.read_with(cx, |s, _| match &s.stage {
        Stage::Failed { reason, .. } => reason.clone(),
        _ => String::new(),
    });
    assert_eq!(reason, "introuvable");
    assert!(cx.debug_bounds("failure").is_some());
    // From there, the demo is one click away.
    click_on(cx, "demo");
    assert!(shell.read_with(cx, |s, _| matches!(s.stage, Stage::Plan)));
}

#[gpui::test]
fn an_empty_folder_says_so(cx: &mut TestAppContext) {
    let dir = temp("vide");
    let (_root, shell, cx, _config) = open(cx, "vide-fenetre", Start::Folder(dir.clone()));
    assert!(shell.read_with(cx, |s, _| matches!(s.stage, Stage::Plan) && s.tiles.is_empty()));
    assert!(cx.debug_bounds("map").is_some());
    fs::remove_dir_all(&dir).unwrap();
}

#[gpui::test]
fn the_start_screen_offers_the_demo(cx: &mut TestAppContext) {
    let (_root, shell, cx, _config) = open(cx, "accueil", Start::Choose);
    for selector in ["scan-home", "choose", "demo"] {
        assert!(cx.debug_bounds(selector).is_some(), "{selector}");
    }
    click_on(cx, "demo");
    assert!(shell.read_with(cx, |s, _| s.disk.as_ref().is_some_and(|d| d.source == Source::Demo)));
}

#[gpui::test]
fn a_scan_can_be_stopped(cx: &mut TestAppContext) {
    let (_root, shell, cx, _config) = open(cx, "arret", Start::Demo);
    // A scan stopped before it ends leaves the plan shown before, and its result is dropped.
    let progress = Arc::new(scan::Progress::default());
    shell.update(cx, |s, _| s.stage = Stage::Scanning { path: "/".into(), progress: progress.clone(), started: Instant::now() });
    cx.simulate_keystrokes("escape");
    assert!(progress.cancel.load(Relaxed));
    assert!(shell.read_with(cx, |s, _| matches!(s.stage, Stage::Plan) && s.disk.as_ref().unwrap().source == Source::Demo));
    shell.update(cx, |s, cx| s.scanned("/".into(), progress, Ok(Node::dir("/", crate::logic::tree::Category::Other, vec![])), cx));
    assert!(shell.read_with(cx, |s, _| s.disk.as_ref().unwrap().source == Source::Demo));
}

#[gpui::test]
fn the_theme_key_goes_round_and_is_kept(cx: &mut TestAppContext) {
    let (_root, shell, cx, _config) = open(cx, "theme", Start::Demo);
    let first = shell.read_with(cx, |s, _| s.theme.name);
    cx.simulate_keystrokes("t");
    let second = shell.read_with(cx, |s, _| s.theme.name);
    assert_ne!(first, second);
    let saved = fs::read_to_string(platform::config_dir().join("settings")).unwrap();
    assert_eq!(Prefs::parse(&saved).theme, second);
    for _ in 1..crate::ui::theme::THEMES.len() {
        cx.simulate_keystrokes("t");
    }
    assert_eq!(shell.read_with(cx, |s, _| s.theme.name), first, "the key goes round every theme");
}

#[gpui::test]
fn the_help_opens_and_closes(cx: &mut TestAppContext) {
    let (_root, shell, cx, _config) = open(cx, "aide", Start::Demo);
    let projects = tile_named(&shell, cx, "projets");
    click(cx, point(px(projects.rect.x + 4.), px(projects.rect.y + 4.)));
    cx.simulate_keystrokes("f1");
    assert!(cx.debug_bounds("help").is_some());
    // Escape closes the help first, and leaves the folder shown.
    cx.simulate_keystrokes("escape");
    assert_eq!(overlay(&shell, cx), None);
    assert_eq!(shell.read_with(cx, |s, _| s.shown.len()), 1);
    cx.simulate_keystrokes("f1");
    click_on(cx, "help");
    assert_eq!(overlay(&shell, cx), Some(Overlay::Help), "a click inside the help keeps it");
}

#[gpui::test]
fn the_demo_has_nothing_to_show_in_the_file_manager(cx: &mut TestAppContext) {
    let (_root, shell, cx, _config) = open(cx, "montrer", Start::Demo);
    let videos = tile_named(&shell, cx, "Vidéos");
    cx.simulate_mouse_move(middle(videos.rect), None, Modifiers::none());
    cx.simulate_keystrokes("o");
    assert!(shell.read_with(cx, |s, _| s.status.clone()).unwrap().contains("inventée"));
}

#[gpui::test]
fn the_pointer_never_draws_the_plots_again(cx: &mut TestAppContext) {
    let (_root, shell, cx, _config) = open(cx, "survol", Start::Demo);
    let plot = shell.read_with(cx, |s, _| s.plot.clone());
    let before = plot.read_with(cx, |p, _| p.renders);
    let map = Shell::map_rect((1440., 900.));
    // Fifty steps across the plan, through many plots: the marker and the title block follow.
    let mut seen = std::collections::HashSet::new();
    for step in 0..50 {
        let at = point(px(map.x + map.w * step as f32 / 50.), px(map.y + map.h * ((step * 7) % 50) as f32 / 50.));
        cx.simulate_mouse_move(at, None, Modifiers::none());
        cx.run_until_parked();
        if let Some(t) = shell.read_with(cx, |s, _| s.hovered.clone()) {
            seen.insert(t.name);
        }
    }
    assert!(seen.len() > 5, "the pointer went over {} plots only", seen.len());
    assert_eq!(plot.read_with(cx, |p, _| p.renders), before, "hovering drew the plots again");
    // A tick does draw them again: the hatching gets darker.
    click_on(cx, "suggestion-0");
    assert!(plot.read_with(cx, |p, _| p.renders) > before);
}
