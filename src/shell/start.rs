//! The screens before a plan: choosing what to scan, the scan in progress, a folder that could not be read.

use gpui::{Div, FontWeight, div, prelude::*, px};

use super::{keymap::keys_of, panel::format_took, *};
use crate::ui::{
    assets::{DISPLAY, MONO},
    widgets::{button, caps, drag_window, verdict_tag},
};

impl Shell {
    pub(super) fn render_start(&self, (w, _): (f32, f32), cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let column = div().w(px((w - 96.).clamp(280., 560.))).flex().flex_col();
        let body = match &self.stage {
            Stage::Scanning { path, progress, started } => column
                .child(caps(tr("Scanning", "Analyse en cours"), &t))
                .child(div().mt(px(10.)).font_family(MONO).text_size(px(13.)).truncate().child(path.display().to_string()))
                .child(div().mt(px(28.)).font_family(DISPLAY).font_weight(FontWeight::LIGHT).text_size(px(48.)).child(lang::size(progress.bytes.load(Relaxed))))
                .child(
                    div()
                        .mt(px(6.))
                        .debug_selector(|| "scan-count".into())
                        .font_family(MONO)
                        .text_size(px(12.))
                        .text_color(t.soft)
                        .child(format!("{} · {}", lang::count(progress.files.load(Relaxed), ("file", "files", "fichier", "fichiers")), format_took(started.elapsed()))),
                )
                .child(div().mt(px(28.)).flex().child(button("stop", format!("{} · {}", tr("Stop", "Arrêter"), keys_of(&Up).split(" / ").next().unwrap_or_default()), false, &t).on_click(cx.listener(|this, _, _, cx| this.up(cx))))),
            Stage::Failed { path, reason } => column
                .child(caps(tr("Could not scan", "Analyse impossible"), &t))
                .child(div().mt(px(10.)).font_family(MONO).text_size(px(13.)).truncate().child(path.display().to_string()))
                .child(div().mt(px(14.)).debug_selector(|| "failure".into()).text_size(px(15.)).child(reason.clone()))
                .child(self.choices(cx)),
            _ => {
                let step = |n: &'static str, name: &'static str, rest: &'static str| {
                    div().flex().gap(px(10.)).child(div().w(px(12.)).flex_none().font_family(MONO).text_color(t.soft).child(n)).child(
                        div().flex_1().min_w_0().child(format!("{name} {rest}")),
                    )
                };
                let meaning = |reclaim: Reclaim, says: &'static str| div().flex().items_center().gap(px(10.)).child(verdict_tag(reclaim, "", &t)).child(div().text_size(px(11.5)).text_color(t.soft).child(says));
                column
                    .child(div().text_size(px(15.)).font_weight(FontWeight::SEMIBOLD).child("débarras"))
                    .child(
                        div()
                            .mt(px(18.))
                            .font_family(DISPLAY)
                            .font_weight(FontWeight::LIGHT)
                            .text_size(px(36.))
                            .line_height(px(40.))
                            .child(tr("See what fills your disk.", "Voyez ce qui remplit votre disque.")),
                    )
                    .child(
                        div()
                            .debug_selector(|| "first-steps".into())
                            .mt(px(18.))
                            .flex()
                            .flex_col()
                            .gap(px(6.))
                            .text_size(px(13.))
                            .line_height(px(18.))
                            .child(step("1", tr("Scan", "Analyser"), tr("a folder. Only sizes are read, never what files hold.", "un dossier. Seules les tailles sont lues, jamais le contenu.")))
                            .child(step("2", tr("Choose", "Choisir"), tr("what goes. The safest is already selected.", "ce qui part. Le plus sûr est déjà coché.")))
                            .child(step("3", tr("Free", "Libérer"), tr("it after a summary, to the trash.", "après un résumé, vers la corbeille."))),
                    )
                    .child(
                        div()
                            .mt(px(16.))
                            .flex()
                            .flex_col()
                            .gap(px(6.))
                            .child(meaning(Reclaim::Safe, tr("rebuilt on its own", "se reconstruit tout seul")))
                            .child(meaning(Reclaim::Review, tr("large, yours to judge", "volumineux, à vous de juger"))),
                    )
                    .child(self.choices(cx))
                    .child(
                        div()
                            .mt(px(14.))
                            .text_size(px(12.))
                            .text_color(t.soft)
                            .child(tr("Nothing leaves your computer, and nothing is deleted without your say.", "Rien ne quitte votre ordinateur, et rien n'est supprimé sans votre accord.")),
                    )
            }
        };
        div().absolute().size_full().child(drag_window(div().id("start").size_full().flex().items_center().justify_center().child(body)))
    }

    /// What can be scanned: the home folder, a chosen folder, the demo.
    fn choices(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        div()
            .mt(px(32.))
            .flex()
            .flex_wrap()
            .gap(px(12.))
            .child(button("scan-home", tr("Scan my home folder", "Analyser mon dossier personnel"), true, &t).on_click(cx.listener(|this, _, _, cx| this.scan_home(cx))))
            .child(
                button("choose", format!("{} · {}", tr("Choose a folder…", "Choisir un dossier…"), keys_of(&OpenFolder)), false, &t)
                    .on_click(cx.listener(|this, _, window, cx| this.choose_folder(window, cx))),
            )
            .child(button("demo", tr("See the demo", "Voir la démo"), false, &t).on_click(cx.listener(|this, _, _, cx| this.show_demo(cx))))
    }
}
