//! Colours: the themes, how one is picked from the preferences and the system appearance, the tint of
//! each family of files, and the contrast rules (WCAG) that keep every text readable.
//!
//! The look is an architect's plan: plots drawn in ink on warm paper, hatching on what can go
//! without loss, dots on what to review, and one red for the hovered plot.

use gpui::{Hsla, WindowAppearance, rgb};

use crate::logic::{prefs::Prefs, tree::Category};

/// Offered themes: (name, [paper, ink, faint, accent, panel, rule]). The first two are the app's
/// own, dark and light, which follow the system.
pub const THEMES: &[(&str, [u32; 6])] = &[
    ("Débarras Dark", [0x1a1917, 0xece7dc, 0x9a948a, 0xf0643a, 0x201f1c, 0x3a3732]),
    ("Débarras Light", [0xf1ede4, 0x1d1c1a, 0x6e6a62, 0xd4471f, 0xf6f3ec, 0xcfc8ba]),
    ("Dracula", [0x282a36, 0xf8f8f2, 0x6272a4, 0xff79c6, 0x343746, 0x44475a]),
    ("One Dark", [0x282c34, 0xabb2bf, 0x5c6370, 0xe06c75, 0x2c313a, 0x3e4451]),
    ("Gruvbox Dark", [0x282828, 0xebdbb2, 0x928374, 0xfb4934, 0x3c3836, 0x504945]),
    ("Nord", [0x2e3440, 0xd8dee9, 0x616e88, 0xbf616a, 0x3b4252, 0x4c566a]),
    ("Catppuccin Mocha", [0x1e1e2e, 0xcdd6f4, 0x6c7086, 0xf38ba8, 0x313244, 0x45475a]),
    ("Tokyo Night", [0x1a1b26, 0xc0caf5, 0x565f89, 0xf7768e, 0x24283b, 0x3b4261]),
    ("Rosé Pine", [0x191724, 0xe0def4, 0x6e6a86, 0xeb6f92, 0x26233a, 0x403d52]),
    ("Solarized Dark", [0x002b36, 0x93a1a1, 0x586e75, 0xdc322f, 0x073642, 0x0a4856]),
    ("Solarized Light", [0xfdf6e3, 0x586e75, 0x93a1a1, 0xdc322f, 0xfffbf0, 0xd9d2c2]),
    ("Catppuccin Latte", [0xeff1f5, 0x4c4f69, 0x9ca0b0, 0xd20f39, 0xffffff, 0xccd0da]),
    ("Gruvbox Light", [0xfbf1c7, 0x3c3836, 0x928374, 0xcc241d, 0xf9f5d7, 0xd5c4a1]),
];

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Theme {
    pub name: &'static str,
    pub dark: bool,
    /// The plan's ground.
    pub paper: Hsla,
    /// The side panel.
    pub panel: Hsla,
    /// Text and outlines.
    pub ink: Hsla,
    /// Secondary text that stays readable (4.5:1) on the paper, the panel and every tint.
    pub soft: Hsla,
    /// Thin separations.
    pub rule: Hsla,
    /// The hovered plot and the hovered button: drawn, so 3:1 on the paper and the panel.
    pub accent: Hsla,
    /// The word « Safe to remove » and its tag's outline; never a fill. Blue-green, off the red/green axis.
    pub safe: Hsla,
    /// The word « Check first » and its tag's outline; never a fill. Ochre.
    pub review: Hsla,
    /// Only for what cannot be undone (deleting for good): never the hover's colour.
    pub danger: Hsla,
    /// The tint of each family, in the order of `Category::ALL`.
    pub tints: [Hsla; Category::ALL.len()],
}

/// The hue and saturation of each family, in the order of `Category::ALL`: system, apps, development,
/// media, documents, caches, archives, models, other. Muted: a plan, not a chart.
const FAMILIES: [(f32, f32); Category::ALL.len()] =
    [(0.08, 0.06), (0.54, 0.3), (0.25, 0.3), (0.09, 0.4), (0.7, 0.2), (0.13, 0.5), (0.03, 0.35), (0.8, 0.2), (0.1, 0.0)];

impl Theme {
    /// The chosen theme; failing that (absent, unknown), the app's own matching the system appearance.
    pub fn of(prefs: &Prefs, appearance: WindowAppearance) -> Self {
        let dark = matches!(appearance, WindowAppearance::Dark | WindowAppearance::VibrantDark);
        let system = &THEMES[if dark { 0 } else { 1 }];
        let (name, colors) = THEMES.iter().find(|t| t.0 == prefs.theme).unwrap_or(system);
        let [paper, ink, faint, accent, panel, rule] = colors.map(|c| Hsla::from(rgb(c)));
        let dark = paper.l < 0.5;
        // A tint is the family's hue laid thinly over the paper, pulled back toward it while the ink reads
        // too weakly, but never down to the paper itself: every plot stands out. The ink is made legible after.
        let tints = FAMILIES.map(|(h, s)| {
            let pure = Hsla { h, s: s.max(0.08), l: if dark { 0.6 } else { 0.5 }, a: 1. };
            (0..=14).map(|i| blend(pure, paper, 0.2 - i as f32 * 0.01)).find(|t| contrast(ink, *t) >= 6.).unwrap_or_else(|| blend(pure, paper, 0.06))
        });
        let grounds: Vec<Hsla> = [paper, panel].into_iter().chain(tints).collect();
        let ink = grounds.iter().fold(ink, |c, g| legible(c, *g));
        // The faintest grey between ink and paper that still reads everywhere it is written.
        let soft = (0..=20)
            .map(|i| blend(faint, ink, 1. - i as f32 * 0.05))
            .find(|c| grounds.iter().all(|g| contrast(*c, *g) >= 4.5))
            .unwrap_or(ink);
        let accent = [paper, panel].iter().fold(accent, |c, g| reach(c, *g, 3.));
        // The house values on the house themes; on the others, the same hues made legible on every ground.
        let [safe, review, danger] = (if dark { [0x86c5c4, 0xe3ad5f, 0xff8a7a] } else { [0x2c6a70, 0x7a4f0f, 0xb3261e] })
            .map(|c| grounds.iter().fold(Hsla::from(rgb(c)), |c, g| legible(c, *g)));
        Self { name, dark, paper, panel, ink, soft, rule, accent, safe, review, danger, tints }
    }

