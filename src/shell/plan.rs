//! The plan's side of the window: the path and the legend above it, the dimension line, the plots,
//! the info bar below, and the pointer over them.

use gpui::{AnyElement, Div, FontWeight, MouseButton, MouseDownEvent, MouseMoveEvent, div, prelude::*, px};

use super::{keymap::keys_of, panel::PANEL_W, *};
use crate::{
    logic::layout::{self, Opts, Rect},
    ui::{
        assets::MONO,
        widgets::{drag_window, one_line, verdict_tag},
    },
    views::plan::{self as draw, hover_marker, items},
};

/// The plan's top edge, under the path and the dimension line.
const TOP: f32 = 104.;
/// The room left under the plan for the title block.
const BOTTOM: f32 = 64.;

impl Shell {
    /// Where the plan is drawn, in the content's coordinates.
    pub(super) fn map_rect((w, h): (f32, f32)) -> Rect {
        Rect::new(PANEL_W + 32., TOP, (w - PANEL_W - 64.).max(200.), (h - TOP - BOTTOM).max(200.))
    }

    pub(super) fn render_plan(&mut self, room: (f32, f32), cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let map = Self::map_rect(room);
        let key = PlanKey { disk: self.disk_count, shown: self.shown.clone(), map, theme: t.name, ticks: self.tick_count };
        if self.plot_key.as_ref() != Some(&key) {
            // Laid out and drawn again only when what it shows changes: never for the pointer.
            let opts = Opts::default();
            self.tiles = self.shown_node().map(|node| layout::layout(node, map, &opts)).unwrap_or_default();
            let ticked: Vec<bool> = self.tiles.iter().map(|tile| tile.path.as_ref().is_some_and(|rel| self.ticked.contains(&self.rooted(rel)))).collect();
            let tiles = self.tiles.clone();
            self.plot.update(cx, |plot, cx| plot.set(tiles, ticked, map, t, opts, cx));
            self.plot_key = Some(key);
        }
        let plots = gpui::AnyView::from(self.plot.clone()).cached(gpui::StyleRefinement::default().size_full());
        let marker = self.hovered.as_ref().map(|tile| hover_marker(tile.rect, map, &t));
        let empty = self.tiles.is_empty();

        div()
            .absolute()
            .size_full()
            .child(self.render_panel(room.1, cx))
            .child(self.render_path(map, cx))
            .child(self.render_dimension(map))
            .child(
                div()
                    .id("map")
                    .debug_selector(|| "map".into())
                    .absolute()
                    .left(px(map.x))
                    .top(px(map.y))
                    .w(px(map.w))
                    .h(px(map.h))
                    .cursor_pointer()
                    .on_mouse_move(cx.listener(|this, e: &MouseMoveEvent, window, cx| {
                        let inset = Self::inset(window);
                        let now = layout::hit(&this.tiles, f32::from(e.position.x) - inset, f32::from(e.position.y) - inset).cloned();
                        if now != this.hovered {
                            this.hovered = now;
                            cx.notify();
                        }
                    }))
                    .on_hover(cx.listener(|this, inside: &bool, _, cx| {
                        if !*inside && this.hovered.take().is_some() {
                            cx.notify();
                        }
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, e: &MouseDownEvent, window, cx| {
                            let inset = Self::inset(window);
                            if let Some(tile) = layout::hit(&this.tiles, f32::from(e.position.x) - inset, f32::from(e.position.y) - inset).cloned() {
                                this.press(tile, cx);
                            }
                        }),
                    )
                    .on_mouse_down(MouseButton::Right, cx.listener(|this, _: &MouseDownEvent, _, cx| this.up(cx)))
                    .when(empty, |d| {
                        d.flex().items_center().justify_center().border_1().border_dashed().border_color(t.rule).child(
                            div().text_size(px(13.)).text_color(t.soft).child(tr("This folder is empty.", "Ce dossier est vide.")),
                        )
                    })
                    .child(plots)
                    .children(marker),
            )
            .child(self.render_title_block(map, room.1))
    }

    /// The path of the shown folder: each name goes back to its folder. The row also moves the window.
    fn render_path(&self, map: Rect, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme;
        let Some(disk) = &self.disk else { return div().into_any_element() };
        let names: Vec<String> = disk.root.names(&self.shown).into_iter().map(str::to_string).collect();
        let last = names.len() - 1;
        let mut row = div().flex().items_center().gap(px(10.)).min_w_0().overflow_hidden().font_family(MONO).text_size(px(13.));
        for (i, name) in names.into_iter().enumerate() {
            if i > 0 {
                row = row.child(div().flex_none().text_color(t.soft).child("/"));
            }
            let accent = t.accent;
            row = row.child(
                div()
                    .id(("crumb", i))
                    .debug_selector(move || format!("crumb-{i}"))
                    .cursor_pointer()
                    .when(i == last, |d| d.min_w_0().truncate().font_weight(FontWeight::MEDIUM))
                    .when(i != last, |d| d.flex_shrink().min_w(px(24.)).truncate().text_color(t.soft).hover(move |s| s.text_color(accent)))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.shown.truncate(i);
                        this.open(this.shown.clone(), cx);
                    }))
                    .child(name),
            );
        }
        let accent = t.accent;
        let back = (!self.shown.is_empty()).then(|| {
            div()
                .id("up-button")
                .debug_selector(|| "up-button".into())
                .flex_none()
                .h(px(24.))
                .px(px(8.))
                .flex()
                .items_center()
                .border_1()
                .border_color(t.ink.opacity(0.75))
                .font_family(MONO)
                .text_size(px(11.))
                .cursor_pointer()
                .hover(move |s| s.border_color(accent).text_color(accent))
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_click(cx.listener(|this, _, _, cx| this.up(cx)))
                .child(format!("← {}", tr("Up", "Remonter")))
        });
        drag_window(
            div()
                .id("path-row")
                .absolute()
                .left(px(map.x))
                .top(px(30.))
                .w(px(map.w))
                .h(px(28.))
                .flex()
                .items_center()
                .gap(px(14.))
                .children(back)
                .child(row)
                .child(div().flex_1())
                .child(self.render_legend(cx)),
        )
        .into_any_element()
    }

    /// The legend of the patterns, above the plan: each verdict's word and what it weighs on the disk.
    /// A click explains how the suggestions are chosen.
    fn render_legend(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme;
        let (mut safe, mut review) = (0, 0);
        for s in self.disk.iter().flat_map(|d| &d.suggestions) {
            if s.reclaim == Reclaim::Review { review += s.size } else { safe += s.size }
        }
        let entry = |reclaim: Reclaim, size: Option<u64>| {
            div().flex().items_center().gap(px(6.)).child(verdict_tag(reclaim, "", &t)).children(size.map(|s| div().font_family(MONO).text_size(px(10.5)).text_color(t.soft).child(lang::size(s))))
        };
        div()
            .id("legend")
            .debug_selector(|| "legend".into())
            .flex_none()
            .flex()
            .items_center()
            .gap(px(14.))
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(|this, _, _, cx| this.toggle_overlay(Overlay::Rules, cx)))
            .child(entry(Reclaim::Safe, Some(safe)))
            .child(entry(Reclaim::Review, Some(review)))
            .child(entry(Reclaim::No, None))
    }

    /// The architect's dimension line above the plan: the shown folder's size and how many items it holds.
    fn render_dimension(&self, map: Rect) -> impl IntoElement {
        let t = self.theme;
        let (size, count) = self.shown_node().map_or((0, 0), |n| (n.size, n.children.len() as u64));
        let tick = || div().w(px(1.)).h(px(12.)).bg(t.ink.opacity(0.55));
        let rule = || div().flex_1().h(px(1.)).bg(t.ink.opacity(0.55));
        div()
            .absolute()
            .left(px(map.x))
            .top(px(74.))
            .w(px(map.w))
            .h(px(12.))
            .flex()
            .items_center()
            .child(tick())
            .child(rule())
            .child(div().px(px(12.)).font_family(MONO).text_size(px(11.5)).child(format!("{}  ·  {}", lang::size(size), items(count))))
            .child(rule())
            .child(tick())
    }

    /// The info bar under the plan: what the hovered block is, its verdict in words, what a click does.
    fn render_title_block(&self, map: Rect, height: f32) -> impl IntoElement {
        let t = self.theme;
        let bar = div().absolute().debug_selector(|| "title-block".into()).left(px(map.x)).top(px(height - BOTTOM + 12.)).w(px(map.w)).flex().flex_col().gap(px(4.));
        let (Some(tile), Some(disk), Some(shown)) = (&self.hovered, &self.disk, self.shown_node()) else {
            return bar.child(div().font_family(MONO).text_size(px(11.)).text_color(t.soft).child(tr("Hover a block to know what it is.", "Survolez un bloc pour savoir ce que c'est.")));
        };
        let mut facts = vec![lang::size(tile.size), format!("{} {} {}", lang::share(tile.size, shown.size), tr("of", "de"), shown.name)];
        let (name, why, tag, click) = match &tile.path {
            Some(rel) => {
                let path = self.rooted(rel);
                facts.push(tile.category.label().to_string());
                let name = match tile.kind {
                    crate::logic::tree::Kind::Rest(_) => draw::label(tile),
                    _ => disk.root.short_name(&path),
                };
                let why = disk.root.at(&path).and_then(|n| n.why).map_or("", |w| w.text());
                let click = match tile.reclaim {
                    Reclaim::No if tile.is_dir() => Some(tr("open", "ouvrir")),
                    Reclaim::No => None,
                    _ if self.ticked.contains(&path) => Some(tr("unselect", "désélectionner")),
                    _ => Some(tr("select", "sélectionner")),
                };
                (name, why, Some(tile.reclaim), click)
            }
            None => (draw::label(tile), tr("Small items grouped together.", "Petits éléments regroupés."), None, None),
        };
        let mut hint: Vec<String> = click.map(|c| format!("{} : {c}", tr("Click", "Clic"))).into_iter().collect();
        if tile.path.is_some() {
            hint.push(format!("{} : {}", keys_of(&Reveal), tr("show in the folder", "montrer dans le dossier")));
        }
        bar.child(
            div()
                .flex()
                .items_center()
                .gap(px(10.))
                .child(one_line(div().min_w_0().text_size(px(12.5)).font_weight(FontWeight::MEDIUM)).child(name))
                .child(div().flex_none().font_family(MONO).text_size(px(10.5)).text_color(t.soft).child(facts.join(" · ")))
                .children(tag.map(|r| verdict_tag(r, "", &t))),
        )
        .child(
            div()
                .flex()
                .justify_between()
                .gap(px(16.))
                .child(one_line(div().flex_1().min_w_0().text_size(px(11.5)).text_color(t.soft)).child(why))
                .child(div().flex_none().font_family(MONO).text_size(px(11.)).text_color(t.soft).child(hint.join(" · "))),
        )
    }
}
