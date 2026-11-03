//! The screens before a plan: choosing what to scan, the scan in progress, a folder that could not be read.

use gpui::{Div, FontWeight, div, prelude::*, px};

use super::{keymap::keys_of, panel::format_took, *};
use crate::ui::{
    assets::{DISPLAY, MONO},
    widgets::{button, caps, drag_window},
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
            _ => column
                .child(div().text_size(px(15.)).font_weight(FontWeight::SEMIBOLD).child("débarras"))
                .child(
                    div()
                        .mt(px(18.))
                        .font_family(DISPLAY)
                        .font_weight(FontWeight::LIGHT)
                        .text_size(px(36.))
                        .line_height(px(42.))
                        .child(tr("See what fills your disk, and free what can go.", "Voyez ce qui remplit votre disque, et libérez ce qui peut partir.")),
                )
                .child(
                    div()
                        .mt(px(14.))
                        .text_size(px(13.))
                        .text_color(t.soft)
                        .child(tr("Nothing leaves your computer, and nothing is deleted without your say.", "Rien ne quitte votre ordinateur, et rien n'est supprimé sans votre accord.")),
                )
                .child(self.choices(cx)),
        };
        div().absolute().size_full().child(drag_window(div().id("start").size_full().flex().items_center().justify_center().child(body)))
    }

    /// What can be scanned: the home folder, a chosen folder, the demo.
    fn choices(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let home = platform::home_dir().map(|h| h.display().to_string()).unwrap_or_else(|| "~".into());
        div()
            .mt(px(32.))
            .flex()
            .flex_wrap()
            .gap(px(12.))
            .child(button("scan-home", format!("{} {home}", tr("Scan", "Analyser")), true, &t).on_click(cx.listener(|this, _, _, cx| this.scan_home(cx))))
            .child(
                button("choose", format!("{} · {}", tr("Choose a folder…", "Choisir un dossier…"), keys_of(&OpenFolder)), false, &t)
                    .on_click(cx.listener(|this, _, window, cx| this.choose_folder(window, cx))),
            )
            .child(button("demo", tr("See the demo", "Voir la démo"), false, &t).on_click(cx.listener(|this, _, _, cx| this.show_demo(cx))))
    }
}
