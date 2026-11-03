//! Freeing for real: the summary, the deletion on disk, what could not be deleted.

use super::*;

/// A folder with what the rules free: two build outputs, a cache holding a model folder, and a film to keep.
fn cluttered(name: &str) -> PathBuf {
    // Not `temp(name)`: `open` empties that one for the settings.
    let dir = temp(&format!("{name}-disque"));
    for (path, size) in [
        ("appli/node_modules/paquet/index.js", 3_000_000),
        ("outil/target/release/outil", 2_000_000),
        ("outil/src/main.rs", 1_500_000),
        (".cache/huggingface/modele.bin", 4_000_000),
        ("films/vacances été 🌞.mkv", 5_000_000),
    ] {
        fs::create_dir_all(dir.join(path).parent().unwrap()).unwrap();
        fs::write(dir.join(path), vec![1u8; size]).unwrap();
    }
    dir
}

fn exists(dir: &Path, path: &str) -> bool {
    dir.join(path).exists()
}

#[gpui::test]
fn what_is_ticked_is_deleted_for_good_after_the_summary(cx: &mut TestAppContext) {
    let dir = cluttered("liberer");
    let (_root, shell, cx, _config) = open(cx, "liberer", Start::Folder(dir.clone()));
    // Ticked from the start: what goes without loss. The model folder inside the cache is ticked too,
    // by hand: it is inside a ticked folder, so it counts once and is deleted once.
    let model = shell.read_with(cx, |s, _| {
        let root = &s.disk.as_ref().unwrap().root;
        root.find_item(&[".cache".into(), "huggingface".into()])
    });
    shell.update(cx, |s, cx| s.toggle(model, cx));
    let (items, picked, total) = shell.read_with(cx, |s, _| (s.to_free().len(), s.ticked_total(), s.disk.as_ref().unwrap().root.size));
    assert_eq!(items, 3, "node_modules, target and .cache; the model is inside .cache");

    // Cancel: nothing moves.
    click_on(cx, "free");
    assert!(cx.debug_bounds("confirm").is_some());
    click_on(cx, "confirm-cancel");
    assert_eq!(overlay(&shell, cx), None);
    assert!(exists(&dir, "appli/node_modules"));
    // Escape closes the summary too.
    click_on(cx, "free");
    assert_eq!(overlay(&shell, cx), Some(Overlay::Confirm));
    cx.simulate_keystrokes("escape");
    assert!(overlay(&shell, cx).is_none() && exists(&dir, "appli/node_modules"));

    click_on(cx, "free");
    click_on(cx, "confirm-delete");
    cx.run_until_parked();
    for gone in ["appli/node_modules", "outil/target", ".cache"] {
        assert!(!exists(&dir, gone), "{gone} is still there");
    }
    for kept in ["outil/src/main.rs", "films/vacances été 🌞.mkv"] {
        assert!(exists(&dir, kept), "{kept} was deleted");
    }
    let (after, ticked, freeing, status) = shell.read_with(cx, |s, _| (s.disk.as_ref().unwrap().root.size, s.ticked.len(), s.freeing, s.status.clone().unwrap()));
    assert_eq!((after, ticked, freeing), (total - picked, 0, false));
    assert!(status.starts_with("Supprimé : 3 éléments"), "{status}");
    fs::remove_dir_all(&dir).unwrap();
}

#[gpui::test]
fn what_could_not_be_deleted_stays_ticked_and_is_said(cx: &mut TestAppContext) {
    let dir = cluttered("echec");
    let (_root, shell, cx, _config) = open(cx, "echec", Start::Folder(dir.clone()));
    // Gone behind Débarras's back, between the scan and the click.
    fs::remove_dir_all(dir.join("outil/target")).unwrap();
    // Opened inside `outil`: the folder shown stays shown after the plan changes.
    let outil = shell.read_with(cx, |s, _| s.disk.as_ref().unwrap().root.find(&["outil".into()]));
    shell.update(cx, |s, cx| s.open(outil, cx));
    click_on(cx, "free");
    click_on(cx, "confirm-delete");
    cx.run_until_parked();
    let (status, ticked, shown) = shell.read_with(cx, |s, _| {
        let root = &s.disk.as_ref().unwrap().root;
        let ticked: Vec<String> = s.ticked.iter().map(|p| root.short_name(p)).collect();
        (s.status.clone().unwrap(), ticked, s.shown_node().unwrap().name.clone())
    });
    assert!(status.contains("1 élément n'a pas pu l'être : outil/target — introuvable"), "{status}");
    assert_eq!(ticked, ["outil/target"], "the failure is still ticked, found again by its name");
    assert_eq!(shown, "outil");
    assert!(!exists(&dir, "appli/node_modules"), "the others were deleted");
    fs::remove_dir_all(&dir).unwrap();
}

