//! The tree of sizes: folders and files, each with its family (`Category`) and what can be done
//! with it (`Reclaim`).

use std::path::PathBuf;

use crate::{lang::tr, logic::rules::Why};

/// The broad family of an item: it tints its plot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    System,
    Apps,
    Dev,
    Media,
    Documents,
    Cache,
    Archives,
    Models,
    Other,
}

impl Category {
    pub const ALL: [Category; 9] = [
        Category::System,
        Category::Apps,
        Category::Dev,
        Category::Media,
        Category::Documents,
        Category::Cache,
        Category::Archives,
        Category::Models,
        Category::Other,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Category::System => tr("System", "Système"),
            Category::Apps => tr("Applications", "Applications"),
            Category::Dev => tr("Development", "Développement"),
            Category::Media => tr("Media", "Médias"),
            Category::Documents => tr("Documents", "Documents"),
            Category::Cache => tr("Caches", "Caches"),
            Category::Archives => tr("Archives", "Archives"),
            Category::Models => tr("AI models", "Modèles d'IA"),
            Category::Other => tr("Other", "Divers"),
        }
    }
}

/// What can be done with an item. Ordered: `Safe` is the most worth freeing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Reclaim {
    /// To keep.
    No,
    /// Large, but the user judges (disk images, models, containers…).
    Review,
    /// Rebuilt without loss (caches, `target/`, `node_modules/`…).
    Safe,
}

impl Reclaim {
    /// The word that doubles the pattern, so that the plan reads without a legend.
    pub fn label(self) -> &'static str {
        match self {
            Reclaim::Safe => tr("Safe to remove", "Sans risque"),
            Reclaim::Review => tr("Check first", "À vérifier"),
            Reclaim::No => tr("Keep", "À garder"),
        }
    }
}

/// What a node stands for on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    File,
    Dir,
    /// The small files of a folder, summed into one plot: how many there are.
    Rest(u64),
}

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    /// Bytes taken on disk; for a folder, the sum of its children.
    pub size: u64,
    pub kind: Kind,
    pub category: Category,
    pub reclaim: Reclaim,
    /// Why it can be freed, shown to the user.
    pub why: Option<Why>,
    /// Largest first (`finish`).
    pub children: Vec<Node>,
}

/// The paths that are not inside another of them, in order: freeing a folder frees what it holds,
/// so what it holds is neither counted nor deleted twice.
pub fn outermost<'a>(paths: impl IntoIterator<Item = &'a Vec<usize>>) -> Vec<Vec<usize>> {
    let mut paths: Vec<&Vec<usize>> = paths.into_iter().collect();
    paths.sort();
    let mut kept: Vec<Vec<usize>> = Vec::new();
    for path in paths {
        // Sorted, a folder comes right before what it holds.
        if kept.last().is_none_or(|k| !path.starts_with(k)) {
            kept.push(path.clone());
        }
    }
    kept
}

impl Node {
    pub fn file(name: impl Into<String>, size: u64, category: Category) -> Self {
        Node { name: name.into(), size, kind: Kind::File, category, reclaim: Reclaim::No, why: None, children: Vec::new() }
    }

    /// A folder: its size is the sum of its children, sorted largest first.
    pub fn dir(name: impl Into<String>, category: Category, children: Vec<Node>) -> Self {
        let mut node = Node { kind: Kind::Dir, children, ..Node::file(name, 0, category) };
        node.finish();
        node
    }

    pub fn with(mut self, reclaim: Reclaim, why: Why) -> Self {
        self.reclaim = reclaim;
        self.why = Some(why);
        self
    }

    pub fn is_dir(&self) -> bool {
        self.kind == Kind::Dir && !self.children.is_empty()
    }

    /// The node reached by following child indices from here.
    pub fn at(&self, path: &[usize]) -> Option<&Node> {
        path.iter().try_fold(self, |node, &i| node.children.get(i))
    }

    fn at_mut(&mut self, path: &[usize]) -> Option<&mut Node> {
        path.iter().try_fold(self, |node, &i| node.children.get_mut(i))
    }

