//! User preferences (`settings` file): parsed from and written to `key=value` lines.

use std::path::PathBuf;

use crate::lang::Language;

/// What the user chose; an empty field keeps the default.
#[derive(Clone, Debug, PartialEq)]
pub struct Prefs {
    /// Name of the theme; empty: the house theme matching the system's appearance.
    pub theme: String,
    pub language: Language,
    /// What was shown last, opened again at the next launch.
    pub last: Option<Last>,
}

/// What a launch without arguments opens again.
#[derive(Clone, Debug, PartialEq)]
pub enum Last {
    Demo,
    Folder(PathBuf),
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs { theme: String::new(), language: Language::System, last: None }
    }
}

impl Prefs {
    /// An unknown key, a line without `=` or an older file are ignored: the default applies.
    pub fn parse(text: &str) -> Self {
        let mut prefs = Self::default();
        for (key, value) in text.lines().filter_map(|line| line.split_once('=')) {
            let value = value.trim();
            match key.trim() {
                "theme" => prefs.theme = value.into(),
                "language" => prefs.language = Language::from_code(value),
                _ => {}
            }
        }
        // A folder is read whole: its name may start or end with spaces.
        prefs.last = match line_value(text, "last") {
            None | Some("") => None,
            Some("demo") => Some(Last::Demo),
            Some(path) => Some(Last::Folder(path.into())),
        };
        prefs
    }

    pub fn to_text(&self) -> String {
        let mut text = String::new();
        if !self.theme.is_empty() {
            text += &format!("theme={}\n", self.theme);
        }
        if self.language != Language::System {
            text += &format!("language={}\n", self.language.code());
        }
        match &self.last {
            Some(Last::Demo) => text += "last=demo\n",
            // A path that cannot be written on one line is not remembered.
            Some(Last::Folder(path)) if path.is_absolute() && !path.to_string_lossy().contains(['\n', '\r']) => text += &format!("last={}\n", path.display()),
            _ => {}
        }
        text
    }
}

/// The value of `key` exactly as written after `=`, the last time it is given.
fn line_value<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines().rev().filter_map(|line| line.split_once('=')).find(|(k, _)| k.trim() == key).map(|(_, v)| v)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An absolute path on every system: `/home` is not one on Windows, `C:/home` is.
    fn abs(path: &str) -> String {
        if cfg!(windows) { format!("C:{path}") } else { path.to_string() }
    }

    #[test]
    fn reads_and_writes_settings() {
        let prefs = Prefs::parse(&format!("theme=Nord\nbogus\nold=1\nlanguage=en\nlast={}\n", abs("/home/moi/projets été")));
        assert_eq!(prefs.theme, "Nord");
        assert_eq!(prefs.language, Language::English);
        assert_eq!(prefs.last, Some(Last::Folder(abs("/home/moi/projets été").into())));
        assert_eq!(Prefs::parse(&prefs.to_text()), prefs);
        for empty in ["", "theme=", "theme", "\n\n", "=Nord", "language=", "language=klingon", "last="] {
            assert_eq!(Prefs::parse(empty), Prefs::default(), "{empty:?}");
        }
        assert_eq!(Prefs::parse(" theme = Rosé Pine \r\n").theme, "Rosé Pine");
        assert_eq!(Prefs::default().to_text(), "");
    }

    #[test]
    fn remembers_the_last_folder_as_it_is_named() {
        let at = |path: &str| Prefs { last: Some(Last::Folder(abs(path).into())), ..Prefs::default() };
        // Spaces at either end, `=`, `|`, `\`, quotes and emoji come back unchanged.
        for path in ["/home/moi/ dossier ", "/a=b/c|d\\e \"f\" 🌞", "/"] {
            assert_eq!(Prefs::parse(&at(path).to_text()), at(path), "{path:?}");
        }
        assert_eq!(Prefs::parse("last=demo\r\n").last, Some(Last::Demo));
        assert_eq!(Prefs::parse("last=/a\nlast=/b\n").last, Some(Last::Folder("/b".into())), "the last line wins");
        // Not remembered: a relative path, or one with a line break.
        assert_eq!(Prefs { last: Some(Last::Folder("relative/path".into())), ..Prefs::default() }.to_text(), "");
        assert_eq!(at("/a\nb").to_text(), "");
    }
}
