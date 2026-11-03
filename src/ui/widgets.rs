//! Small pieces drawn the same way everywhere: the fill of a plot (hatching, dots), the legend's
//! swatches, the labels in capitals and the large figures.

use gpui::{AnyElement, Bounds, Div, FontWeight, Hsla, MouseButton, Pixels, Stateful, Window, canvas, div, pattern_slash, point, prelude::*, px, svg};

use crate::{
    lang,
    logic::tree::Reclaim,
    ui::{
        assets::{DISPLAY, DOTS, DOTS_SIDE, MONO},
        theme::Theme,
    },
};

/// How a plot is filled, after what can be done with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pattern {
    None,
    /// Hatching at 45°: can go without loss.
    Hatch,
    /// Dots: to review.
    Dots,
}

impl Pattern {
    pub fn of(reclaim: Reclaim) -> Self {
        match reclaim {
            Reclaim::Safe => Pattern::Hatch,
            Reclaim::Review => Pattern::Dots,
            Reclaim::No => Pattern::None,
        }
    }
}

/// The pattern over a whole plot of `w` × `h` px, the same ticked or not: the checkbox says that.
/// Each is a handful of primitives whatever the plot's size: hatching is drawn by the shader, dots are
/// tiles of one picture, rasterized once. A dot per primitive cost milliseconds a frame.
pub fn pattern_layer(pattern: Pattern, ink: Hsla, (w, h): (f32, f32)) -> Option<AnyElement> {
    let layer = || div().absolute().top_0().left_0().size_full().overflow_hidden();
    match pattern {
        Pattern::None => None,
        Pattern::Hatch => Some(layer().bg(pattern_slash(ink.opacity(0.18), 1., 6.)).into_any_element()),
        Pattern::Dots => {
            let dot = ink.opacity(0.42);
            let (cols, rows) = ((w / DOTS_SIDE).ceil() as usize, (h / DOTS_SIDE).ceil() as usize);
            let tiles = (0..rows * cols).map(move |i| {
                let (x, y) = ((i % cols) as f32 * DOTS_SIDE, (i / cols) as f32 * DOTS_SIDE);
                svg().path(DOTS).absolute().left(px(x)).top(px(y)).size(px(DOTS_SIDE)).text_color(dot)
            });
            Some(layer().children(tiles).into_any_element())
        }
    }
}

/// A square of the legend, filled with the pattern it explains.
pub fn swatch(pattern: Pattern, t: &Theme) -> AnyElement {
    let d = div().relative().size(px(11.)).flex_none().border_1().border_color(t.ink.opacity(0.6)).overflow_hidden();
    match pattern {
        Pattern::Hatch => d.bg(pattern_slash(t.ink.opacity(0.5), 1., 3.)).into_any_element(),
        Pattern::Dots => d.flex().items_center().justify_center().child(div().size(px(2.)).bg(t.ink.opacity(0.7))).into_any_element(),
        Pattern::None => d.into_any_element(),
    }
}

/// Text kept on one line, cut with an ellipsis. Not `truncate()`: gpui 0.2.2 keeps the layout of a
/// text that does not wrap from its first measure, at its natural width, and never cuts it; one
/// wrapped line is measured again when its width is known.
pub fn one_line<E: Styled>(element: E) -> E {
    element.overflow_hidden().line_clamp(1).text_ellipsis()
}

/// Two strokes from corner to corner of a `side` px square, each a thin parallelogram.
fn cross(side: f32, stroke: f32, ink: Hsla) -> impl IntoElement {
    canvas(|_, _, _| {}, move |b: Bounds<Pixels>, _, window: &mut Window, _| {
        let (x, y, s, w) = (b.origin.x, b.origin.y, b.size.width, px(stroke));
        for (from, to) in [((x, y), (x + s, y + s)), ((x + s, y), (x, y + s))] {
            let mut path = gpui::Path::new(point(from.0 - w / 2., from.1));
            path.line_to(point(from.0 + w / 2., from.1));
            path.line_to(point(to.0 + w / 2., to.1));
            path.line_to(point(to.0 - w / 2., to.1));
            window.paint_path(path, ink);
        }
    })
    .size(px(side))
}

/// What is ticked to be freed: an empty box, or a box with a cross. Drawn on the panel's colour,
/// so that it reads the same over every plot.
pub fn checkbox(on: bool, t: &Theme) -> Div {
    div()
        .size(px(13.))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .bg(t.panel)
        .border_1()
        .border_color(t.ink)
        .when(on, |d| d.child(cross(7., 1.6, t.ink)))
}

/// A small label in capitals, in the secondary colour.
pub fn caps(text: &str, t: &Theme) -> Div {
    div().font_family(MONO).text_size(px(10.5)).text_color(t.soft).child(text.to_uppercase())
}

/// A size in large figures, its unit set smaller.
pub fn figure(bytes: u64, height: f32, t: &Theme) -> Div {
    let (n, unit) = lang::size_parts(bytes);
    div()
        .flex()
        .items_baseline()
        .gap(px(6.))
        .child(div().font_family(DISPLAY).font_weight(FontWeight::LIGHT).text_size(px(height)).line_height(px(height * 1.05)).child(n))
        .child(div().text_size(px(height * 0.38)).text_color(t.soft).child(unit))
}

/// A button: `primary` is set in ink (the one action of a screen), the others are outlined.
/// Pressing it never moves the window, even inside a drag handle.
pub fn button(id: &'static str, text: impl Into<gpui::SharedString>, primary: bool, t: &Theme) -> Stateful<Div> {
    let accent = t.accent;
    div()
        .id(id)
        .debug_selector(move || id.into())
        .h(px(40.))
        .px(px(18.))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(13.))
        .font_weight(FontWeight::MEDIUM)
        .cursor_pointer()
        .when(primary, |d| d.bg(t.ink).text_color(t.panel).hover(move |s| s.bg(accent)))
        .when(!primary, |d| d.border_1().border_color(t.ink.opacity(0.75)).hover(move |s| s.border_color(accent).text_color(accent)))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .child(text.into())
}

/// Turns an element into a window handle, where the app draws its title bar (Linux): dragging
/// moves it, double-click maximizes it, right-click opens the system menu.
pub fn drag_window<E: InteractiveElement>(element: E) -> E {
    element
        .on_mouse_down(MouseButton::Left, |e, window, _| if e.click_count == 2 { window.zoom_window() } else { window.start_window_move() })
        .on_mouse_down(MouseButton::Right, |e, window, _| window.show_window_menu(e.position))
}

/// The three window buttons, drawn in ink like the plan: a line, a square, a cross.
pub fn window_buttons(window: &Window, t: &Theme) -> Div {
    let controls = window.window_controls();
    let ink = t.ink.opacity(0.8);
    let accent = t.accent;
    let slot = |id: &'static str| {
        div()
            .id(id)
            .size(px(26.))
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(move |s| s.bg(accent.opacity(0.15)))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .debug_selector(move || id.into())
    };
    let line = |w: f32, h: f32| div().w(px(w)).h(px(h)).bg(ink);
    div()
        .flex()
        .gap(px(4.))
        .when(controls.minimize, |d| d.child(slot("minimize").child(line(11., 1.5)).on_click(|_, window, _| window.minimize_window())))
        .when(controls.maximize, |d| d.child(slot("maximize").child(div().size(px(10.)).border_1().border_color(ink)).on_click(|_, window, _| window.zoom_window())))
        .child(slot("close").child(cross(10., 1.5, ink)).on_click(|_, window, _| window.remove_window()))
}
