//! The plan itself: one plot per tile of the layout, the red marker of the hovered plot, and the
//! words that name a plot.

use gpui::{AnyElement, Context, FontWeight, Window, div, prelude::*, px};

use crate::{
    lang,
    logic::{
        layout::{Opts, Rect, Tile},
        tree::Kind,
    },
    ui::{
        assets::{DISPLAY, MONO},
        theme::Theme,
        widgets::{Pattern, checkbox, pattern_layer},
    },
};

/// The name a plot shows: its own, or what a summed plot holds.
pub fn label(t: &Tile) -> String {
    match t.kind {
        _ if t.grouped > 0 => lang::count(t.grouped as u64, ("other", "others", "autre", "autres")),
        Kind::Rest(n) => lang::count(n, ("small file", "small files", "petit fichier", "petits fichiers")),
        _ => t.name.clone(),
    }
}

/// How many items, in words: `0 items`, `1 item`, `1,204 items`.
pub fn items(n: u64) -> String {
    lang::count(n, ("item", "items", "élément", "éléments"))
}

/// One plot, placed relative to the plan's corner `map`. `ticked`: chosen to be freed.
pub fn tile(t: &Tile, map: Rect, theme: &Theme, ticked: bool, opts: &Opts) -> AnyElement {
    let r = t.rect;
    let summed = t.grouped > 0 || matches!(t.kind, Kind::Rest(_));
    let tint = if t.grouped > 0 { theme.paper } else { theme.tint(t.category) };
    let pattern = Pattern::of(t.reclaim);
    // Over a pattern, a name sits on a patch of the panel to stay readable.
    let label_bg = if pattern == Pattern::None { tint.opacity(0.) } else { theme.panel.opacity(0.92) };
    let size = lang::size(t.size);
    let name = label(t);

    let mut d = div()
        .absolute()
        .left(px(r.x - map.x))
        .top(px(r.y - map.y))
        .w(px(r.w))
        .h(px(r.h))
        .bg(tint)
        .border_1()
        .border_color(theme.ink.opacity(if t.depth == 1 { 0.85 } else { 0.45 }))
        .when(summed, |d| d.border_dashed())
        .overflow_hidden()
        .children(pattern_layer(pattern, theme.ink, (r.w, r.h)));

    if t.open {
        // An open folder: its name and size in a band at the top, its children below.
        d = d.child(
            div()
                .absolute()
                .top(px(3.))
                .left(px(3.))
                .right(px(3.))
                .h(px(opts.header - 6.))
                .px(px(5.))
                .bg(label_bg)
                .flex()
                .items_center()
                .justify_between()
                .gap(px(8.))
                .child(
                    div()
                        .min_w_0()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .when(pattern != Pattern::None, |d| d.child(checkbox(ticked, theme)))
                        .child(div().min_w_0().text_size(px(12.)).font_weight(FontWeight::SEMIBOLD).truncate().child(name)),
                )
                .child(div().font_family(MONO).text_size(px(11.)).text_color(theme.soft).flex_none().child(size)),
        );
    } else if r.w > 56. && r.h > 36. {
        let big = r.w > 150. && r.h > 96.;
        d = d.child(
            div()
                .absolute()
                .top(px(4.))
                .left(px(4.))
                .max_w(px(r.w - 8.))
                .px(px(4.))
                .py(px(1.))
                .bg(label_bg)
                .flex()
                .flex_col()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .when(pattern != Pattern::None, |d| d.child(checkbox(ticked, theme)))
                        .child(div().min_w_0().text_size(px(if t.depth == 1 { 12.5 } else { 11.5 })).font_weight(FontWeight::MEDIUM).truncate().child(name)),
                )
                .when(!big && r.h > 44., |d| d.child(div().font_family(MONO).text_size(px(10.5)).text_color(theme.soft).child(size.clone()))),
        );
        if big {
            let (n, unit) = lang::size_parts(t.size);
            let height = (r.h * 0.16).clamp(16., 30.);
            d = d.child(
                div()
                    .absolute()
                    .bottom(px(4.))
                    .left(px(4.))
                    .px(px(4.))
                    .bg(label_bg)
                    .flex()
                    .items_baseline()
                    .gap(px(4.))
                    .child(div().font_family(DISPLAY).font_weight(FontWeight::LIGHT).text_size(px(height)).child(n))
                    .child(div().text_size(px(height * 0.5)).text_color(theme.soft).child(unit)),
            );
        }
    }
    d.into_any_element()
}

