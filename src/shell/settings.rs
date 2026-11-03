//! The settings, over the window: the theme, shown in its own colours, and the language.

use gpui::{FontWeight, WindowAppearance, div, prelude::*, px};

use super::{keymap::keys_of, *};
use crate::{
    lang::Language,
    ui::{
        assets::MONO,
        theme::THEMES,
        widgets::{caps, one_line},
    },
};

impl Shell {
    fn pick_theme(&mut self, name: &str, window: &Window, cx: &mut Context<Self>) {
        self.prefs.theme = name.to_string();
        self.theme = Theme::of(&self.prefs, window.appearance());
        self.save_prefs();
        cx.notify();
    }

    fn pick_language(&mut self, language: Language, cx: &mut Context<Self>) {
        self.prefs.language = language;
        lang::set_language(language);
        self.save_prefs();
        cx.notify();
    }

    pub(super) fn render_settings(&self, (w, h): (f32, f32), cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme;
        let current = self.prefs.theme.as_str();
        // The system's choice first (empty name), then every theme, each drawn in its own colours.
        let names = std::iter::once("").chain(THEMES.iter().map(|(name, _)| *name));
        let tiles = names.enumerate().map(|(i, name)| {
            let own = Theme::of(&Prefs { theme: name.to_string(), ..Prefs::default() }, if t.dark { WindowAppearance::Dark } else { WindowAppearance::Light });
            let on = current == name || (!THEMES.iter().any(|(n, _)| *n == current) && name.is_empty());
            let label = if name.is_empty() { tr("Like the system", "Comme le système").to_string() } else { name.to_string() };
            let accent = t.accent;
            div()
                .id(("theme", i))
                .debug_selector(move || format!("theme-{i}"))
                .h(px(56.))
                .px(px(12.))
                .flex()
                .flex_col()
                .justify_center()
                .gap(px(6.))
                .bg(own.paper)
                .text_color(own.ink)
                .border_1()
                .border_color(if on { t.ink } else { t.rule })
                .when(on, |d| d.border_2())
                .cursor_pointer()
                .hover(move |s| s.border_color(accent))
                .on_click(cx.listener(move |this, _, window, cx| this.pick_theme(name, window, cx)))
                .child(one_line(div().text_size(px(12.5)).font_weight(if on { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })).child(label))
                .child(div().flex().gap(px(4.)).children(own.tints.iter().take(5).map(|c| div().w(px(14.)).h(px(6.)).bg(*c))).child(div().w(px(14.)).h(px(6.)).bg(own.accent)))
        });
        let languages = Language::ALL.into_iter().map(|language| {
            let on = self.prefs.language == language;
            let (ink, panel, accent) = (t.ink, t.panel, t.accent);
            div()
                .id(language.code())
                .debug_selector(move || format!("language-{}", if language.code().is_empty() { "system" } else { language.code() }))
                .h(px(36.))
                .px(px(16.))
                .flex()
                .items_center()
                .text_size(px(13.))
                .cursor_pointer()
                .border_1()
                .border_color(ink.opacity(0.75))
                .when(on, |d| d.bg(ink).text_color(panel))
                .when(!on, |d| d.hover(move |s| s.border_color(accent).text_color(accent)))
                .on_click(cx.listener(move |this, _, _, cx| this.pick_language(language, cx)))
                .child(language.label())
        });
        div()
            .id("settings-veil")
            // Nothing under the veil hears the pointer: a click here never reaches the plan.
            .occlude()
            .absolute()
            .size_full()
            .bg(t.paper.opacity(0.85))
            .flex()
            .items_center()
            .justify_center()
            .on_click(cx.listener(|this, _, _, cx| {
                this.overlay = None;
                cx.notify();
            }))
            .child(
                div()
                    .id("settings")
                    .debug_selector(|| "settings".into())
                    .w(px((w - 64.).min(640.)))
                    .max_h(px(h - 64.))
                    .overflow_y_scroll()
                    .bg(t.panel)
                    .border_1()
                    .border_color(t.ink.opacity(0.85))
                    .p(px(28.))
                    .flex()
                    .flex_col()
                    .gap(px(14.))
                    .on_click(|_, _, cx| cx.stop_propagation())
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_baseline()
                            .child(div().text_size(px(15.)).font_weight(FontWeight::SEMIBOLD).child(tr("Settings", "Réglages")))
                            .child(div().font_family(MONO).text_size(px(11.)).text_color(t.soft).child(format!("débarras {}", env!("CARGO_PKG_VERSION")))),
                    )
                    .child(div().mt(px(8.)).child(caps(&format!("{} · {}", tr("Theme", "Thème"), keys_of(&NextTheme)), &t)))
                    .child(div().grid().grid_cols(3).gap(px(8.)).children(tiles))
                    .child(div().mt(px(14.)).child(caps(tr("Language", "Langue"), &t)))
                    .child(div().flex().flex_wrap().gap(px(8.)).children(languages))
                    .child(
                        div().mt(px(14.)).text_size(px(12.)).text_color(t.soft).child(format!(
                            "{} {}",
                            tr("Débarras opens the last folder again at launch. Every shortcut:", "Débarras rouvre le dernier dossier au lancement. Tous les raccourcis :"),
                            keys_of(&ToggleHelp)
                        )),
                    ),
            )
    }
}
