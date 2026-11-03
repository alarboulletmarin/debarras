//! Freeing what is selected: the summary to confirm, the deletion in the background, then the plan
//! without what is gone. Nothing leaves the disk before the user picks the trash or deletion.

use std::path::PathBuf;

use gpui::{AnyElement, FontWeight, div, prelude::*, px};

use super::*;
use crate::{
    logic::rules::Why,
    platform::remove,
    ui::{
        assets::MONO,
        widgets::{button, danger_button, one_line, verdict_color, verdict_tag},
    },
};

/// How the ticked items leave.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Way {
    /// To the system's trash: the room comes back once it is emptied.
    Trash,
    /// Deleted for good.
    Delete,
}

impl Shell {
    /// The free button: the summary to confirm, or why there is nothing to free.
    pub(super) fn ask_free(&mut self, cx: &mut Context<Self>) {
        if self.freeing {
            return;
        }
        if self.to_free().is_empty() {
            self.status = Some(tr("Nothing is selected: click a hatched or dotted block first.", "Rien n'est sélectionné : cliquez d'abord un bloc hachuré ou pointillé.").into());
        } else {
            self.overlay = Some(Overlay::Confirm);
        }
        cx.notify();
    }

    /// Frees what is ticked, the confirmed way. The demo only pretends: nothing on disk is touched.
    pub(super) fn free(&mut self, way: Way, cx: &mut Context<Self>) {
        let Some(disk) = &self.disk else { return };
        let items: Vec<(Vec<usize>, Option<PathBuf>, u64, bool)> =
            self.to_free().into_iter().map(|(p, n)| (p.clone(), disk.root.disk_path(&p), n.size, n.why == Some(Why::Trash))).collect();
        self.overlay = None;
        if items.is_empty() || self.freeing {
            return cx.notify();
        }
        if disk.source == Source::Demo {
            let done = items.into_iter().map(|(p, _, size, _)| (p, size, Ok(()))).collect();
            return self.freed(self.disk_count, way, done, cx);
        }
        self.freeing = true;
        self.status = Some(tr("Freeing…", "Libération en cours…").into());
        cx.notify();
        let work = cx.background_spawn(async move {
            items
                .into_iter()
                .map(|(path, at, size, is_trash)| {
                    let done = match &at {
                        // The trash itself only goes by being emptied.
                        Some(_) if way == Way::Trash && is_trash => {
                            Err(std::io::Error::other(tr("it is the trash: delete it for good to empty it", "c'est la corbeille : supprimez-la définitivement pour la vider")))
                        }
                        Some(at) if way == Way::Trash => remove::trash(at),
                        Some(at) => remove::delete(at),
                        None => Err(std::io::ErrorKind::NotFound.into()),
                    };
                    (path, size, done)
                })
                .collect::<Vec<_>>()
        });
        let disk_count = self.disk_count;
        cx.spawn(async move |this, cx| {
            let done = work.await;
            this.update(cx, |this, cx| this.freed(disk_count, way, done, cx)).ok();
        })
        .detach();
    }

    /// What was freed leaves the plan, at once and without a new scan; what could not be stays ticked.
    fn freed(&mut self, disk_count: u64, way: Way, done: Vec<(Vec<usize>, u64, std::io::Result<()>)>, cx: &mut Context<Self>) {
        self.freeing = false;
        let (gone, failed): (Vec<_>, Vec<_>) = done.into_iter().partition(|(_, _, r)| r.is_ok());
        let bytes: u64 = gone.iter().map(|(_, size, _)| size).sum();
        let mut said = match way {
            Way::Trash => format!("{} {} · {}", tr("Moved to the trash:", "Mis à la corbeille :"), lang::count(gone.len() as u64, ("item", "items", "élément", "éléments")), lang::size(bytes)),
            Way::Delete => format!("{} {} · {}", tr("Deleted:", "Supprimé :"), lang::count(gone.len() as u64, ("item", "items", "élément", "éléments")), lang::size(bytes)),
        };
        // Another scan came meanwhile: its plan is already what the disk holds.
        let Some(mut disk) = self.disk.take().filter(|_| self.disk_count == disk_count) else {
            self.status = Some(said.into());
            return cx.notify();
        };
        let names: Vec<String> = disk.root.names(&self.shown).iter().skip(1).map(|n| n.to_string()).collect();
        let still: Vec<Vec<String>> = failed.iter().map(|(p, ..)| disk.root.names(p).iter().skip(1).map(|n| n.to_string()).collect()).collect();
        if let Some((path, _, Err(e))) = failed.first() {
            said += &format!(
                ". {} {} — {}",
                lang::count(failed.len() as u64, ("item could not be:", "items could not be:", "élément n'a pas pu l'être :", "éléments n'ont pas pu l'être :")),
                disk.root.short_name(path),
                lang::io_reason(e)
            );
        }
        disk.root.remove(&gone.into_iter().map(|(p, ..)| p).collect::<Vec<_>>());
        let disk = Disk::new(disk.root, disk.source, disk.files, disk.errors, disk.took);
        self.shown = disk.root.find(&names);
        // A failure is found again by its names: the indices moved.
        self.auto_ticked = None;
        self.ticked = still.iter().map(|names| disk.root.find_item(names)).filter(|p| !p.is_empty()).collect();
        self.disk = Some(disk);
        self.disk_count += 1;
        self.tick_count += 1;
        self.hovered = None;
        self.list = List::Ticked;
        self.unfolded.clear();
        self.status = Some(said.into());
        cx.notify();
    }