    pub fn tint(&self, category: Category) -> Hsla {
        self.tints[Category::ALL.iter().position(|c| *c == category).unwrap_or(Category::ALL.len() - 1)]
    }

    /// The theme after this one in `THEMES`, for the theme key.
    pub fn next_name(&self) -> &'static str {
        let i = THEMES.iter().position(|t| t.0 == self.name).unwrap_or(0);
        THEMES[(i + 1) % THEMES.len()].0
    }
}

/// Relative luminance of a colour, in the WCAG sense.
fn luminance(color: Hsla) -> f32 {
    let c = color.to_rgb();
    let linear = |x: f32| if x <= 0.03928 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) };
    0.2126 * linear(c.r) + 0.7152 * linear(c.g) + 0.0722 * linear(c.b)
}

/// Contrast ratio of two colours (WCAG): at least 4.5 for text, 3 for strokes.
pub fn contrast(a: Hsla, b: Hsla) -> f32 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// `color`, moved just enough to read on `bg` as text; it keeps its hue.
pub fn legible(color: Hsla, bg: Hsla) -> Hsla {
    reach(color, bg, 4.5)
}

/// `color`, darkened on a light background or lightened on a dark one, until its contrast with `bg` reaches `ratio`.
pub fn reach(color: Hsla, bg: Hsla, ratio: f32) -> Hsla {
    let step = if luminance(bg) > 0.18 { -0.02 } else { 0.02 };
    let mut color = color;
    while contrast(color, bg) < ratio && (0.0..=1.0).contains(&(color.l + step)) {
        color.l += step;
    }
    color
}

/// `top` laid over `under` with opacity `share`, opaque.
pub fn blend(top: Hsla, under: Hsla, share: f32) -> Hsla {
    let (a, b) = (top.to_rgb(), under.to_rgb());
    let mix = |x: f32, y: f32| x * share + y * (1. - share);
    Hsla::from(gpui::Rgba { r: mix(a.r, b.r), g: mix(a.g, b.g), b: mix(a.b, b.b), a: 1. })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn every_theme() -> impl Iterator<Item = Theme> {
        THEMES.iter().map(|(name, _)| Theme::of(&Prefs { theme: name.to_string(), ..Prefs::default() }, WindowAppearance::Light))
    }

    /// In each theme every text reads (4.5:1) where it is written: names on the paper, the panel and every tint;
    /// the hover stroke stands out (3:1); and every plot stands out from the paper.
    #[test]
    fn every_theme_reads() {
        for t in every_theme() {
            let name = t.name;
            for (ground, what) in [(t.paper, "paper"), (t.panel, "panel")].into_iter().chain(t.tints.iter().map(|g| (*g, "tint"))) {
                assert!(contrast(t.ink, ground) >= 4.5, "{name}: ink on {what} {:.2}", contrast(t.ink, ground));
                assert!(contrast(t.soft, ground) >= 4.5, "{name}: soft on {what} {:.2}", contrast(t.soft, ground));
                for (word, c) in [("safe", t.safe), ("review", t.review), ("danger", t.danger)] {
                    assert!(contrast(c, ground) >= 4.5, "{name}: {word} on {what} {:.2}", contrast(c, ground));
                }
            }
            assert!(contrast(t.accent, t.paper) >= 3. && contrast(t.accent, t.panel) >= 3., "{name}: accent");
            assert!(t.danger != t.accent && t.safe != t.review, "{name}: a hover looks like a danger");
            assert!(contrast(t.paper, t.panel) >= 1.02, "{name}: panel and paper look the same");
            for (i, tint) in t.tints.iter().enumerate().take(Category::ALL.len() - 1) {
                assert!(*tint != t.paper, "{name}: family {i} has no tint");
            }
        }
    }

    #[test]
    fn picks_the_theme() {
        let dark = Theme::of(&Prefs::default(), WindowAppearance::Dark);
        let light = Theme::of(&Prefs::default(), WindowAppearance::Light);
        assert_eq!((dark.name, dark.dark, light.name, light.dark), ("Débarras Dark", true, "Débarras Light", false));
        // An unknown name, from an older version or a typo, falls back on the system's.
        assert_eq!(Theme::of(&Prefs { theme: "Inconnu".into(), ..Prefs::default() }, WindowAppearance::Dark).name, "Débarras Dark");
        assert_eq!(Theme::of(&Prefs { theme: "Nord".into(), ..Prefs::default() }, WindowAppearance::Light).name, "Nord");
        // The theme key goes round every theme and back.
        let mut name = light.name;
        for _ in 0..THEMES.len() {
            name = Theme::of(&Prefs { theme: name.into(), ..Prefs::default() }, WindowAppearance::Light).next_name();
        }
        assert_eq!(name, light.name);
    }
}
