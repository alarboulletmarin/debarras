//! Drawing the window: its frame where the app draws it (Linux), the shortcuts, then the screen
//! of the current stage and the help over it.

use gpui::{CursorStyle, Decorations, MouseButton, MouseMoveEvent, Pixels, Point, Size, div, prelude::*, px};

use super::*;
use crate::ui::{
    assets::SANS,
    widgets::{drag_window, window_buttons},
};

/// The transparent margin around a framed window: it holds the shadow and the resize edges.
pub(super) const SHADOW: Pixels = px(10.);

impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme;
        // On Linux the app provides the shadow, the rounded corners and the buttons; on macOS and
        // Windows gpui announces `Server` and the system does.
        let tiling = match window.window_decorations() {
            Decorations::Client { tiling } => Some(tiling),
            Decorations::Server => None,
        };
        let client = tiling.is_some();
        let framed = tiling.is_some_and(|t| !t.is_tiled()) && !window.is_maximized() && !window.is_fullscreen();
        let inset = if framed { SHADOW } else { px(0.) };
        window.set_client_inset(inset);
        let viewport = window.viewport_size();
        let room = (f32::from(viewport.width - inset * 2.), f32::from(viewport.height - inset * 2.));

        let screen = match self.stage {
            Stage::Plan => self.render_plan(room, cx).into_any_element(),
            _ => self.render_start(room, cx).into_any_element(),
        };
        let content = div()
            .id("content")
            .size_full()
            .relative()
            .overflow_hidden()
            .bg(t.paper)
            .text_color(t.ink)
            .font_family(SANS)
            .when(framed, |d| d.rounded(px(12.)).border_1().border_color(t.rule))
            .track_focus(&self.focus)
            .key_context("Debarras")
            .on_action(cx.listener(|this, _: &Up, _, cx| this.up(cx)))
            .on_action(cx.listener(|this, _: &NextTheme, window, cx| this.next_theme(window, cx)))
            .on_action(cx.listener(|this, _: &ToggleHelp, _, cx| {
                this.help = !this.help;
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &OpenFolder, window, cx| this.choose_folder(window, cx)))
            .on_action(cx.listener(|this, _: &Rescan, _, cx| this.rescan(cx)))
            .on_action(cx.listener(|this, _: &Reveal, _, cx| this.reveal(cx)))
            // A folder dropped anywhere is scanned.
            .on_drop(cx.listener(|this, paths: &gpui::ExternalPaths, _, cx| {
                if let Some(dir) = paths.paths().iter().find(|p| p.is_dir()) {
                    this.scan(dir.clone(), cx);
                }
            }))
            // The top strip moves the window, where the app draws its frame.
            .when(client, |d| d.child(drag_window(div().id("title-strip").absolute().top_0().left_0().w_full().h(px(28.)))))
            .child(screen)
            .when(client, |d| d.child(div().absolute().top(px(6.)).right(px(8.)).child(window_buttons(window, &t))))
            .when(self.help, |d| d.child(self.render_help(room, cx)));

        div()
            .size_full()
            .when(framed, |d| {
                d.p(inset)
                    .on_mouse_move(cx.listener(|this, e: &MouseMoveEvent, window, cx| {
                        let edge = resize_edge(e.position, window.viewport_size());
                        if edge != this.edge {
                            this.edge = edge;
                            cx.notify();
                        }
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, window, _| {
                            if let Some(edge) = this.edge {
                                window.start_window_resize(edge)
                            }
                        }),
                    )
                    .when_some(self.edge, |d, edge| d.cursor(cursor_of(edge)))
            })
            .child(content)
    }
}

impl Shell {
    /// The margin between the window's edge and its content, in which `room` and the tiles are measured.
    pub(super) fn inset(window: &Window) -> f32 {
        f32::from(window.client_inset().unwrap_or(px(0.)))
    }
}

fn cursor_of(edge: ResizeEdge) -> CursorStyle {
    match edge {
        ResizeEdge::Top | ResizeEdge::Bottom => CursorStyle::ResizeUpDown,
        ResizeEdge::Left | ResizeEdge::Right => CursorStyle::ResizeLeftRight,
        ResizeEdge::TopLeft | ResizeEdge::BottomRight => CursorStyle::ResizeUpLeftDownRight,
        ResizeEdge::TopRight | ResizeEdge::BottomLeft => CursorStyle::ResizeUpRightDownLeft,
    }
}

/// Window edge or corner under the pointer, within the margin.
pub(super) fn resize_edge(pos: Point<Pixels>, size: Size<Pixels>) -> Option<ResizeEdge> {
    let m = SHADOW;
    let (left, right) = (pos.x < m, pos.x > size.width - m);
    let (top, bottom) = (pos.y < m, pos.y > size.height - m);
    Some(match (top, bottom, left, right) {
        (true, _, true, _) => ResizeEdge::TopLeft,
        (true, _, _, true) => ResizeEdge::TopRight,
        (_, true, true, _) => ResizeEdge::BottomLeft,
        (_, true, _, true) => ResizeEdge::BottomRight,
        (true, ..) => ResizeEdge::Top,
        (_, true, ..) => ResizeEdge::Bottom,
        (_, _, true, _) => ResizeEdge::Left,
        (_, _, _, true) => ResizeEdge::Right,
        _ => return None,
    })
}
