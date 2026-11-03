//! The page of the user guide that lists every shortcut, `book/src/shortcuts.md`, is written from
//! the help panel's sections: the same lines, in English, with the keys of Linux and Windows.

use super::keymap::help_sections;

const PAGE: &str = "book/src/shortcuts.md";

const HEAD: &str = "# Keyboard shortcuts

<!-- Written by the test `shortcuts_page_is_current` from `help_sections` (src/shell/keymap.rs).
     Do not edit by hand: run `make docs`. -->

Every line of the help panel (`F1`), with the keys of Linux and Windows. On macOS, read `Cmd` for `Ctrl`.
";

/// Text of a table cell: the Markdown signs are escaped.
fn cell(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if matches!(c, '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '>' | '|' | '#') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Keys as code: `F5 / Ctrl+R` gives `F5` / `Ctrl+R`; a gesture in words stays a word.
fn keys(keys: &str) -> String {
    keys.split(" / ").map(|k| if k.contains('+') || k.len() <= 3 || k == "Esc" || k == "Backspace" { format!("`{k}`") } else { cell(k) }).collect::<Vec<_>>().join(" / ")
}

fn page() -> String {
    crate::lang::in_english(|| {
        let mut out = String::from(HEAD);
        for (title, rows) in help_sections() {
            out.push_str(&format!("\n## {title}\n\n| Keys | Action |\n|---|---|\n"));
            for row in rows {
                out.push_str(&format!("| {} | {} |\n", keys(&row.keys), cell(row.effect)));
            }
        }
        out
    })
}

/// `make docs` (`DEBARRAS_WRITE_DOCS=1`) writes the page; otherwise a page that no longer matches
/// the help panel fails, with the command that brings it up to date.
#[cfg(not(target_os = "macos"))]
#[test]
fn shortcuts_page_is_current() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(PAGE);
    let page = page();
    if std::env::var_os("DEBARRAS_WRITE_DOCS").is_some() {
        std::fs::write(&path, &page).unwrap();
        return;
    }
    let written = std::fs::read_to_string(&path).unwrap_or_default().replace("\r\n", "\n");
    assert!(written == page, "{PAGE} no longer matches the help panel: run `make docs`");
}

#[test]
fn shortcuts_page_is_in_english() {
    let page = page();
    for title in ["## The plan", "## Folders", "## The app"] {
        assert!(page.contains(&format!("\n{title}\n")), "{title} missing");
    }
    assert!(page.contains("| `Esc` / `Backspace` |"));
    assert_eq!(crate::lang::tr("Help", "Aide"), "Aide", "the rest of the thread is back in French");
}
