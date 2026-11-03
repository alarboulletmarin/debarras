//! Reading a folder from the disk into a `Node` tree, in parallel (one rayon task per folder).
//!
//! The scan stays on the file system it starts on (no `/proc`, no mounted disks), never follows a
//! link, and counts what it cannot read instead of stopping. Files under `SMALL` are summed per
//! folder into one plot: a tree of millions of files stays a few megabytes.

use std::{
    fs, io,
    path::Path,
    sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed},
};
#[cfg(unix)]
use std::{collections::HashSet, sync::Mutex};

use rayon::prelude::*;

use crate::logic::{
    rules,
    tree::{Category, Kind, Node},
};

/// Below this size a file is not kept by name.
pub const SMALL: u64 = 1_000_000;

/// Shared with the window while the scan runs: what has been read so far, and the request to stop.
#[derive(Default)]
pub struct Progress {
    pub files: AtomicU64,
    pub bytes: AtomicU64,
    /// Entries that could not be read (permission denied, vanished meanwhile).
    pub errors: AtomicU64,
    pub cancel: AtomicBool,
    /// Files with several hard links already counted, by (device, inode): each counts once.
    #[cfg(unix)]
    linked: Mutex<HashSet<(u64, u64)>>,
}

/// Scans `root`. An error if it cannot be read or is not a folder, or if the scan was cancelled
/// (`ErrorKind::Interrupted`).
pub fn scan(root: &Path, progress: &Progress) -> io::Result<Node> {
    let meta = fs::metadata(root)?;
    if !meta.is_dir() {
        return Err(io::ErrorKind::NotADirectory.into());
    }
    fs::read_dir(root)?;
    let mut node = walk(root, String::new(), "", device(&meta), progress);
    if progress.cancel.load(Relaxed) {
        return Err(io::ErrorKind::Interrupted.into());
    }
    node.name = root.display().to_string();
    node.finish();
    Ok(node)
}

#[cfg(unix)]
fn device(meta: &fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    meta.dev()
}

// ponytail: on Windows a scan may cross into a mounted folder; volume serial numbers need an
// unstable API or a Win32 call.
#[cfg(not(unix))]
fn device(_: &fs::Metadata) -> u64 {
    0
}

/// Bytes really taken on the disk (allocated blocks) where the system tells them: a sparse file
/// counts for what it uses. A file with several hard links counts once, at the first link the scan
/// meets, as `du` does: package stores link the same files into many folders.
#[cfg(unix)]
fn disk_size(meta: &fs::Metadata, progress: &Progress) -> u64 {
    use std::os::unix::fs::MetadataExt;
    let first = meta.nlink() < 2 || progress.linked.lock().map_or(true, |mut seen| seen.insert((meta.dev(), meta.ino())));
    if first { meta.blocks() * 512 } else { 0 }
}

// ponytail: on Windows, sizes are file lengths and a hard-linked file counts once per link; the
// file index needs a handle per file.
#[cfg(not(unix))]
fn disk_size(meta: &fs::Metadata, _: &Progress) -> u64 {
    meta.len()
}

