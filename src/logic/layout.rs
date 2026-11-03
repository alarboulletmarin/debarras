//! Hierarchical squarified treemap (Bruls, Huizing & van Wijk, 2000).
//!
//! Output: a flat list of `Tile`s in absolute coordinates, in drawing order (parents before
//! children). The view only has to paint them.

use crate::logic::tree::{Category, Kind, Node, Reclaim};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect { x, y, w, h }
    }

    pub fn area(&self) -> f32 {
        self.w.max(0.) * self.h.max(0.)
    }

    /// The rectangle shrunk by these margins (top, right, bottom, left), never below zero.
    pub fn inset(&self, t: f32, r: f32, b: f32, l: f32) -> Rect {
        Rect::new(self.x + l, self.y + t, (self.w - l - r).max(0.), (self.h - t - b).max(0.))
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Opts {
    /// Deepest level drawn under the shown folder (1: its children only).
    pub max_depth: usize,
    /// Gutter between two sibling plots.
    pub gap: f32,
    /// Inner margin of an open folder.
    pub padding: f32,
    /// Height of the title band of an open folder.
    pub header: f32,
    /// Under this area (px²), an item goes into the « others » plot.
    pub min_area: f32,
    /// A folder is opened in place only if it is at least this wide and this high (px).
    pub min_open: f32,
}

impl Default for Opts {
    fn default() -> Self {
        Opts { max_depth: 3, gap: 6., padding: 6., header: 22., min_area: 900., min_open: 120. }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tile {
    /// Path from the shown folder; `None` for the « others » plot.
    pub path: Option<Vec<usize>>,
    pub rect: Rect,
    /// 1: a child of the shown folder.
    pub depth: usize,
    pub name: String,
    pub size: u64,
    pub kind: Kind,
    pub category: Category,
    pub reclaim: Reclaim,
    /// A folder drawn open: its children are inside.
    pub open: bool,
    /// How many items the « others » plot holds.
    pub grouped: usize,
}

impl Tile {
    pub fn is_dir(&self) -> bool {
        self.kind == Kind::Dir
    }
}

/// Squarify: one rectangle per value, in the order given. A value of zero (or less) gets an empty rectangle.
pub fn squarify(values: &[f64], bounds: Rect) -> Vec<Rect> {
    let mut out = vec![Rect::new(bounds.x, bounds.y, 0., 0.); values.len()];
    let total: f64 = values.iter().filter(|v| **v > 0.).sum();
    if total <= 0. || bounds.area() <= 0. {
        return out;
    }
    let mut order: Vec<usize> = (0..values.len()).filter(|&i| values[i] > 0.).collect();
    order.sort_by(|&a, &b| values[b].total_cmp(&values[a]));
    let scale = bounds.area() as f64 / total;
    let areas: Vec<f64> = order.iter().map(|&i| values[i] * scale).collect();

    let mut free = bounds;
    let (mut start, mut end) = (0, 0);
    while end < areas.len() {
        let side = free.w.min(free.h) as f64;
        if end == start || worst(&areas[start..=end], side) <= worst(&areas[start..end], side) {
            end += 1;
        } else {
            free = place_row(&areas[start..end], &order[start..end], free, &mut out);
            start = end;
        }
    }
    if start < end {
        place_row(&areas[start..end], &order[start..end], free, &mut out);
    }
    out
}

/// Worst aspect ratio of a row laid along a side of length `side`.
fn worst(row: &[f64], side: f64) -> f64 {
    if row.is_empty() {
        return f64::INFINITY;
    }
    let sum: f64 = row.iter().sum();
    let (max, min) = row.iter().fold((f64::MIN, f64::MAX), |(mx, mn), &a| (mx.max(a), mn.min(a)));
    let (s2, w2) = (sum * sum, side * side);
    (w2 * max / s2).max(s2 / (w2 * min))
}

fn place_row(areas: &[f64], idx: &[usize], free: Rect, out: &mut [Rect]) -> Rect {
    let sum: f64 = areas.iter().sum();
    let last = areas.len() - 1;
    // The last plot of a row ends on the free edge itself: floats never let a row spill over it.
    if free.w >= free.h {
        // A column on the left.
        let w = ((sum / free.h as f64) as f32).min(free.w);
        let mut y = free.y;
        for (k, &a) in areas.iter().enumerate() {
            let h = if k == last { free.y + free.h - y } else { (a / w as f64) as f32 };
            out[idx[k]] = Rect::new(free.x, y, w, h.max(0.));
            y += h;
        }
        Rect::new(free.x + w, free.y, (free.w - w).max(0.), free.h)
    } else {
        // A row at the top.
        let h = ((sum / free.w as f64) as f32).min(free.h);
        let mut x = free.x;
        for (k, &a) in areas.iter().enumerate() {
            let w = if k == last { free.x + free.w - x } else { (a / h as f64) as f32 };
            out[idx[k]] = Rect::new(x, free.y, w.max(0.), h);
            x += w;
        }
        Rect::new(free.x, free.y + h, free.w, (free.h - h).max(0.))
    }
}

/// Lays out the children of `root` (the shown folder) in `bounds`.
pub fn layout(root: &Node, bounds: Rect, opts: &Opts) -> Vec<Tile> {
    let mut tiles = Vec::new();
    lay(root, bounds, 1, &mut Vec::new(), opts, &mut tiles);
    tiles
}

fn lay(node: &Node, area: Rect, depth: usize, path: &mut Vec<usize>, opts: &Opts, tiles: &mut Vec<Tile>) {
    if node.children.is_empty() || area.area() <= 0. || node.size == 0 {
        return;
    }
    // The children are sorted: they are kept while their area would reach `min_area`, the rest
    // becomes one « others » plot. A single small one stays itself.
    let per_byte = area.area() as f64 / node.size as f64;
    let mut kept = node.children.iter().take_while(|c| (c.size as f64 * per_byte) as f32 >= opts.min_area).count();
    if node.children.len() - kept == 1 {
        kept += 1;
    }
    let rest = &node.children[kept..];
    let rest_size: u64 = rest.iter().map(|c| c.size).sum();

    let mut values: Vec<f64> = node.children[..kept].iter().map(|c| c.size as f64).collect();
    if !rest.is_empty() {
        values.push(rest_size as f64);
    }
    let g = opts.gap / 2.;
    for (i, r) in squarify(&values, area).into_iter().enumerate() {
        let rect = r.inset(g, g, g, g);
        if rect.w < 1. || rect.h < 1. {
            continue;
        }
        if i == kept {
            tiles.push(Tile {
                path: None,
                rect,
                depth,
                name: String::new(),
                size: rest_size,
                kind: Kind::Rest(rest.len() as u64),
                category: Category::Other,
                reclaim: Reclaim::No,
                open: false,
                grouped: rest.len(),
            });
            continue;
        }
        let child = &node.children[i];
        path.push(i);
        let open = depth < opts.max_depth && child.is_dir() && rect.w >= opts.min_open && rect.h >= opts.min_open;
        tiles.push(Tile {
            path: Some(path.clone()),
            rect,
            depth,
            name: child.name.clone(),
            size: child.size,
            kind: child.kind,
            category: child.category,
            reclaim: child.reclaim,
            open,
            grouped: 0,
        });
        if open {
            let inner = rect.inset(opts.header, opts.padding, opts.padding, opts.padding);
            lay(child, inner, depth + 1, path, opts, tiles);
        }
        path.pop();
    }
}

/// The deepest tile under the point (hover and click).
pub fn hit(tiles: &[Tile], x: f32, y: f32) -> Option<&Tile> {
    tiles.iter().filter(|t| t.rect.contains(x, y)).max_by_key(|t| t.depth)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::{sample, tree::Category::Other};

    fn overlap(a: Rect, b: Rect) -> bool {
        let ox = (a.x + a.w).min(b.x + b.w) - a.x.max(b.x);
        let oy = (a.y + a.h).min(b.y + b.h) - a.y.max(b.y);
        ox > 0.01 && oy > 0.01
    }

    fn inside(a: Rect, b: Rect) -> bool {
        a.x >= b.x - 0.01 && a.y >= b.y - 0.01 && a.x + a.w <= b.x + b.w + 0.01 && a.y + a.h <= b.y + b.h + 0.01
    }

    /// A small deterministic generator, for the random tests.
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            self.0 >> 33
        }
    }

    #[test]
    fn squarify_keeps_areas_inside_without_overlap() {
        let mut rng = Lcg(7);
        for round in 0..300 {
            let n = 1 + (rng.next() % 40) as usize;
            // Sizes spread over six orders of magnitude, as on a disk.
            let values: Vec<f64> = (0..n).map(|_| 10f64.powi((rng.next() % 6) as i32) * (1 + rng.next() % 9) as f64).collect();
            let bounds = Rect::new(10., 20., 50. + (rng.next() % 1500) as f32, 50. + (rng.next() % 900) as f32);
            let rects = squarify(&values, bounds);
            let total: f64 = values.iter().sum();
            for (i, (v, r)) in values.iter().zip(&rects).enumerate() {
                let expected = (*v / total) as f32 * bounds.area();
                // Within a thousandth, and a tenth of a pixel for the plots of one pixel (float drift on the edges).
                assert!((r.area() - expected).abs() <= expected * 1e-3 + 0.1, "round {round}: area of {i}");
                assert!(inside(*r, bounds), "round {round}: {i} leaves the bounds: {r:?} in {bounds:?}, values {values:?}");
                for (j, other) in rects.iter().enumerate().skip(i + 1) {
                    assert!(!overlap(*r, *other), "round {round}: {i} and {j} overlap");
                }
            }
        }
    }

    #[test]
    fn squarify_keeps_plots_near_square() {
        let values: Vec<f64> = (1..=20).rev().map(f64::from).collect();
        let rects = squarify(&values, Rect::new(0., 0., 1000., 700.));
        let worst = rects.iter().map(|r| (r.w / r.h).max(r.h / r.w)).fold(0., f32::max);
        assert!(worst < 4., "worst ratio {worst}");
    }

    #[test]
    fn squarify_survives_nothing_to_lay() {
        assert!(squarify(&[], Rect::new(0., 0., 10., 10.)).is_empty());
        let rects = squarify(&[0., 5.], Rect::new(0., 0., 10., 10.));
        assert_eq!(rects[0].area(), 0.);
        assert!((rects[1].area() - 100.).abs() < 0.5);
        assert!(squarify(&[1., 2.], Rect::new(0., 0., 0., 300.)).iter().all(|r| r.area() == 0.));
        assert!(squarify(&[0., 0.], Rect::new(0., 0., 10., 10.)).iter().all(|r| r.area() == 0.));
    }

    #[test]
    fn children_stay_inside_their_open_folder() {
        let root = sample::home();
        let tiles = layout(&root, Rect::new(0., 0., 1100., 760.), &Opts::default());
        assert!(tiles.iter().any(|t| t.depth == 2), "no folder opened in place");
        // The « others » plot has no path; the plots that do are checked against their folder.
        for t in tiles.iter().filter(|t| t.depth > 1 && t.path.is_some()) {
            let p = t.path.as_ref().unwrap();
            let parent = tiles.iter().find(|u| u.path.as_deref() == Some(&p[..p.len() - 1])).unwrap_or_else(|| panic!("no parent for {} {:?}", t.name, t.path));
            assert!(parent.open && inside(t.rect, parent.rect), "{} leaves {}", t.name, parent.name);
        }
        // Siblings never overlap.
        for (i, a) in tiles.iter().enumerate() {
            for b in tiles.iter().skip(i + 1).filter(|b| b.depth == a.depth) {
                assert!(!overlap(a.rect, b.rect), "{} and {} overlap", a.name, b.name);
            }
        }
    }

    #[test]
    fn small_items_are_grouped_but_a_single_one_stays() {
        // Each small one would be about 160 px², under `min_area`; together they make a plot that shows.
        let mut kids = vec![Node::file("gros", 1_000_000, Other)];
        kids.extend((0..500).map(|i| Node::file(format!("petit-{i}"), 400, Other)));
        let tiles = layout(&Node::dir("d", Other, kids), Rect::new(0., 0., 800., 600.), &Opts::default());
        let others = tiles.iter().find(|t| t.path.is_none()).expect("an « others » plot");
        assert_eq!((others.grouped, others.size, others.kind), (500, 200_000, Kind::Rest(500)));
        assert_eq!(tiles.len(), 2);

        let one = Node::dir("d", Other, vec![Node::file("gros", 1_000_000, Other), Node::file("seul", 1, Other)]);
        let tiles = layout(&one, Rect::new(0., 0., 800., 600.), &Opts::default());
        assert!(tiles.iter().all(|t| t.path.is_some()), "a single small item is not an « others » plot");
    }

    #[test]
    fn lays_out_nothing_in_no_room() {
        let root = sample::home();
        for bounds in [Rect::new(0., 0., 0., 0.), Rect::new(0., 0., 1., 1.), Rect::new(0., 0., 5000., 0.)] {
            assert!(layout(&root, bounds, &Opts::default()).iter().all(|t| t.rect.w >= 1. && t.rect.h >= 1.));
        }
        assert!(layout(&Node::dir("vide", Other, vec![]), Rect::new(0., 0., 800., 600.), &Opts::default()).is_empty());
    }

    #[test]
    fn a_folder_of_ten_thousand_files_stays_light() {
        let kids = (0..10_000).map(|i| Node::file(format!("f{i}"), 1 + i % 977, Other)).collect();
        let tiles = layout(&Node::dir("d", Other, kids), Rect::new(0., 0., 1200., 800.), &Opts::default());
        // Each plot is at least `min_area` (less the gutters), so few are drawn.
        assert!(tiles.len() < 1200, "{} tiles", tiles.len());
    }

    #[test]
    fn hit_finds_the_deepest_tile() {
        let root = sample::home();
        let tiles = layout(&root, Rect::new(0., 0., 1100., 760.), &Opts::default());
        // The middle of a plot that is not open finds that plot, whatever its depth.
        for leaf in tiles.iter().filter(|t| !t.open) {
            let (cx, cy) = (leaf.rect.x + leaf.rect.w / 2., leaf.rect.y + leaf.rect.h / 2.);
            assert_eq!(hit(&tiles, cx, cy), Some(leaf));
        }
        // The band of an open folder finds the folder.
        let open = tiles.iter().find(|t| t.open).unwrap();
        assert_eq!(hit(&tiles, open.rect.x + 2., open.rect.y + 2.), Some(open));
        assert!(hit(&tiles, -5., -5.).is_none());
        assert!(hit(&[], 1., 1.).is_none());
    }
}
