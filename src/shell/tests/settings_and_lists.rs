//! The panel: the list of what is ticked, the settings, and what opens again at the next launch.

use super::*;
use crate::{lang::Language, ui::theme::THEMES};

/// The settings as written, where the app writes them on this system (the environment is isolated).
fn settings_file() -> Prefs {
    Prefs::parse(&fs::read_to_string(platform::config_dir().join("settings")).unwrap_or_default())
}

/// What the list shows: (depth, label, items ticked, items, open).
fn rows(shell: &Entity<Shell>, cx: &mut VisualTestContext) -> Vec<(usize, String, usize, usize, Option<bool>)> {
    shell.read_with(cx, |s, _| s.lines().into_iter().map(|r| (r.depth, r.label, r.items.iter().filter(|p| s.ticked.contains(*p)).count(), r.items.len(), r.open)).collect())
}

#[gpui::test]
fn the_suggestions_are_a_tree_of_folders(cx: &mut TestAppContext) {
    let (_root, shell, cx, _config) = open(cx, "arbre", Start::Demo);
    // Folders closed, largest first: a few lines for every suggestion.
    let top = rows(&shell, cx);
    let all = shell.read_with(cx, |s, _| s.disk.as_ref().unwrap().suggestions.len());
    assert_eq!(top.iter().map(|r| r.3).sum::<usize>(), all, "every suggestion is under a line");
    assert!(top.len() < all);
    let folder = top.iter().position(|r| r.4 == Some(false) && r.3 >= 2).expect("a closed folder");
    let at = |i: usize| -> &'static str { Box::leak(format!("suggestion-{i}").into_boxed_str()) };
    let boxed = |i: usize| -> &'static str { Box::leak(format!("suggestion-{i}-box").into_boxed_str()) };
    // A click on the line opens the folder, its items under it; a second closes it.
    click_on(cx, at(folder));
    let opened = rows(&shell, cx);
    assert_eq!(opened[folder].4, Some(true));
    assert_eq!(opened.len(), top.len() + opened.iter().filter(|r| r.0 == 1).count());
    assert!(opened.iter().filter(|r| r.0 == 1).count() >= 2);
    assert!(cx.debug_bounds(at(folder + 1)).unwrap().left() > cx.debug_bounds(at(folder)).unwrap().left() - px(1.), "the items are under it");
    assert!(cx.debug_bounds(boxed(folder + 1)).unwrap().left() > cx.debug_bounds(boxed(folder)).unwrap().left(), "and set in");
    // The folder's box ticks all it holds, then unticks them all, without closing it.
    let before = shell.read_with(cx, |s, _| s.ticked_total());
    // Partly ticked, a click ticks all; all ticked, it unticks all.
    for _ in 0..2 {
        if rows(&shell, cx)[folder].2 > 0 {
            click_on(cx, boxed(folder));
        }
    }
    assert_eq!(rows(&shell, cx)[folder].2, 0);
    click_on(cx, boxed(folder));
    assert_eq!(rows(&shell, cx)[folder].2, opened[folder].3, "all ticked");
    assert!(rows(&shell, cx).iter().filter(|r| r.0 == 1).all(|r| r.2 == 1));
    click_on(cx, boxed(folder));
    assert_eq!(rows(&shell, cx)[folder].2, 0, "all unticked");
    assert_eq!(rows(&shell, cx)[folder].4, Some(true), "the box does not close the folder");
    // An item's line ticks it: the folder is partly ticked.
    click_on(cx, at(folder + 1));
    assert_eq!(rows(&shell, cx)[folder].2, 1);
    assert_ne!(shell.read_with(cx, |s, _| s.ticked_total()), before);
    click_on(cx, at(folder));
    assert_eq!(rows(&shell, cx).len(), top.len(), "closed again");
}

