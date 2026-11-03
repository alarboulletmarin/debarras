//! The side panel: the space used, what is ticked to be freed, the legend, the suggestions and the
//! button that frees them, then what opens the settings and the help.

use gpui::{AnyElement, Div, FontWeight, div, pattern_slash, prelude::*, px, uniform_list};

use super::*;
use crate::ui::{
    assets::MONO,
    widgets::{Pattern, button, caps, checkbox, drag_window, figure, one_line, swatch},
};

pub(super) const PANEL_W: f32 = 360.;
/// Height of a row of the lists: all the same, so that a long list only lays out what shows.
const ROW: f32 = 46.;

/// A row of the panel's lists, as the list shows it.
pub(super) struct Line {
    path: Vec<usize>,
    name: String,
    size: u64,
    why: &'static str,
}

impl Shell {
    /// The rows of the list shown: the suggestions, or what is ticked (what freeing would delete).
    pub(super) fn lines(&self) -> Vec<Line> {
        let Some(disk) = &self.disk else { return Vec::new() };
        match self.list {
            List::Suggestions => disk
                .suggestions
                .iter()
                .map(|s| Line { path: s.path.clone(), name: s.name.clone(), size: s.size, why: s.why.map_or("", |w| w.text()) })
                .collect(),
            List::Ticked => self
                .to_free()
                .into_iter()
                .map(|(path, n)| Line { name: disk.root.short_name(&path), size: n.size, why: n.why.map_or("", |w| w.text()), path })
                .collect(),
        }
    }

