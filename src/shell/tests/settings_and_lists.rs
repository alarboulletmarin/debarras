//! The panel: the list of what is ticked, the settings, and what opens again at the next launch.

use super::*;
use crate::{lang::Language, ui::theme::THEMES};

/// The settings as written, where the app writes them on this system (the environment is isolated).
fn settings_file() -> Prefs {
    Prefs::parse(&fs::read_to_string(platform::config_dir().join("settings")).unwrap_or_default())
}

#[gpui::test]
fn the_ticked_list_recalls_every_tick(cx: &mut TestAppContext) {
    let (_root, shell, cx, _config) = open(cx, "coche", Start::Demo);
    let ticked = shell.read_with(cx, |s, _| s.to_free().len());
    assert!(ticked >= 2);
    click_on(cx, "tab-ticked");
    assert!(cx.debug_bounds(Box::leak(format!("ticked-{}", ticked - 1).into_boxed_str())).is_some(), "every ticked item is listed");
    assert!(cx.debug_bounds(Box::leak(format!("ticked-{ticked}").into_boxed_str())).is_none());
    // Largest first; a click unticks.
    let sizes: Vec<u64> = shell.read_with(cx, |s, _| s.to_free().iter().map(|(_, n)| n.size).collect());
    assert!(sizes.windows(2).all(|w| w[0] >= w[1]));
    click_on(cx, "ticked-0");
    assert_eq!(shell.read_with(cx, |s, _| s.to_free().len()), ticked - 1);
    // Everything unticked: the list says how to tick.
    while shell.read_with(cx, |s, _| !s.ticked.is_empty()) {
        click_on(cx, "ticked-0");
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
    cx.simulate_keystrokes("ctrl-,");
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