#[gpui::test]
fn the_ticked_list_recalls_every_tick(cx: &mut TestAppContext) {
    let (_root, shell, cx, _config) = open(cx, "coche", Start::Demo);
    let ticked = shell.read_with(cx, |s, _| s.to_free().len());
    assert!(ticked >= 2);
    click_on(cx, "tab-ticked");
    let lines = rows(&shell, cx);
    assert_eq!(lines.iter().map(|r| r.3).sum::<usize>(), ticked, "every ticked item is under a line");
    assert!(lines.iter().all(|r| r.2 == r.3), "all of them ticked");
    assert!(cx.debug_bounds(Box::leak(format!("ticked-{}", lines.len() - 1).into_boxed_str())).is_some());
    // Largest first; a box unticks the line.
    let sizes: Vec<u64> = shell.read_with(cx, |s, _| s.lines().iter().map(|r| r.size).collect());
    assert!(sizes.windows(2).all(|w| w[0] >= w[1]));
    click_on(cx, "ticked-0-box");
    assert_eq!(shell.read_with(cx, |s, _| s.to_free().len()), ticked - lines[0].3);
    // Everything unticked: the list says how to tick.
    while shell.read_with(cx, |s, _| !s.ticked.is_empty()) {
        click_on(cx, "ticked-0-box");
    }
    assert!(shell.read_with(cx, |s, _| s.lines().is_empty()));
    click_on(cx, "tab-suggestions");
    assert!(cx.debug_bounds("suggestion-0").is_some());
}

#[gpui::test]
fn the_settings_choose_the_theme_and_the_language(cx: &mut TestAppContext) {
    let (_root, shell, cx, _config) = open(cx, "reglages", Start::Demo);
    click_on(cx, "settings-button");
    assert!(cx.debug_bounds("settings").is_some());
    // Every theme is offered, after the system's choice.
    assert!(cx.debug_bounds(Box::leak(format!("theme-{}", THEMES.len()).into_boxed_str())).is_some());
    click_on(cx, "theme-4");
    assert_eq!(shell.read_with(cx, |s, _| s.theme.name), THEMES[3].0);
    assert_eq!(settings_file().theme, THEMES[3].0);
    click_on(cx, "theme-0");
    assert_eq!(settings_file().theme, "", "back to following the system");

    click_on(cx, "language-en");
    assert_eq!(tr("English", "Français"), "English");
    assert_eq!(settings_file().language, Language::English);
    assert_eq!(overlay(&shell, cx), Some(Overlay::Settings), "the settings stay open while choosing");
    click_on(cx, "language-system");
    assert_eq!(tr("English", "Français"), "Français", "tests run in French");
    assert_eq!(settings_file().language, Language::System);

    // A click beside the settings closes them, as Escape does; the shortcut opens them again.
    click(cx, point(px(20.), px(20.)));
    assert_eq!(overlay(&shell, cx), None);
    cx.simulate_keystrokes(&format!("{}-,", super::super::keymap::MOD));
    assert_eq!(overlay(&shell, cx), Some(Overlay::Settings));
    cx.simulate_keystrokes("escape");
    assert_eq!(overlay(&shell, cx), None);
    // The help has its button too.
    click_on(cx, "help-button");
    assert_eq!(overlay(&shell, cx), Some(Overlay::Help));
}

#[gpui::test]
fn the_language_chosen_is_spoken_at_the_next_launch(cx: &mut TestAppContext) {
    let (_root, _shell, cx, _config) = open(cx, "langue", Start::Demo);
    click_on(cx, "settings-button");
    click_on(cx, "language-en");
    lang::set_language(Language::System);
    // A new window reads the settings: English again.
    let _ = cx.update(|window, cx| cx.new(|cx| Shell::new(Start::Demo, window, cx)));
    assert_eq!(tr("English", "Français"), "English");
    lang::set_language(Language::System);
    assert_eq!(settings_file().language, Language::English);
}