    /// Takes the items at `paths` (from here) out of the tree, as after deleting them from the disk,
    /// then sums and sorts again: the indices of what is left change. A path that leads nowhere is skipped.
    pub fn remove(&mut self, paths: &[Vec<usize>]) {
        let mut paths = paths.to_vec();
        // The last sibling first: removing it leaves the indices of the others as they were.
        paths.sort_unstable_by(|a, b| b.cmp(a));
        paths.dedup();
        for path in paths {
            let Some((&last, parent)) = path.split_last() else { continue };
            if let Some(parent) = self.at_mut(parent).filter(|p| last < p.children.len()) {
                parent.children.remove(last);
            }
        }
        self.finish();
    }

    /// Sums the folders' sizes, drops what weighs nothing, sorts largest first; ties by name, so that
    /// the plan does not change between two scans of the same disk. Returns the size.
    pub fn finish(&mut self) -> u64 {
        if self.kind == Kind::Dir {
            self.size = self.children.iter_mut().map(Node::finish).sum();
            self.children.retain(|c| c.size > 0);
            self.children.sort_by(|a, b| b.size.cmp(&a.size).then_with(|| a.name.cmp(&b.name)));
        }
        self.size
    }

    /// The items worth freeing, each counted once: the highest of the tree (a cache inside a cache is
    /// not counted again). Safe ones first, then by size.
    pub fn suggestions(&self) -> Vec<(Vec<usize>, &Node)> {
        fn walk<'a>(node: &'a Node, path: &mut Vec<usize>, out: &mut Vec<(Vec<usize>, &'a Node)>) {
            for (i, child) in node.children.iter().enumerate() {
                path.push(i);
                if child.reclaim != Reclaim::No {
                    out.push((path.clone(), child));
                } else {
                    walk(child, path, out);
                }
                path.pop();
            }
        }
        let mut out = Vec::new();
        walk(self, &mut Vec::new(), &mut out);
        out.sort_by(|a, b| b.1.reclaim.cmp(&a.1.reclaim).then(b.1.size.cmp(&a.1.size)));
        out
    }

    /// Where `names` (from here, this node's own excluded) lead now; as deep as they still go.
    pub fn find(&self, names: &[String]) -> Vec<usize> {
        let mut path = Vec::new();
        let mut node = self;
        for name in names {
            let Some(i) = node.children.iter().position(|c| &c.name == name && c.is_dir()) else { break };
            path.push(i);
            node = &node.children[i];
        }
        path
    }

    /// Where the item named by `names` (from here) is now; empty when it is not there any more.
    pub fn find_item(&self, names: &[String]) -> Vec<usize> {
        let mut path = Vec::new();
        let mut node = self;
        for name in names {
            let Some(i) = node.children.iter().position(|c| &c.name == name) else { return Vec::new() };
            path.push(i);
            node = &node.children[i];
        }
        path
    }

    /// What can be freed without loss (`Reclaim::Safe`), in bytes.
    #[cfg(test)]
    pub fn safe_total(&self) -> u64 {
        self.suggestions().iter().filter(|(_, n)| n.reclaim == Reclaim::Safe).map(|(_, n)| n.size).sum()
    }

    /// The names from here down to `path`, this node's own first.
    pub fn names(&self, path: &[usize]) -> Vec<&str> {
        let mut names = vec![self.name.as_str()];
        let mut node = self;
        for &i in path {
            let Some(child) = node.children.get(i) else { break };
            names.push(&child.name);
            node = child;
        }
        names
    }

    /// The short name of a deep item, with its parent: `bref/target` rather than `target`.
    pub fn short_name(&self, path: &[usize]) -> String {
        match self.names(path).as_slice() {
            [only] => only.to_string(),
            [_, one] => one.to_string(),
            [.., parent, last] => format!("{parent}/{last}"),
            [] => String::new(),
        }
    }

