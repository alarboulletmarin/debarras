//! A made-up but plausible home folder (a developer's, on Linux), always the same: the demo
//! (`debarras --demo`), the screenshots and the tests use it.

use crate::{
    lang::tr,
    logic::{
        rules::Why,
        tree::{Category, Category::*, Node, Reclaim::*},
    },
};

const M: u64 = 1_000_000;

/// A small deterministic generator, to fill folders.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }

    fn range(&mut self, lo: u64, hi: u64) -> u64 {
        lo + self.next() % (hi - lo)
    }
}

fn files(rng: &mut Lcg, prefix: &str, ext: &str, n: usize, (lo, hi): (u64, u64), category: Category) -> Vec<Node> {
    (0..n).map(|i| Node::file(format!("{prefix}-{:03}.{ext}", i + 1), rng.range(lo, hi), category)).collect()
}

fn project(rng: &mut Lcg, name: &str, build: Option<(&str, u64)>) -> Node {
    let mut kids = vec![Node::dir("src", Dev, files(rng, "mod", "rs", 6, (2 * M, 30 * M), Dev)), Node::file(".git", rng.range(40 * M, 400 * M), Dev)];
    if let Some((dir, size)) = build {
        let why = if dir == "target" { Why::RustTarget } else { Why::NodeModules };
        kids.push(Node::file(dir, size, Dev).with(Safe, why));
    }
    Node::dir(name, Dev, kids)
}

pub fn home() -> Node {
    let mut r = Lcg(0xDEBA_77A5);
    let projects = Node::dir(
        tr("projects", "projets"),
        Dev,
        vec![
            project(&mut r, "atlas", Some(("target", 6_400 * M))),
            project(&mut r, "comet", Some(("target", 3_900 * M))),
            project(&mut r, "typeset", Some(("target", 5_100 * M))),
            project(&mut r, "harbor", Some(("node_modules", 1_150 * M))),
            project(&mut r, "ledger", Some(("node_modules", 820 * M))),
            project(&mut r, "quill", Some(("node_modules", 640 * M))),
            project(&mut r, "rampart", None),
            project(&mut r, "dada", None),
        ],
    );
    let cache = Node::dir(
        ".cache",
        Cache,
        vec![
            Node::file("browser", 2_300 * M, Cache),
            Node::file("pip", 1_800 * M, Cache),
            Node::file("editor", 900 * M, Cache),
            Node::file("thumbnails", 640 * M, Cache),
            Node::file("ms-playwright", 1_400 * M, Cache),
            Node::file("fontconfig", 12 * M, Cache),
        ],
    )
    .with(Safe, Why::UserCache);
    let cargo = Node::dir(".cargo", Dev, vec![Node::file("registry", 4_700 * M, Dev).with(Safe, Why::CargoRegistry), Node::file("bin", 380 * M, Dev)]);
    let local = Node::dir(
        ".local/share",
        Apps,
        vec![
            Node::dir(
                "ollama",
                Models,
                vec![
                    Node::file("coder-30b.gguf", 18_600 * M, Models),
                    Node::file("scout-q4.gguf", 11_200 * M, Models),
                    Node::file("embed-text.gguf", 274 * M, Models),
                ],
            )
            .with(Review, Why::Models),
            Node::dir(
                "flatpak",
                Apps,
                vec![
                    Node::file("org.gimp.GIMP", 1_100 * M, Apps),
                    Node::file("org.inkscape.Inkscape", 640 * M, Apps),
                    Node::file("runtimes", 4_200 * M, System),
                ],
            ),
            Node::file("Trash", 2_900 * M, Other).with(Safe, Why::Trash),
            Node::file("containers", 9_400 * M, Dev).with(Review, Why::Containers),
        ],
    );
    let pictures = Node::dir(
        tr("Pictures", "Images"),
        Media,
        vec![
            Node::dir("2024", Media, files(&mut r, "IMG", "heic", 40, (2 * M, 9 * M), Media)),
            Node::dir("2025", Media, files(&mut r, "IMG", "heic", 70, (2 * M, 9 * M), Media)),
            Node::dir("2026", Media, files(&mut r, "IMG", "heic", 90, (2 * M, 9 * M), Media)),
            Node::dir(tr("Races", "Courses"), Media, files(&mut r, "DSC", "jpg", 30, (6 * M, 14 * M), Media)),
        ],
    );
    let videos = Node::dir(
        tr("Videos", "Vidéos"),
        Media,
        vec![
            Node::file("half-marathon-2026.mp4", 7_800 * M, Media),
            Node::file("summer-coast.mov", 12_300 * M, Media),
            Node::file("timelapse-pass.mp4", 2_100 * M, Media),
            Node::file("screencasts", 1_600 * M, Media),
        ],
    );
    let downloads = Node::dir(
        tr("Downloads", "Téléchargements"),
        Documents,
        vec![
            Node::file("ubuntu-26.04-desktop-amd64.iso", 6_100 * M, Archives).with(Review, Why::DiskImage),
            Node::file("fedora-workstation-44.iso", 2_400 * M, Archives).with(Review, Why::DiskImage),
            Node::file("watch-export.zip", 1_300 * M, Archives),
            Node::dir("pdf", Documents, files(&mut r, "doc", "pdf", 50, (M / 2, 12 * M), Documents)),
        ],
    );
    let documents = Node::dir(
        "Documents",
        Documents,
        vec![
            Node::dir("admin", Documents, files(&mut r, "scan", "pdf", 60, (M / 3, 4 * M), Documents)),
            Node::dir("cv", Documents, files(&mut r, "cv", "typ", 8, (M / 10, 2 * M), Documents)),
            Node::dir("notes", Documents, files(&mut r, "note", "md", 40, (M / 5, 6 * M), Documents)),
        ],
    );
    Node::dir("~", Other, vec![projects, cache, cargo, local, pictures, videos, downloads, documents, Node::file(".mozilla", 1_900 * M, Apps)])
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_demo_has_room_to_free() {
        let home = super::home();
        assert!(home.size > 100_000_000_000);
        assert!(home.safe_total() > 20_000_000_000);
        // Always the same tree.
        assert_eq!(home.size, super::home().size);
    }
}
