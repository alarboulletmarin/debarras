//! Measures, run by hand: `cargo test --release --locked -- --ignored --nocapture`.

use std::time::Instant;

use super::*;
use crate::logic::tree::Category::Other;

/// A tree of `n` folders of ten files, three levels deep: a large home folder.
fn big_tree(n: usize) -> Node {
    let leaf = |i: usize| Node::dir(format!("d{i}"), Other, (0..10).map(|k| Node::file(format!("f{k}"), 1_000_000 + (i * 7 + k) as u64 * 13_331, Other)).collect());
    let mid = |j: usize| Node::dir(format!("m{j}"), Other, (0..n / 100).map(|i| leaf(j * 1000 + i)).collect());
    Node::dir("/big", Other, (0..100).map(mid).collect())
}

/// Average time of one pointer move over the plan, the plots included.
fn hover_cost(cx: &mut VisualTestContext, shell: &Entity<Shell>) -> f64 {
    let map = Shell::map_rect((1440., 900.));
    let start = Instant::now();
    let steps = 300;
    for step in 0..steps {
        let at = point(px(map.x + map.w * (step % 60) as f32 / 60.), px(map.y + map.h * ((step * 7) % 60) as f32 / 60.));
        cx.simulate_mouse_move(at, None, Modifiers::none());
        cx.run_until_parked();
    }
    let _ = shell;
    start.elapsed().as_secs_f64() * 1000. / steps as f64
}

#[gpui::test]
#[ignore]
fn bench_hover(cx: &mut TestAppContext) {
    let (_root, shell, cx, _config) = open(cx, "bench-survol", Start::Demo);
    println!("demo: {:.3} ms per pointer move", hover_cost(cx, &shell));
    let built = Instant::now();
    let disk = Disk::new(big_tree(100_000), Source::Demo, 1_000_000, 0, Duration::ZERO);
    println!("100,000 folders: suggestions in {:.1} ms", built.elapsed().as_secs_f64() * 1000.);
    shell.update(cx, |s, cx| s.set_disk(disk, cx));
    cx.run_until_parked();
    println!("100,000 folders: {:.3} ms per pointer move", hover_cost(cx, &shell));
}

/// A real scan of the home folder (or `DEBARRAS_BENCH_DIR`): the second run, with the system's caches warm.
#[test]
#[ignore]
fn bench_scan() {
    use crate::platform::scan::{Progress, scan};
    let dir = std::env::var_os("DEBARRAS_BENCH_DIR").map(std::path::PathBuf::from).or_else(crate::platform::home_dir).expect("a folder to scan");
    for run in ["cold or warm", "warm"] {
        let progress = Progress::default();
        let start = Instant::now();
        let root = scan(&dir, &progress).expect("scan");
        let took = start.elapsed();
        let files = progress.files.load(std::sync::atomic::Ordering::Relaxed);
        println!("scan ({run}): {files} files, {} ({} bytes) in {took:.2?}", crate::lang::size(root.size), root.size);
        let built = Instant::now();
        let disk = Disk::new(root, Source::Demo, files, 0, took);
        println!("suggestions in {:.1} ms", built.elapsed().as_secs_f64() * 1000.);
        // The panel's tree, built at every frame: closed, then with every folder open.
        let items: Vec<Vec<usize>> = disk.suggestions.iter().map(|s| s.path.clone()).collect();
        let tree = Instant::now();
        let top = crate::logic::outline::outline(&disk.root, &items, &Default::default());
        let closed = tree.elapsed();
        let mut open: std::collections::HashSet<Vec<usize>> = Default::default();
        loop {
            let rows = crate::logic::outline::outline(&disk.root, &items, &open);
            let before = open.len();
            open.extend(rows.into_iter().filter(|r| r.open == Some(false)).map(|r| r.path));
            if open.len() == before {
                break;
            }
        }
        let tree = Instant::now();
        let all = crate::logic::outline::outline(&disk.root, &items, &open);
        println!("tree of {} items: {} lines closed in {closed:.1?}, {} lines open in {:.1?}", items.len(), top.len(), all.len(), tree.elapsed());
    }
}
