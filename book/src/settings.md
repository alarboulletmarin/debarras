# Settings and appearance

## Themes

Press `T` to switch to the next theme. Débarras remembers it for the next launch.

The two house themes, **Débarras Light** and **Débarras Dark**, follow your system's appearance until you choose a theme. The others are: Dracula, One Dark, Gruvbox Dark, Nord, Catppuccin Mocha, Tokyo Night, Rosé Pine, Solarized Dark, Solarized Light, Catppuccin Latte and Gruvbox Light.

Every theme is checked by a test: text reads at a contrast of at least 4.5:1 on every ground, including every family tint, and the hover red stands out at 3:1.

## Language

Débarras speaks English and French. It follows your system:

- from a terminal, the first of `LC_ALL`, `LC_MESSAGES` and `LANG` that is set (`fr…` gives French);
- on macOS and Windows, launched from the Finder or the Start menu, the system's language.

To force a language from a terminal:

```bash
LANG=en_US.UTF-8 debarras
LANG=fr_FR.UTF-8 debarras
```

## The settings file

Débarras writes one small text file, `settings`, only when you change the theme:

| System | Folder |
|---|---|
| Linux | `$XDG_CONFIG_HOME/debarras`, or `~/.config/debarras` |
| macOS | `~/Library/Application Support/debarras` |
| Windows | `%APPDATA%\debarras` |

It holds one line per setting:

```
theme=Nord
```

An unknown line is ignored, and a missing or empty file means the defaults. Deleting the folder resets Débarras.
