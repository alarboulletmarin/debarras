//! The side panel: the space used, what is ticked to be freed, the legend, the suggestions and the
//! button that frees them.

use gpui::{AnyElement, Div, FontWeight, div, pattern_slash, prelude::*, px};

use super::*;
use crate::ui::{
    assets::MONO,
    widgets::{Pattern, button, caps, drag_window, figure, swatch},
};

pub(super) const PANEL_W: f32 = 360.;
/// The suggestions listed; the others are counted in the heading.
const LISTED: usize = 8;

impl Shell {
    pub(super) fn render_panel(&self, _height: f32, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let Some(disk) = &self.disk else { return div() };
        let total = disk.root.size;
        let picked = self.ticked_total();
        let share = if total == 0 { 0. } else { (picked as f64 / total as f64).min(1.) as f32 };
        let count = disk.suggestions.len();
        let rows: Vec<AnyElement> = disk
            .suggestions
            .iter()
            .take(LISTED)
            .enumerate()
            .map(|(i, item)| {
                let path = item.path.clone();
                let on = self.ticked.contains(&path);
                let name = item.name.clone();
                let review = item.reclaim == Reclaim::Review;
                let ink = t.ink;
                let paper = t.paper;
                div()
                    .id(("suggestion", i))
                    .debug_selector(move || format!("suggestion-{i}"))
                    .flex()
                    .gap(px(10.))
                    .py(px(6.))
                    .px(px(6.))
                    .mx(px(-6.))
                    .cursor_pointer()
                    .hover(move |s| s.bg(paper))
                    .on_click(cx.listener(move |this, _, _, cx| this.toggle(path.clone(), cx)))
                    .child(div().mt(px(3.)).size(px(12.)).flex_none().border_1().border_color(ink.opacity(0.75)).when(on, |d| d.bg(ink)).when(review && !on, |d| d.border_dashed()))
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
                                    .child(div().min_w_0().text_size(px(13.)).font_weight(FontWeight::MEDIUM).truncate().child(name))
                                    .child(div().font_family(MONO).text_size(px(12.)).flex_none().child(lang::size(item.size))),
                            )
                            .child(div().text_size(px(11.)).text_color(t.soft).truncate().child(item.why.map_or("", |w| w.text()))),
                    )
                    .into_any_element()
            })
            .collect();
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
            .pb(px(32.))
            .flex()
            .flex_col()
            .child(
                drag_window(div().id("panel-head").flex().justify_between().items_baseline())
                    .child(div().text_size(px(15.)).font_weight(FontWeight::SEMIBOLD).child("débarras"))
                    .child(div().font_family(MONO).text_size(px(11.)).text_color(t.soft).child(tr("disk plan", "plan du disque"))),
            )
            .child(div().mt(px(44.)).child(caps(tr("Used", "Occupé"), &t)))
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
                    .mt(px(18.))
                    .flex()
                    .flex_wrap()
                    .gap(px(18.))
                    .child(legend(Pattern::Hatch, tr("without loss", "sans perte")))
                    .child(legend(Pattern::Dots, tr("to review", "à examiner")))
                    .child(legend(Pattern::None, tr("to keep", "à garder"))),
            )
            .child(div().mt(px(32.)).mb(px(8.)).child(caps(&format!("{} · {}", tr("Suggestions", "Suggestions"), lang::number(count as u64)), &t)))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .flex_1()
                    .min_h_0()
                    .mb(px(12.))
                    .overflow_hidden()
                    .when(count == 0, |d| d.child(div().text_size(px(12.)).text_color(t.soft).child(tr("Nothing the rules know how to free here.", "Rien ici que les règles sachent libérer."))))
                    .children(rows),
            )
            .when_some(self.status.clone(), |d, s| d.child(div().debug_selector(|| "status".into()).mb(px(10.)).text_size(px(11.5)).text_color(t.soft).child(s)))
            .child(
                button("free", format!("{} · {}", tr("Move to the trash", "Mettre à la corbeille"), lang::size(picked)), true, &t)
                    .on_click(cx.listener(|this, _, _, cx| this.free_ticked(cx))),
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
