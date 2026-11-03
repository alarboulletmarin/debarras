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
}
