//! Small pieces drawn the same way everywhere: the fill of a plot (hatching, dots), the verdict's
//! tag, the labels in capitals and the large figures.

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

/// The colour of a verdict's word: blue-green, ochre, or the secondary grey for what stays.
pub fn verdict_color(reclaim: Reclaim, t: &Theme) -> Hsla {
    match reclaim {
        Reclaim::Safe => t.safe,
        Reclaim::Review => t.review,
        Reclaim::No => t.soft,
    }
}

/// The tag that names a verdict: a small swatch of its pattern and its word, in its colour, outlined.
/// `extra` follows the word (a count), or is empty.
pub fn verdict_tag(reclaim: Reclaim, extra: &str, t: &Theme) -> Div {
    let color = verdict_color(reclaim, t);
    let word = if extra.is_empty() { reclaim.label().to_uppercase() } else { format!("{} · {extra}", reclaim.label().to_uppercase()) };
    div()
        .flex_none()
        .h(px(20.))
        .pl(px(4.))
        .pr(px(6.))
        .flex()
        .items_center()
        .gap(px(6.))
        .bg(t.panel)
        .border_1()
        .border_color(color)
        .font_family(MONO)
        .text_size(px(10.5))
        .text_color(color)
        .child(small_swatch(Pattern::of(reclaim), color, t))
        .child(word)
}

/// A swatch of 14 × 10 px in `color`, its pattern drawn closer so that it reads at that size: in a tag
/// and before a name of the lists.
pub fn small_swatch(pattern: Pattern, color: Hsla, t: &Theme) -> Div {
    let (w, h) = (14., 10.);
    div()
        .relative()
        .w(px(w))
        .h(px(h))
        .flex_none()
        .bg(t.panel)
        .border_1()
        .border_color(color)
        .overflow_hidden()
        .children(match pattern {
            Pattern::Hatch => Some(div().absolute().size_full().bg(pattern_slash(color, 1., 3.)).into_any_element()),
            pattern => pattern_layer(pattern, color, (w, h)),
        })
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

/// A box partly ticked: some of what a folder holds, not all.
pub fn checkbox_some(t: &Theme) -> Div {
    checkbox(false, t).child(div().w(px(7.)).h(px(1.6)).bg(t.ink))
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
        .gap(px(4.))
        .child(div().font_family(DISPLAY).font_weight(FontWeight::LIGHT).text_size(px(height)).line_height(px(height * 1.04)).child(n))
        .child(div().text_size(px(height * 0.38)).text_color(t.soft).child(unit))
}

/// A button: `primary` is set in ink (the one action of a screen), the others are outlined.
/// Pressing it never moves the window, even inside a drag handle.
pub fn button(id: &'static str, text: impl Into<gpui::SharedString>, primary: bool, t: &Theme) -> Stateful<Div> {
    let accent = t.accent;
    base_button(id, text)
        .when(primary, |d| d.bg(t.ink).text_color(t.panel).hover(move |s| s.bg(accent)))
        .when(!primary, |d| d.border_1().border_color(t.ink.opacity(0.75)).hover(move |s| s.border_color(accent).text_color(accent)))
}

/// The button of what cannot be undone: outlined in the danger colour, filled with it under the pointer.
pub fn danger_button(id: &'static str, text: impl Into<gpui::SharedString>, t: &Theme) -> Stateful<Div> {
    let (danger, on) = (t.danger, t.panel);
    base_button(id, text).border_1().border_color(danger).text_color(danger).hover(move |s| s.bg(danger).text_color(on))
}

fn base_button(id: &'static str, text: impl Into<gpui::SharedString>) -> Stateful<Div> {
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