    fn render_line(&self, i: usize, line: Line, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme;
        let on = self.list == List::Ticked || self.ticked.contains(&line.path);
        let paper = t.paper;
        let prefix = if self.list == List::Ticked { "ticked" } else { "suggestion" };
        let path = line.path;
        div()
            .id((prefix, i))
            .debug_selector(move || format!("{prefix}-{i}"))
            .w_full()
            .h(px(ROW))
            .flex()
            .gap(px(10.))
            .pt(px(6.))
            .px(px(6.))
            .cursor_pointer()
            .hover(move |s| s.bg(paper))
            .on_click(cx.listener(move |this, _, _, cx| this.toggle(path.clone(), cx)))
            .child(checkbox(on, &t).mt(px(2.)))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .gap(px(2.))
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .gap(px(8.))
                            .child(one_line(div().min_w_0().text_size(px(13.)).font_weight(FontWeight::MEDIUM)).child(line.name))
                            .child(div().font_family(MONO).text_size(px(12.)).flex_none().child(lang::size(line.size))),
                    )
                    .child(one_line(div().text_size(px(11.)).text_color(t.soft)).child(line.why)),
            )
            .into_any_element()
    }

    pub(super) fn render_panel(&self, _height: f32, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let Some(disk) = &self.disk else { return div() };
        let total = disk.root.size;
        let picked = self.ticked_total();
        let share = if total == 0 { 0. } else { (picked as f64 / total as f64).min(1.) as f32 };
        let lines = self.lines().len();
        let ticked = self.to_free().len();
        let legend = |pattern: Pattern, text: &'static str| div().flex().items_center().gap(px(6.)).child(swatch(pattern, &t)).child(div().text_size(px(11.5)).text_color(t.soft).child(text));
        let source = match &disk.source {
            Source::Demo => tr("demo, made up", "démo, inventée").to_string(),
            Source::Folder(_) => {
                let mut s = format!("{} · {}", lang::count(disk.files, ("file", "files", "fichier", "fichiers")), format_took(disk.took));
                if disk.errors > 0 {
                    s.push_str(&format!(" · {} {}", lang::number(disk.errors), tr("unreadable", if disk.errors > 1 { "illisibles" } else { "illisible" })));
                }
                s
            }
        };
        let tab = |id: &'static str, list: List, text: String| {
            let on = self.list == list;
            let (ink, soft, accent) = (t.ink, t.soft, t.accent);
            div()
                .id(id)
                .debug_selector(move || id.into())
                .pb(px(4.))
                .font_family(MONO)
                .text_size(px(10.5))
                .text_color(if on { ink } else { soft })
                .border_b_1()
                .border_color(if on { ink } else { gpui::transparent_black() })
                .cursor_pointer()
                .hover(move |s| s.text_color(accent))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.list = list;
                    cx.notify();
                }))
                .child(text.to_uppercase())
        };
        let empty = match self.list {
            List::Suggestions => tr("Nothing the rules know how to free here.", "Rien ici que les règles sachent libérer."),
            List::Ticked => tr("Nothing ticked. Click a hatched or dotted plot, or a suggestion.", "Rien de coché. Cliquez une parcelle hachurée ou pointillée, ou une suggestion."),
        };
        let small = |id: &'static str, text: String| {
            let (soft, accent) = (t.soft, t.accent);
            div().id(id).debug_selector(move || id.into()).text_size(px(12.)).text_color(soft).cursor_pointer().hover(move |s| s.text_color(accent)).child(text)
        };
        let free = if self.freeing {
            button("free", tr("Freeing…", "Libération en cours…"), false, &t).opacity(0.6)
        } else {
            button("free", format!("{} · {}", tr("Free", "Libérer"), lang::size(picked)), true, &t).on_click(cx.listener(|this, _, _, cx| this.ask_free(cx)))
        };

        div()
            .absolute()
            .left_0()
            .top_0()
            .w(px(PANEL_W))
            .h_full()
            .bg(t.panel)
            .border_r_1()
            .border_color(t.rule)
            .px(px(32.))
            .pt(px(36.))
            .pb(px(20.))
            .flex()
            .flex_col()
            .child(
                drag_window(div().id("panel-head").flex().justify_between().items_baseline())
                    .child(div().text_size(px(15.)).font_weight(FontWeight::SEMIBOLD).child("débarras"))
                    .child(div().font_family(MONO).text_size(px(11.)).text_color(t.soft).child(tr("disk plan", "plan du disque"))),
            )
            .child(div().mt(px(36.)).child(caps(tr("Used", "Occupé"), &t)))
            .child(div().mt(px(6.)).child(figure(total, 60., &t)))
            .child(div().mt(px(2.)).font_family(MONO).text_size(px(11.)).text_color(t.soft).truncate().child(source))
            .child(div().mt(px(22.)).child(caps(tr("Ticked to free", "Coché pour libérer"), &t)))
            .child(div().mt(px(6.)).child(figure(picked, 40., &t)))
            .child(
                div()
                    .mt(px(12.))
                    .h(px(8.))
                    .w_full()
                    .border_1()
                    .border_color(t.ink.opacity(0.6))
                    .child(div().h_full().w(gpui::relative(share)).bg(pattern_slash(t.ink.opacity(0.5), 1., 4.))),
            )
            .child(div().mt(px(6.)).flex().justify_end().font_family(MONO).text_size(px(11.)).text_color(t.soft).child(format!("{} {}", lang::share(picked, total), tr("of the total", "du total"))))
            .child(
                div()
                    .mt(px(14.))
                    .flex()
                    .flex_wrap()
                    .gap(px(18.))
                    .child(legend(Pattern::Hatch, tr("without loss", "sans perte")))
                    .child(legend(Pattern::Dots, tr("to review", "à examiner")))
                    .child(legend(Pattern::None, tr("to keep", "à garder"))),
            )
            .child(
                div()
                    .mt(px(24.))
                    .mb(px(6.))
                    .flex()
                    .gap(px(18.))
                    .child(tab("tab-suggestions", List::Suggestions, format!("{} · {}", tr("Suggestions", "Suggestions"), lang::number(disk.suggestions.len() as u64))))
                    .child(tab("tab-ticked", List::Ticked, format!("{} · {}", tr("Ticked", "Coché"), lang::number(ticked as u64)))),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .mx(px(-6.))
                    .mb(px(10.))
                    .when(lines == 0, |d| d.px(px(6.)).pt(px(6.)).child(div().text_size(px(12.)).text_color(t.soft).child(empty)))
                    .when(lines > 0, |d| {
                        d.child(
                            uniform_list(
                                "lines",
                                lines,
                                cx.processor(|this, range: std::ops::Range<usize>, _, cx| {
                                    let lines = this.lines();
                                    lines.into_iter().enumerate().skip(range.start).take(range.len()).map(|(i, line)| this.render_line(i, line, cx)).collect::<Vec<_>>()
                                }),
                            )
                            .track_scroll(self.lines_scroll.clone())
                            .size_full(),
                        )
                    }),
            )
            .when_some(self.status.clone(), |d, s| d.child(div().debug_selector(|| "status".into()).mb(px(10.)).text_size(px(11.5)).text_color(t.soft).child(s)))
            .child(free)
            .child(
                div()
                    .mt(px(14.))
                    .flex()
                    .gap(px(20.))
                    .child(small("open-button", tr("Open…", "Ouvrir…").to_string()).on_click(cx.listener(|this, _, window, cx| this.choose_folder(window, cx))))
                    .child(small("settings-button", tr("Settings", "Réglages").to_string()).on_click(cx.listener(|this, _, _, cx| this.toggle_overlay(Overlay::Settings, cx))))
                    .child(small("help-button", tr("Help", "Aide").to_string()).on_click(cx.listener(|this, _, _, cx| this.toggle_overlay(Overlay::Help, cx)))),
            )
    }
}

/// How long the scan took: `0.4 s`, `12 s`, `3 min 05 s`.
pub(super) fn format_took(took: Duration) -> String {
    let secs = took.as_secs_f32();
    if secs < 9.95 {
        return format!("{secs:.1}\u{a0}s").replace('.', tr(".", ","));
    }
    // Rounded first: 59.6 s is a minute.
    let whole = secs.round() as u64;
    if whole < 60 { format!("{whole}\u{a0}s") } else { format!("{}\u{a0}min {:02}\u{a0}s", whole / 60, whole % 60) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_how_long_the_scan_took() {
        let said: Vec<String> = [0, 400, 9_949, 9_960, 59_400, 59_600, 185_000, 7_200_000].map(|ms| format_took(Duration::from_millis(ms)).replace('\u{a0}', " ")).into();
        assert_eq!(said, ["0,0 s", "0,4 s", "9,9 s", "10 s", "59 s", "1 min 00 s", "3 min 05 s", "120 min 00 s"]);
    }
}
