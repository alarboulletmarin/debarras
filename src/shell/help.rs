//! What is drawn to explain: the help panel (every shortcut and gesture, from
//! `keymap::help_sections`, and the version) and the rules behind the suggestions.

use gpui::{AnyElement, FontWeight, div, prelude::*, px};

use super::{keymap::help_sections, *};
use crate::{
    logic::rules::LISTED,
    ui::{
        assets::MONO,
        widgets::{caps, verdict_tag},
    },
};

impl Shell {
    pub(super) fn render_help(&self, room: (f32, f32), cx: &mut Context<Self>) -> impl IntoElement {
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
        self.sheet("help", tr("Help", "Aide"), format!("débarras {}", env!("CARGO_PKG_VERSION")), sections.map(IntoElement::into_any_element).collect(), room, cx)
    }

    /// How the suggestions are chosen: by name only, each rule with its reason.
    pub(super) fn render_rules(&self, room: (f32, f32), cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme;
        let text = |s: &'static str| div().text_size(px(12.5)).text_color(t.soft).child(s);
        let section = |reclaim: Reclaim, title: &'static str, says: &'static str| {
            div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(div().mb(px(2.)).flex().items_center().gap(px(8.)).child(verdict_tag(reclaim, "", &t)).child(caps(title, &t)))
                .child(div().mb(px(4.)).child(text(says)))
                .children(LISTED.iter().filter(|r| r.0 == reclaim).map(|&(_, names, why)| {
                    div()
                        .flex()
                        .gap(px(16.))
                        .child(div().w(px(190.)).flex_none().font_family(MONO).text_size(px(12.)).child(names.join(", ")))
                        .child(div().flex_1().min_w_0().text_size(px(12.5)).text_color(t.soft).child(why.text()))
                }))
                .into_any_element()
        };
        let body = vec![
            text(tr(
                "Débarras marks a folder or a file only when its name says, without doubt, what it is: it never reads inside files, and never guesses from a date. Each suggestion is the highest marked item: a cache inside a cache counts once. Everything else is left plain, and nothing leaves the disk before you confirm it, in the summary that Free opens.",
                "Débarras ne marque un dossier ou un fichier que lorsque son nom dit, sans doute possible, ce qu'il est : il ne lit jamais le contenu des fichiers et ne devine rien d'après une date. Chaque suggestion est l'élément marqué le plus haut : un cache dans un cache ne compte qu'une fois. Tout le reste est laissé uni, et rien ne quitte le disque avant votre confirmation, dans le récapitulatif qu'ouvre Libérer.",
            ))
            .into_any_element(),
            section(Reclaim::Safe, tr("Hatched: safe to remove", "Hachures : sans risque"), tr("Rebuilt or downloaded again on its own. Selected from the start.", "Se reconstruit ou se retélécharge tout seul. Coché d'office dès l'analyse.")),
            section(Reclaim::Review, tr("Dotted: check first", "Pointillés : à vérifier"), tr("Can go, but it is yours to decide: never selected for you.", "Peut partir, mais c'est à vous de décider : jamais coché d'office.")),
        ];
        self.sheet("rules", tr("How the suggestions are chosen", "Comment les suggestions sont choisies"), String::new(), body, room, cx)
    }

    /// A sheet over a veil; a click on the veil closes it, nothing under it hears the pointer.
    fn sheet(&self, id: &'static str, title: &'static str, aside: String, body: Vec<AnyElement>, (w, h): (f32, f32), cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme;
        div()
            .id((id, 0usize))
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
                    .id(id)
                    .debug_selector(move || id.into())
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
                            .gap(px(16.))
                            .child(div().text_size(px(15.)).font_weight(FontWeight::SEMIBOLD).child(title))
                            .child(div().font_family(MONO).text_size(px(11.)).text_color(t.soft).child(aside)),
                    )
                    .children(body),
            )
    }
}
