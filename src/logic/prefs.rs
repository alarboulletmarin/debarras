//! User preferences (`settings` file): parsed from and written to `key=value` lines.

/// What the user chose; an empty field keeps the default.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Prefs {
    /// Name of the theme; empty: the house theme matching the system's appearance.
    pub theme: String,
}

impl Prefs {
    /// An unknown key, a line without `=` or an older file are ignored: the default applies.
    pub fn parse(text: &str) -> Self {
        let mut prefs = Self::default();
        for (key, value) in text.lines().filter_map(|line| line.split_once('=')) {
            if key.trim() == "theme" {
                prefs.theme = value.trim().into();
            }
        }
        prefs
    }

    pub fn to_text(&self) -> String {
        if self.theme.is_empty() { String::new() } else { format!("theme={}\n", self.theme) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_and_writes_settings() {
        let prefs = Prefs::parse("theme=Nord\nbogus\nold=1\n");
        assert_eq!(prefs.theme, "Nord");
        assert_eq!(Prefs::parse(&prefs.to_text()), prefs);
        for empty in ["", "theme=", "theme", "\n\n", "=Nord"] {
            assert_eq!(Prefs::parse(empty), Prefs::default(), "{empty:?}");
        }
        assert_eq!(Prefs::parse(" theme = Rosé Pine \r\n").theme, "Rosé Pine");
        assert_eq!(Prefs::default().to_text(), "");
    }
}