#[gpui::test]
fn the_summary_lists_every_item_however_many(cx: &mut TestAppContext) {
    let (_root, shell, cx, _config) = open(cx, "recap", Start::Demo);
    // A thousand ticked caches: the summary scrolls, the lists only lay out the rows they show.
    let many = Node::dir(
        "/beaucoup",
        crate::logic::tree::Category::Other,
        (0..1_000).map(|i| Node::dir(format!("p{i:04}"), crate::logic::tree::Category::Dev, vec![Node::file("node_modules", 2_000_000 + i, crate::logic::tree::Category::Dev).with(Reclaim::Safe, crate::logic::rules::Why::NodeModules)])).collect(),
    );
    shell.update(cx, |s, cx| s.set_disk(Disk::new(many, Source::Demo, 1_000, 0, Duration::ZERO), cx));
    cx.run_until_parked();
    assert_eq!(shell.read_with(cx, |s, _| s.to_free().len()), 1_000);
    assert!(cx.debug_bounds("suggestion-0").is_some());
    assert!(cx.debug_bounds("suggestion-999").is_none(), "only the rows in view are drawn");
    click_on(cx, "free");
    let list = cx.debug_bounds("confirm-list").unwrap();
    let dialog = cx.debug_bounds("confirm").unwrap();
    assert!(list.bottom() <= dialog.bottom() && dialog.bottom() <= px(900.), "the list scrolls inside the window");
    for button in ["confirm-cancel", "confirm-trash", "confirm-delete"] {
        let b = cx.debug_bounds(button).unwrap_or_else(|| panic!("{button}"));
        assert!(b.bottom() <= dialog.bottom(), "{button} is cut off");
    }
    // The smallest window: the summary and its buttons still fit, side by side.
    cx.simulate_resize(size(px(900.), px(600.)));
    cx.run_until_parked();
    cx.simulate_mouse_move(point(px(1.), px(1.)), None, Modifiers::none());
    let dialog = cx.debug_bounds("confirm").unwrap();
    assert!(dialog.bottom() <= px(600.) && dialog.right() <= px(900.), "{dialog:?}");
    let (cancel, delete) = (cx.debug_bounds("confirm-cancel").unwrap(), cx.debug_bounds("confirm-delete").unwrap());
    assert!(delete.bottom() <= dialog.bottom() && cancel.origin.y == delete.origin.y, "{cancel:?} {delete:?}");
}

#[gpui::test]
fn the_trash_is_not_moved_to_the_trash(cx: &mut TestAppContext) {
    let dir = temp("corbeille-disque");
    fs::create_dir_all(dir.join(".local/share/Trash/files")).unwrap();
    fs::write(dir.join(".local/share/Trash/files/vieux.iso"), vec![1u8; 2_000_000]).unwrap();
    let (_root, shell, cx, _config) = open(cx, "corbeille", Start::Folder(dir.clone()));
    assert_eq!(shell.read_with(cx, |s, _| s.to_free().len()), 1);
    click_on(cx, "free");
    click_on(cx, "confirm-trash");
    cx.run_until_parked();
    let (status, ticked) = shell.read_with(cx, |s, _| (s.status.clone().unwrap(), s.ticked.len()));
    assert!(status.contains("c'est la corbeille"), "{status}");
    assert_eq!(ticked, 1, "still ticked, to be deleted for good");
    assert!(dir.join(".local/share/Trash/files/vieux.iso").exists());
    // Deleting it for good empties it.
    click_on(cx, "free");
    click_on(cx, "confirm-delete");
    cx.run_until_parked();
    assert!(!dir.join(".local/share/Trash").exists());
    fs::remove_dir_all(&dir).unwrap();
}
