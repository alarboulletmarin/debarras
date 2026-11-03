//! The plan's side of the window: the path above it, the dimension line, the plots, the title
//! block below, and the pointer over them.

use gpui::{AnyElement, Div, FontWeight, MouseButton, MouseDownEvent, MouseMoveEvent, div, prelude::*, px};

use super::{keymap::keys_of, panel::PANEL_W, *};
use crate::{
    logic::layout::{self, Opts, Rect},
    ui::{assets::MONO, widgets::drag_window},
    views::plan::{self as draw, hover_marker, items},
};

/// The plan's top edge, under the path and the dimension line.
const TOP: f32 = 104.;
/// The room left under the plan for the title block.
const BOTTOM: f32 = 56.;

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
        let hint = format!(
            "{} : {}   ·   {} : {}   ·   {} : {}",
            tr("click", "clic"),
            tr("open", "ouvrir"),
            keys_of(&Up).split(" / ").next().unwrap_or_default(),
            tr("back", "remonter"),
            keys_of(&ToggleHelp),
            tr("help", "aide"),
        );
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
                .justify_between()
                .gap(px(24.))
                .child(row)
                .child(div().flex_none().font_family(MONO).text_size(px(11.)).text_color(t.soft).child(hint)),
        )
        .into_any_element()
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

    /// The title block under the plan: what is under the pointer.
    fn render_title_block(&self, map: Rect, height: f32) -> impl IntoElement {
        let t = self.theme;
        let text: String = match (&self.hovered, &self.disk, self.shown_node()) {
            (Some(tile), Some(disk), Some(shown)) => {
                let mut parts = vec![String::new(), lang::size(tile.size), format!("{} {}", lang::share(tile.size, shown.size), tr("of", "de")) + " " + &shown.name];
                match &tile.path {
                    Some(rel) => {
                        let path = self.rooted(rel);
                        parts[0] = match tile.kind {
                            crate::logic::tree::Kind::Rest(_) => draw::label(tile),
                            _ => disk.root.short_name(&path),
                        };
                        parts.push(tile.category.label().to_string());
                        if let Some(why) = disk.root.at(&path).and_then(|n| n.why) {
                            parts.push(why.text().to_string());
                        }
                    }
                    None => {
                        parts[0] = draw::label(tile);
                        parts.push(tr("Small items grouped together.", "Petits éléments regroupés.").to_string());
                    }
                }
                parts.join("   ·   ")
            }
            _ => tr("Hover a plot. Hatching: can go without loss. Dots: to review.", "Survolez une parcelle. Hachures : libérable sans perte. Pointillés : à examiner.").to_string(),
        };
        div()
            .absolute()
            .debug_selector(|| "title-block".into())
            .left(px(map.x))
            .top(px(height - 36.))
            .w(px(map.w))
            .font_family(MONO)
            .text_size(px(11.5))
            .text_color(if self.hovered.is_some() { t.ink } else { t.soft })
            .truncate()
            .child(text)
    }
}