    /// The summary over the window: what will leave, grouped by verdict with its subtotal, then the
    /// trash first and deleting for good aside.
    pub(super) fn render_confirm(&self, (w, h): (f32, f32), cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme;
        let items = self.to_free();
        let total: u64 = items.iter().map(|(_, n)| n.size).sum();
        let demo = self.disk.as_ref().is_some_and(|d| d.source == Source::Demo);
        let Some(disk) = &self.disk else { return div().into_any_element() };
        let row = |path: &Vec<usize>, size: u64| {
            let at = disk.root.disk_path(path).map(|p| p.display().to_string()).unwrap_or_default();
            div()
                .flex()
                .items_center()
                .gap(px(10.))
                .py(px(4.))
                .pl(px(4.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(one_line(div().text_size(px(12.5)).font_weight(FontWeight::MEDIUM)).child(disk.root.short_name(path)))
                        .child(one_line(div().font_family(MONO).text_size(px(10.5)).text_color(t.soft)).child(at)),
                )
                .child(div().flex_none().font_family(MONO).text_size(px(10.5)).text_color(t.soft).child(lang::size(size)))
                .into_any_element()
        };
        let groups: Vec<AnyElement> = [Reclaim::Safe, Reclaim::Review]
            .into_iter()
            .filter_map(|reclaim| {
                let mine: Vec<_> = items.iter().filter(|(_, n)| (n.reclaim == Reclaim::Review) == (reclaim == Reclaim::Review)).collect();
                if mine.is_empty() {
                    return None;
                }
                let subtotal: u64 = mine.iter().map(|(_, n)| n.size).sum();
                let warn = (reclaim == Reclaim::Review).then(|| {
                    format!(
                        "{} {}",
                        lang::count(mine.len() as u64, ("item to check first:", "items to check first:", "élément à vérifier :", "éléments à vérifier :")),
                        tr("make sure you no longer need it.", "assurez-vous de ne plus en avoir besoin.")
                    )
                });
                Some(
                    div()
                        .debug_selector(move || format!("confirm-group-{}", reclaim as usize))
                        .border_t_1()
                        .border_color(t.rule)
                        .pb(px(8.))
                        .child(
                            div()
                                .h(px(36.))
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(verdict_tag(reclaim, &lang::number(mine.len() as u64), &t))
                                .child(div().font_family(MONO).text_size(px(10.5)).child(lang::size(subtotal))),
                        )
                        .children(warn.map(|w| div().pl(px(4.)).pb(px(4.)).text_size(px(11.5)).text_color(verdict_color(reclaim, &t)).child(w)))
                        .children(mine.iter().map(|(path, n)| row(path, n.size)))
                        .into_any_element(),
                )
            })
            .collect();
        let note = if demo {
            tr("The demo is made up: nothing on your disk is touched.", "La démo est inventée : rien n'est touché sur votre disque.")
        } else {
            tr(
                "The trash keeps everything until you empty it: the room comes back then. Deleting for good frees it now, and cannot be undone.",
                "La corbeille garde tout jusqu'à ce que vous la vidiez : la place ne revient qu'alors. Supprimer définitivement la libère tout de suite, sans retour possible.",
            )
        };
        let cancel = cx.listener(|this: &mut Self, _: &gpui::ClickEvent, _: &mut Window, cx: &mut Context<Self>| {
            this.overlay = None;
            cx.notify();
        });
        div()
            .id("confirm-veil")
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
                    .id("confirm")
                    .debug_selector(|| "confirm".into())
                    .w(px((w - 64.).min(560.)))
                    .max_h(px(h - 64.))
                    .bg(t.panel)
                    .border_1()
                    .border_color(t.ink.opacity(0.45))
                    .p(px(24.))
                    .flex()
                    .flex_col()
                    .on_click(|_, _, cx| cx.stop_propagation())
                    .child(div().text_size(px(15.)).line_height(px(20.)).font_weight(FontWeight::SEMIBOLD).child(format!(
                        "{} {} · {}",
                        tr("Free", "Libérer"),
                        lang::size(total),
                        lang::count(items.len() as u64, ("item", "items", "élément", "éléments"))
                    )))
                    .child(div().id("confirm-list").debug_selector(|| "confirm-list".into()).mt(px(16.)).flex_1().min_h_0().overflow_y_scroll().border_b_1().border_color(t.rule).children(groups))
                    .child(div().mt(px(14.)).text_size(px(11.5)).line_height(px(16.)).text_color(t.soft).child(note))
                    .child(
                        div()
                            .mt(px(20.))
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(button("confirm-trash", tr("Move to the trash", "Mettre à la corbeille"), true, &t).w_full().on_click(cx.listener(|this, _, _, cx| this.free(Way::Trash, cx))))
                            .child(
                                div()
                                    .flex()
                                    .gap(px(8.))
                                    .child(danger_button("confirm-delete", tr("Delete for good", "Supprimer définitivement"), &t).flex_1().on_click(cx.listener(|this, _, _, cx| this.free(Way::Delete, cx))))
                                    .child(button("confirm-cancel", tr("Cancel", "Annuler"), false, &t).on_click(cancel)),
                            ),
                    ),
            )
            .into_any_element()
    }
}
