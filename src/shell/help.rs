//! The help panel: every shortcut and gesture, from `keymap::help_sections`, and the version.

use gpui::{FontWeight, div, prelude::*, px};

use super::{keymap::help_sections, *};
use crate::ui::{assets::MONO, widgets::caps};

impl Shell {
    pub(super) fn render_help(&self, (w, h): (f32, f32), cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme;
        let sections = help_sections().into_iter().map(|(title, rows)| {
            div().flex().flex_col().gap(px(6.)).child(div().mb(px(4.)).child(caps(title, &t))).children(rows.into_iter().map(|row| {
                div()
                    .flex()
                    .gap(px(16.))
                    .child(div().w(px(150.)).flex_none().font_family(MONO).text_size(px(12.)).child(row.keys))
                    .child(div().flex_1().min_w_0().text_size(px(12.5)).text_color(t.soft).child(row.effect))
            }))
        });
        // A veil over the window; a click on it closes the help.
        div()
            .id("help-veil")
            .absolute()
            .size_full()
            .bg(t.paper.opacity(0.85))
            .flex()
            .items_center()
            .justify_center()
            .on_click(cx.listener(|this, _, _, cx| {
                this.help = false;
                cx.notify();
            }))
            .child(
                div()
                    .id("help")
                    .debug_selector(|| "help".into())
                    .w(px((w - 64.).min(640.)))
                    .max_h(px(h - 64.))
                    .overflow_y_scroll()
                    .bg(t.panel)
                    .border_1()
                    .border_color(t.ink.opacity(0.85))
                    .p(px(28.))
                    .flex()
                    .flex_col()
                    .gap(px(22.))
                    .on_click(|_, _, cx| cx.stop_propagation())
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_baseline()
                            .child(div().text_size(px(15.)).font_weight(FontWeight::SEMIBOLD).child(tr("Help", "Aide")))
                            .child(div().font_family(MONO).text_size(px(11.)).text_color(t.soft).child(format!("débarras {}", env!("CARGO_PKG_VERSION")))),
                    )
                    .children(sections),
            )
    }
}