#[gpui::test]
fn the_last_folder_opens_again_at_launch(cx: &mut TestAppContext) {
    let dir = temp("dernier");
    fs::write(dir.join("gros"), vec![1u8; 2_000_000]).unwrap();
    let (_root, _shell, cx, _config) = open(cx, "dernier-fenetre", Start::Folder(dir.clone()));
    assert_eq!(settings_file().last, Some(Last::Folder(dir.clone())));
    // Launched without arguments: the same folder is scanned again, no start screen.
    let again = cx.update(|window, cx| cx.new(|cx| Shell::new(Start::Choose, window, cx)));
    cx.run_until_parked();
    let source = again.read_with(cx, |s, _| s.disk.as_ref().map(|d| d.source.clone()));
    assert_eq!(source, Some(Source::Folder(dir.clone())));
    // A folder given on the command line wins over the one remembered.
    let demo = cx.update(|window, cx| cx.new(|cx| Shell::new(Start::Demo, window, cx)));
    assert!(demo.read_with(cx, |s, _| s.disk.as_ref().is_some_and(|d| d.source == Source::Demo)));
    assert_eq!(settings_file().last, Some(Last::Demo), "the demo is remembered too");

    // The demo remembered opens again; a folder gone since then leaves the start screen.
    let remembered = cx.update(|window, cx| cx.new(|cx| Shell::new(Start::Choose, window, cx)));
    assert!(remembered.read_with(cx, |s, _| s.disk.as_ref().is_some_and(|d| d.source == Source::Demo)));
    fs::write(platform::config_dir().join("settings"), format!("last={}\n", dir.join("parti").display())).unwrap();
    let gone = cx.update(|window, cx| cx.new(|cx| Shell::new(Start::Choose, window, cx)));
    assert!(gone.read_with(cx, |s, _| matches!(s.stage, Stage::Choose) && s.disk.is_none()));
    fs::remove_dir_all(&dir).unwrap();
}

#[gpui::test]
fn a_click_over_the_window_never_reaches_the_plan_below(cx: &mut TestAppContext) {
    let (_root, shell, cx, _config) = open(cx, "voile", Start::Demo);
    let before = |cx: &mut VisualTestContext| shell.read_with(cx, |s, _| (s.ticked.clone(), s.shown.clone()));
    let start = before(cx);
    click_on(cx, "settings-button");
    // Every theme and every language, each over a plot of the plan.
    for i in 0..=THEMES.len() {
        click_on(cx, Box::leak(format!("theme-{i}").into_boxed_str()));
        assert_eq!(before(cx), start, "theme-{i} reached the plan");
    }
    for language in ["language-en", "language-fr", "language-system"] {
        click_on(cx, language);
        assert_eq!(before(cx), start, "{language} reached the plan");
    }
    // The summary and the help hold the clicks too.
    cx.simulate_keystrokes("escape");
    click_on(cx, "free");
    click_on(cx, "confirm-list");
    assert_eq!((before(cx), overlay(&shell, cx)), (start.clone(), Some(Overlay::Confirm)));
    cx.simulate_keystrokes("escape f1");
    click_on(cx, "help");
    assert_eq!(before(cx), start);
}

#[gpui::test]
fn the_rules_behind_the_suggestions_are_explained(cx: &mut TestAppContext) {
    let (_root, shell, cx, _config) = open(cx, "regles", Start::Demo);
    let start = shell.read_with(cx, |s, _| (s.ticked.clone(), s.shown.clone()));
    // From the link by the lists, and from the legend.
    for opener in ["rules-button", "legend"] {
        click_on(cx, opener);
        assert_eq!(overlay(&shell, cx), Some(Overlay::Rules), "{opener}");
        let sheet = cx.debug_bounds("rules").unwrap();
        assert!(sheet.top() >= px(0.) && sheet.bottom() <= px(900.), "the sheet fits the window");
        click_on(cx, "rules");
        assert_eq!(overlay(&shell, cx), Some(Overlay::Rules), "a click inside keeps it");
        cx.simulate_keystrokes("escape");
        assert_eq!(overlay(&shell, cx), None);
    }
    assert_eq!(shell.read_with(cx, |s, _| (s.ticked.clone(), s.shown.clone())), start, "nothing reached the plan");
}