/// The plots of the shown folder, as one view: the window draws it cached, so that moving the
/// pointer (the red marker, the title block) never lays out nor paints the plots again. It is
/// notified only when what it shows changes (`set`).
pub struct Plot {
    tiles: Vec<Tile>,
    ticked: Vec<bool>,
    map: Rect,
    theme: Option<Theme>,
    opts: Opts,
    /// How many times the plots were drawn: the tests check that the pointer costs none.
    #[cfg(test)]
    pub renders: usize,
}

impl Plot {
    pub fn new() -> Self {
        Plot {
            tiles: Vec::new(),
            ticked: Vec::new(),
            map: Rect::default(),
            theme: None,
            opts: Opts::default(),
            #[cfg(test)]
            renders: 0,
        }
    }

    /// What to draw: the tiles in the content's coordinates, which of them are ticked, the plan's corner.
    pub fn set(&mut self, tiles: Vec<Tile>, ticked: Vec<bool>, map: Rect, theme: Theme, opts: Opts, cx: &mut Context<Self>) {
        (self.tiles, self.ticked, self.map, self.theme, self.opts) = (tiles, ticked, map, Some(theme), opts);
        cx.notify();
    }
}

impl Render for Plot {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        #[cfg(test)]
        {
            self.renders += 1;
        }
        let plots: Vec<AnyElement> = match &self.theme {
            Some(theme) => self.tiles.iter().zip(&self.ticked).map(|(t, on)| tile(t, self.map, theme, *on, &self.opts)).collect(),
            None => Vec::new(),
        };
        div().size_full().relative().children(plots)
    }
}

/// The hovered plot: a red outline, and two corner marks as a pencil would set them.
pub fn hover_marker(r: Rect, map: Rect, theme: &Theme) -> AnyElement {
    let corner = |x: f32, y: f32| div().absolute().left(px(x)).top(px(y)).size(px(10.)).border_color(theme.accent);
    div()
        .absolute()
        .left(px(r.x - map.x - 1.))
        .top(px(r.y - map.y - 1.))
        .w(px(r.w + 2.))
        .h(px(r.h + 2.))
        .border_2()
        .border_color(theme.accent)
        .child(corner(-7., -7.).border_t_2().border_l_2())
        .child(corner(r.w - 5., r.h - 5.).border_b_2().border_r_2())
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::tree::{Category, Reclaim};

    fn tile(kind: Kind, grouped: usize) -> Tile {
        Tile { path: None, rect: Rect::default(), depth: 1, name: "nom".into(), size: 1, kind, category: Category::Other, reclaim: Reclaim::No, open: false, grouped }
    }

    #[test]
    fn names_summed_plots_with_their_count() {
        assert_eq!(label(&tile(Kind::File, 0)), "nom");
        assert_eq!(label(&tile(Kind::Rest(1), 0)), "1 petit fichier");
        assert_eq!(label(&tile(Kind::Rest(1_000), 0)), "1\u{202f}000 petits fichiers");
        assert_eq!(label(&tile(Kind::Rest(3), 2)), "2 autres");
        assert_eq!(crate::lang::in_english(|| label(&tile(Kind::Rest(2), 0))), "2 small files");
        assert_eq!((items(0), items(1), items(2)), ("0 élément".into(), "1 élément".into(), "2 éléments".into()));
    }
}
