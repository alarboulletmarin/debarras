//! The window at its smallest and at a large size: nothing overlaps, every plot stays in the plan.

use super::*;

#[gpui::test]
fn the_plan_fits_every_window_size(cx: &mut TestAppContext) {
    let (_root, shell, cx, _config) = open(cx, "tailles", Start::Demo);
    for (w, h) in [(900., 600.), (1440., 900.), (2560., 1440.)] {
        cx.simulate_resize(size(px(w), px(h)));
        cx.run_until_parked();
        let map = Shell::map_rect((w, h));
        let tiles = shell.read_with(cx, |s, _| s.tiles.clone());
        assert!(!tiles.is_empty(), "{w}×{h}: no plot");
        for t in &tiles {
            let r = t.rect;
            assert!(r.x >= map.x - 0.5 && r.y >= map.y - 0.5 && r.x + r.w <= map.x + map.w + 0.5 && r.y + r.h <= map.y + map.h + 0.5, "{w}×{h}: {} leaves the plan", t.name);
        }
        // The plan starts right of the panel and ends inside the window.
        assert!(map.x > panel::PANEL_W && map.x + map.w <= w && map.y + map.h <= h - 36., "{w}×{h}: {map:?}");
        let drawn = cx.debug_bounds("map").unwrap();
        assert_eq!((f32::from(drawn.size.width), f32::from(drawn.size.height)), (map.w, map.h), "{w}×{h}");
    }
}