fn walk(path: &Path, name: String, parent: &str, dev: u64, progress: &Progress) -> Node {
    let mut node = Node::dir(name, Category::Other, Vec::new());
    if let Some(verdict) = rules::classify_dir(&node.name, parent) {
        node.category = verdict.category;
        node.reclaim = verdict.reclaim;
        node.why = verdict.why;
    }
    if progress.cancel.load(Relaxed) {
        return node;
    }
    let Ok(entries) = fs::read_dir(path) else {
        progress.errors.fetch_add(1, Relaxed);
        return node;
    };
    let mut dirs = Vec::new();
    // The small files: how many, and their bytes by family, so that their plot takes the main one.
    let (mut small, mut small_by) = (0u64, [0u64; Category::ALL.len()]);
    let (mut files, mut bytes) = (0, 0);
    for entry in entries {
        let Ok(entry) = entry else {
            progress.errors.fetch_add(1, Relaxed);
            continue;
        };
        // The entry's own type and metadata: a link is not followed.
        let (Ok(kind), Ok(meta)) = (entry.file_type(), entry.metadata()) else {
            progress.errors.fetch_add(1, Relaxed);
            continue;
        };
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            if device(&meta) == dev {
                dirs.push(entry);
            }
            continue;
        }
        let size = disk_size(&meta, progress);
        files += 1;
        bytes += size;
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let category = rules::classify_file(&file_name);
        if size < SMALL {
            small += 1;
            small_by[Category::ALL.iter().position(|c| *c == category).unwrap_or(0)] += size;
            continue;
        }
        let mut file = Node::file(file_name.as_str(), size, category);
        if let Some(why) = rules::review_file(&file_name, size) {
            file = file.with(crate::logic::tree::Reclaim::Review, why);
        }
        node.children.push(file);
    }
    progress.files.fetch_add(files, Relaxed);
    progress.bytes.fetch_add(bytes, Relaxed);
    if small > 0 {
        let main = (0..small_by.len()).max_by_key(|&i| small_by[i]).unwrap_or(0);
        let mut rest = Node::file(String::new(), small_by.iter().sum(), Category::ALL[main]);
        rest.kind = Kind::Rest(small);
        node.children.push(rest);
    }
    let parent = node.name.clone();
    node.children.extend(
        dirs.par_iter()
            .map(|entry| walk(&entry.path(), entry.file_name().to_string_lossy().into_owned(), &parent, dev, progress))
            .collect::<Vec<_>>(),
    );
    node.finish();
    // A folder no rule knows takes the family of its largest item.
    if node.category == Category::Other
        && let Some(largest) = node.children.first()
    {
        node.category = largest.category;
    }
    node
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::tree::Reclaim;

    fn temp(name: &str) -> std::path::PathBuf {
        let base = std::env::temp_dir().join(format!("debarras-scan-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        base
    }

    fn child<'a>(node: &'a Node, name: &str) -> &'a Node {
        node.children.iter().find(|c| c.name == name).unwrap_or_else(|| panic!("{name} missing in {}", node.name))
    }

    #[test]
    fn reads_a_tree_with_its_rules() {
        let base = temp("arbre");
        fs::create_dir_all(base.join("proj/node_modules/pkg/deep")).unwrap();
        fs::create_dir_all(base.join("photos été 🌞")).unwrap();
        fs::create_dir_all(base.join("vide")).unwrap();
        fs::write(base.join("proj/node_modules/pkg/deep/a.js"), vec![1u8; 2_000_000]).unwrap();
        fs::write(base.join("photos été 🌞/x.jpg"), vec![1u8; 3_000_000]).unwrap();
        fs::write(base.join("lisez-moi.md"), vec![1u8; 10_000]).unwrap();
        fs::write(base.join("zero"), b"").unwrap();

        let progress = Progress::default();
        let root = scan(&base, &progress).unwrap();
        assert_eq!(progress.files.load(Relaxed), 4);
        assert_eq!(root.name, base.display().to_string());
        let modules = child(child(&root, "proj"), "node_modules");
        assert_eq!(modules.reclaim, Reclaim::Safe);
        assert!(modules.size >= 2_000_000);
        // Deep folders are kept whole: the plan can open them.
        assert!(child(child(modules, "pkg"), "deep").is_dir());
        let photos = child(&root, "photos été 🌞");
        assert_eq!(photos.category, Category::Media, "a folder takes the family of its largest item");
        assert!(root.children.iter().all(|c| c.name != "vide"), "an empty folder weighs nothing");
        // The small files are one plot that says how many.
        let rest = root.children.iter().find(|c| matches!(c.kind, Kind::Rest(_))).unwrap();
        assert_eq!(rest.kind, Kind::Rest(2));
        assert_eq!(root.suggestions().len(), 1);
        assert_eq!(root.size, root.children.iter().map(|c| c.size).sum::<u64>());
        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn refuses_what_is_not_a_folder() {
        let base = temp("refus");
        fs::write(base.join("fichier"), b"x").unwrap();
        let progress = Progress::default();
        assert_eq!(scan(&base.join("fichier"), &progress).unwrap_err().kind(), io::ErrorKind::NotADirectory);
        assert_eq!(scan(&base.join("absent"), &progress).unwrap_err().kind(), io::ErrorKind::NotFound);
        // An empty folder is a plan with nothing in it.
        fs::create_dir(base.join("vide")).unwrap();
        let empty = scan(&base.join("vide"), &progress).unwrap();
        assert_eq!((empty.size, empty.children.len()), (0, 0));
        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn stops_when_asked() {
        let base = temp("arret");
        fs::create_dir_all(base.join("a/b")).unwrap();
        let progress = Progress::default();
        progress.cancel.store(true, Relaxed);
        assert_eq!(scan(&base, &progress).unwrap_err().kind(), io::ErrorKind::Interrupted);
        fs::remove_dir_all(&base).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn follows_no_link_and_counts_what_it_cannot_read() {
        use std::os::unix::fs::PermissionsExt;
        let base = temp("liens");
        fs::create_dir_all(base.join("dedans")).unwrap();
        fs::write(base.join("dedans/gros"), vec![1u8; 2_000_000]).unwrap();
        // A link to the folder and one to itself: neither is followed, nothing is counted twice.
        std::os::unix::fs::symlink(base.join("dedans"), base.join("lien")).unwrap();
        std::os::unix::fs::symlink(&base, base.join("boucle")).unwrap();
        fs::create_dir(base.join("interdit")).unwrap();
        fs::write(base.join("interdit/secret"), vec![1u8; 2_000_000]).unwrap();
        fs::set_permissions(base.join("interdit"), fs::Permissions::from_mode(0o000)).unwrap();

        // Root reads everything: the refusal is only expected when the system refuses.
        let denied = fs::read_dir(base.join("interdit")).is_err();

        let progress = Progress::default();
        let root = scan(&base, &progress).unwrap();
        fs::set_permissions(base.join("interdit"), fs::Permissions::from_mode(0o755)).unwrap();
        assert!(root.children.iter().all(|c| c.name != "lien" && c.name != "boucle"));
        assert_eq!(progress.errors.load(Relaxed), u64::from(denied));
        assert_eq!(root.size >= 4_000_000, !denied);
        fs::remove_dir_all(&base).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn counts_a_hard_linked_file_once() {
        let base = temp("liens-physiques");
        fs::create_dir_all(base.join("a")).unwrap();
        fs::create_dir_all(base.join("b/c")).unwrap();
        fs::write(base.join("a/gros"), vec![1u8; 2_000_000]).unwrap();
        fs::write(base.join("seul"), vec![1u8; 3_000_000]).unwrap();
        for link in ["a/gros-2", "b/gros", "b/c/gros"] {
            fs::hard_link(base.join("a/gros"), base.join(link)).unwrap();
        }
        let one = |p: &str| {
            use std::os::unix::fs::MetadataExt;
            fs::metadata(base.join(p)).unwrap().blocks() * 512
        };

        let progress = Progress::default();
        let root = scan(&base, &progress).unwrap();
        assert_eq!(progress.files.load(Relaxed), 5, "every link is a file");
        assert_eq!(root.size, one("a/gros") + one("seul"), "four links, one copy on the disk");
        assert_eq!(progress.bytes.load(Relaxed), root.size);
        // The copy shows where the scan met it first; the other links weigh nothing and are not drawn.
        fn drawn(n: &Node) -> usize {
            usize::from(!n.is_dir() && n.name.starts_with("gros")) + n.children.iter().map(drawn).sum::<usize>()
        }
        assert_eq!(drawn(&root), 1);
        fs::remove_dir_all(&base).unwrap();
    }
}