    /// Where an item lives on disk, when this node is the scanned folder (its name is its path).
    /// `None` for the small files summed together, which are not one file.
    pub fn disk_path(&self, path: &[usize]) -> Option<PathBuf> {
        let node = self.at(path)?;
        if matches!(node.kind, Kind::Rest(_)) {
            return None;
        }
        Some(self.names(path).into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Category::Other;

    #[test]
    fn a_folder_sums_and_sorts_its_children() {
        let d = Node::dir("d", Other, vec![Node::file("a", 1, Other), Node::file("b", 5, Other), Node::file("c", 3, Other), Node::file("vide", 0, Other)]);
        assert_eq!(d.size, 9);
        let names: Vec<_> = d.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["b", "c", "a"], "largest first, and what weighs nothing is gone");
        // Ties are ordered by name: two scans of the same disk draw the same plan.
        let tie = Node::dir("t", Other, vec![Node::file("z", 2, Other), Node::file("a", 2, Other)]);
        assert_eq!(tie.children[0].name, "a");
    }

    #[test]
    fn an_empty_folder_is_no_folder_to_open() {
        let empty = Node::dir("vide", Other, vec![]);
        assert_eq!(empty.size, 0);
        assert!(!empty.is_dir());
        assert!(empty.suggestions().is_empty());
        assert_eq!(empty.safe_total(), 0);
    }

    #[test]
    fn suggestions_count_each_item_once() {
        let inner = Node::file("inner", 10, Category::Cache).with(Reclaim::Safe, Why::UserCache);
        let outer = Node::dir("outer", Category::Cache, vec![inner]).with(Reclaim::Safe, Why::UserCache);
        let review = Node::file("disque.iso", 50, Category::Archives).with(Reclaim::Review, Why::DiskImage);
        let root = Node::dir("/", Other, vec![outer, review]);
        let found = root.suggestions();
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].1.name, "outer", "safe items first, even when smaller");
        assert_eq!(root.safe_total(), 10);
    }

    #[test]
    fn names_an_item_by_its_path() {
        let root = Node::dir("/home/moi", Other, vec![Node::dir("projets", Other, vec![Node::dir("bref", Other, vec![Node::file("target", 4, Other)])])]);
        assert_eq!(root.short_name(&[]), "/home/moi");
        assert_eq!(root.short_name(&[0]), "projets");
        assert_eq!(root.short_name(&[0, 0, 0]), "bref/target");
        assert_eq!(root.disk_path(&[0, 0, 0]).unwrap(), PathBuf::from("/home/moi/projets/bref/target"));
        assert!(root.at(&[3]).is_none() && root.disk_path(&[3]).is_none());
        let rest = Node::dir("/r", Other, vec![Node { kind: Kind::Rest(40), ..Node::file("40", 9, Other) }]);
        assert!(rest.disk_path(&[0]).is_none(), "small files summed together are not one file");
    }

    #[test]
    fn a_ticked_folder_covers_what_it_holds() {
        let p = |v: &[usize]| v.to_vec();
        let ticked = [p(&[1, 0, 2]), p(&[0]), p(&[1, 0]), p(&[0, 3]), p(&[1, 0]), p(&[2]), p(&[1, 1])];
        assert_eq!(outermost(&ticked), [p(&[0]), p(&[1, 0]), p(&[1, 1]), p(&[2])]);
        // [1] is not inside [10]: a prefix of the indices, not of their digits.
        assert_eq!(outermost(&[p(&[1]), p(&[10, 0])]), [p(&[1]), p(&[10, 0])]);
        assert!(outermost(&[]).is_empty());
    }

    #[test]
    fn removing_items_sums_and_sorts_again() {
        let f = |name: &str, size| Node::file(name, size, Other);
        let mut root = Node::dir("/", Other, vec![Node::dir("a", Other, vec![f("a1", 50), f("a2", 40), f("a3", 30)]), Node::dir("b", Other, vec![f("b1", 60)]), f("c", 70)]);
        // Sorted: a (120), c (70), b (60). Two siblings of `a`, the whole of `b`, and paths leading nowhere.
        let a = root.find(&["a".into()]);
        let b = root.find(&["b".into()]);
        assert_eq!((a.clone(), b.clone()), (vec![0], vec![2]));
        root.remove(&[vec![0, 0], vec![0, 2], b, vec![9], vec![0, 9], vec![]]);
        assert_eq!(root.size, 110);
        let names: Vec<_> = root.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["c", "a"], "b is gone with its only file, a is now smaller than c");
        assert_eq!(root.at(&root.find(&["a".into()])).unwrap().children.len(), 1);
        // A name no longer there: as deep as the path still goes.
        assert_eq!(root.find(&["a".into(), "gone".into()]), [1]);
        assert_eq!(root.find(&["c".into()]), Vec::<usize>::new(), "a file is not a folder to show");
        assert_eq!(root.find_item(&["c".into()]), [0], "but it is an item");
        assert_eq!(root.find_item(&["a".into(), "a2".into()]), [1, 0]);
        assert!(root.find_item(&["b".into(), "b1".into()]).is_empty(), "removed");
    }
}
