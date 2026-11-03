//! The shortcuts: the default table, the help sections that list them, and their installation.
//! A new shortcut is one line in `defaults` and one in `help_sections`; the help panel and the
//! guide's shortcuts page follow.

use gpui::{Action, App, KeyBinding};

use crate::{actions::*, lang::tr};

/// The key of commands: `Cmd` on macOS, `Ctrl` elsewhere.
const MOD: &str = if cfg!(target_os = "macos") { "cmd" } else { "ctrl" };

/// Every binding, as (keys, action). An action may have several; the first is shown first.
pub fn defaults() -> Vec<(String, Box<dyn Action>)> {
    let b = |keys: &str, action: Box<dyn Action>| (keys.to_string(), action);
    vec![
        b("escape", Box::new(Up)),
        b("backspace", Box::new(Up)),
        b("t", Box::new(NextTheme)),
        b("f1", Box::new(ToggleHelp)),
        b(&format!("{MOD}-o"), Box::new(OpenFolder)),
        b("f5", Box::new(Rescan)),
        b(&format!("{MOD}-r"), Box::new(Rescan)),
        b("o", Box::new(Reveal)),
        b(&format!("{MOD}-q"), Box::new(Quit)),
    ]
}

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys(defaults().into_iter().map(|(keys, action)| KeyBinding::load(&keys, action, None, false, None, &gpui::DummyKeyboardMapper).expect("valid keys")));
}

/// How a binding is written for the user: `ctrl-o` gives `Ctrl+O`, `escape` gives `Esc`.
pub fn shown(keys: &str) -> String {
    keys.split('-')
        .map(|part| match part {
            "ctrl" => "Ctrl".to_string(),
            "cmd" => "Cmd".to_string(),
            "shift" => tr("Shift", "Maj").to_string(),
            "alt" => "Alt".to_string(),
            "escape" => tr("Esc", "Échap").to_string(),
            "backspace" => tr("Backspace", "Retour arrière").to_string(),
            "enter" => tr("Enter", "Entrée").to_string(),
            "space" => tr("Space", "Espace").to_string(),
            key => key.to_uppercase(),
        })
        .collect::<Vec<_>>()
        .join("+")
}

/// Every key of `action`, as shown: `F5 / Ctrl+R`.
pub fn keys_of(action: &dyn Action) -> String {
    defaults().iter().filter(|(_, a)| a.partial_eq(action)).map(|(k, _)| shown(k)).collect::<Vec<_>>().join(" / ")
}

/// A line of the help: the keys or the gesture, then what it does.
pub struct Row {
    pub keys: String,
    pub effect: &'static str,
}

/// The help panel's content, section by section.
pub fn help_sections() -> Vec<(&'static str, Vec<Row>)> {
    let k = |action: &dyn Action, effect: &'static str| Row { keys: keys_of(action), effect };
    let g = |keys: &'static str, effect: &'static str| Row { keys: keys.to_string(), effect };
    vec![
        (
            tr("The plan", "Le plan"),
            vec![
                g(tr("Click", "Clic"), tr("Open a folder; tick or untick a plot that can be freed", "Ouvrir un dossier ; cocher ou décocher une parcelle libérable")),
                k(&Up, tr("Back to the parent folder (also a right click, or the path above the plan)", "Remonter au dossier parent (aussi le clic droit, ou le chemin au-dessus du plan)")),
                k(&Reveal, tr("Show the hovered item in the file manager", "Montrer l'élément survolé dans le gestionnaire de fichiers")),
                g(tr("Hatching", "Hachures"), tr("Can go without loss: rebuilt or downloaded again when needed", "Libérable sans perte : reconstruit ou retéléchargé au besoin")),
                g(tr("Dots", "Pointillés"), tr("Large, to review: you judge", "Volumineux, à examiner : à vous de juger")),
            ],
        ),
        (
            tr("Folders", "Dossiers"),
            vec![
                k(&OpenFolder, tr("Choose a folder to scan (a folder dropped on the window too)", "Choisir un dossier à analyser (un dossier déposé sur la fenêtre aussi)")),
                k(&Rescan, tr("Scan the same folder again", "Analyser de nouveau le même dossier")),
            ],
        ),
        (
            tr("The app", "L'application"),
            vec![k(&NextTheme, tr("Next theme (kept for the next launch)", "Thème suivant (gardé au prochain lancement)")), k(&ToggleHelp, tr("This help", "Cette aide")), k(&Quit, tr("Quit", "Quitter"))],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_keys_for_the_user() {
        assert_eq!(shown("ctrl-o"), "Ctrl+O");
        assert_eq!(shown("escape"), "Échap");
        assert_eq!(shown("f5"), "F5");
        assert_eq!(crate::lang::in_english(|| shown("ctrl-shift-backspace")), "Ctrl+Shift+Backspace");
        assert_eq!(keys_of(&Up), "Échap / Retour arrière");
    }

    #[test]
    fn every_action_is_bound_and_in_the_help() {
        let help: Vec<String> = help_sections().into_iter().flat_map(|(_, rows)| rows).map(|r| r.keys).collect();
        for (keys, action) in defaults() {
            assert!(help.iter().any(|h| h.contains(&shown(&keys))), "{keys} ({}) is not in the help", action.name());
        }
    }
}
