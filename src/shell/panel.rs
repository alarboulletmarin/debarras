//! The side panel: the space used, what is ticked to be freed, the legend, the suggestions and the
//! button that frees them, then what opens the settings and the help.

use gpui::{AnyElement, Div, FontWeight, div, pattern_slash, prelude::*, px, uniform_list};

use super::*;
use crate::logic::outline::{Row, outline};
use crate::ui::{
    assets::MONO,
    widgets::{Pattern, button, caps, checkbox, checkbox_some, drag_window, figure, one_line, swatch},
};

pub(super) const PANEL_W: f32 = 360.;
/// Height of a row of the lists: all the same, so that a long list only lays out what shows.
const ROW: f32 = 46.;

impl Shell {
    /// The lines of the list shown, as a tree: the suggestions, or what is ticked (what freeing would delete).
    pub(super) fn lines(&self) -> Vec<Row> {
        let Some(disk) = &self.disk else { return Vec::new() };
        let items: Vec<Vec<usize>> = match self.list {
            List::Suggestions => disk.suggestions.iter().map(|s| s.path.clone()).collect(),
            List::Ticked => self.to_free().into_iter().map(|(path, _)| path).collect(),
        };
        outline(&disk.root, &items, &self.unfolded)
    }

    /// Ticks every item of a line, or unticks them all when they all are.
    fn tick_all(&mut self, items: &[Vec<usize>], cx: &mut Context<Self>) {
        if self.freeing {
            return;
        }
        if items.iter().all(|p| self.ticked.contains(p)) {
            items.iter().for_each(|p| _ = self.ticked.remove(p));
        } else {
            self.ticked.extend(items.iter().cloned());
        }
        self.tick_count += 1;
        self.status = None;
        cx.notify();
    }

    fn render_line(&self, i: usize, row: &Row, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme;
        let paper = t.paper;
        let prefix = if self.list == List::Ticked { "ticked" } else { "suggestion" };
        let ticked = row.items.iter().filter(|p| self.ticked.contains(*p)).count();
        let tick = if ticked == 0 {
            checkbox(false, &t)
        } else if ticked == row.items.len() {
            checkbox(true, &t)
        } else {
            checkbox_some(&t)
        };
        let below = match row.open {
            None => self.disk.as_ref().and_then(|d| d.root.at(&row.path)).and_then(|n| n.why).map_or("", |w| w.text()).to_string(),
            Some(_) => lang::count(row.items.len() as u64, ("item", "items", "élément", "éléments")),
        };
        let items = row.items.clone();
        let line = div()
            .id((prefix, i))
            .debug_selector(move || format!("{prefix}-{i}"))
            .w_full()
            .h(px(ROW))
            .flex()
            .gap(px(8.))
            .pt(px(6.))
            .pl(px(6. + 14. * row.depth as f32))
            .pr(px(6.))
            .cursor_pointer()
            .hover(move |s| s.bg(paper));
        let line = match row.open {
            // An item: the whole line ticks it.
            None => line.child(div().w(px(9.)).flex_none()).child(checkbox_hit(tick, prefix, i)).on_click(cx.listener(move |this, _, _, cx| this.tick_all(&items, cx))),
            // A folder: the line opens it, its box ticks all it holds.
            Some(open) => {
                let path = row.path.clone();
                line.child(div().w(px(9.)).flex_none().pt(px(1.)).text_size(px(8.)).text_color(t.soft).child(if open { "▼" } else { "▶" }))
                    .child(checkbox_hit(tick, prefix, i).on_click(cx.listener(move |this, _, _, cx| {
                        cx.stop_propagation();
                        this.tick_all(&items, cx);
                    })))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if !this.unfolded.remove(&path) {
                            this.unfolded.insert(path.clone());
                        }
                        cx.notify();
                    }))
            }
        };
        line.child(
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
                        .child(one_line(div().min_w_0().text_size(px(13.)).font_weight(FontWeight::MEDIUM)).child(row.label.clone()))
                        .child(div().font_family(MONO).text_size(px(12.)).flex_none().child(lang::size(row.size))),
                )
                .child(one_line(div().text_size(px(11.)).text_color(t.soft)).child(below)),
        )
        .into_any_element()
    }

    pub(super) fn render_panel(&self, _height: f32, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let Some(disk) = &self.disk else { return div() };
        let total = disk.root.size;
        let picked = self.ticked_total();
        let share = if total == 0 { 0. } else { (picked as f64 / total as f64).min(1.) as f32 };
        let rows = std::rc::Rc::new(self.lines());
        let lines = rows.len();
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
            // The legend: a click explains it.
            .child(
                div()
                    .id("legend")
                    .debug_selector(|| "legend".into())
                    .mt(px(14.))
                    .flex()
                    .flex_wrap()
                    .gap(px(16.))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_overlay(Overlay::Rules, cx)))
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
                    .child(tab("tab-ticked", List::Ticked, format!("{} · {}", tr("Ticked", "Coché"), lang::number(ticked as u64))))
                    .child(div().flex_1())
                    .child(small("rules-button", tr("Why?", "Pourquoi ?").to_string()).text_size(px(11.)).on_click(cx.listener(|this, _, _, cx| this.toggle_overlay(Overlay::Rules, cx)))),
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
                                cx.processor(move |this, range: std::ops::Range<usize>, _, cx| range.map(|i| this.render_line(i, &rows[i], cx)).collect::<Vec<_>>()),
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

/// A line's box, which tests click by `{prefix}-{i}-box`.
fn checkbox_hit(tick: Div, prefix: &'static str, i: usize) -> gpui::Stateful<Div> {
    div().id(("box", i)).debug_selector(move || format!("{prefix}-{i}-box")).flex_none().mt(px(2.)).child(tick)
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
