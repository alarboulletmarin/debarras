//! The panel's lists as a tree: the items grouped under the folders that hold them, so that thousands
//! of them read as a few lines. A folder with a single way down is one line (`projets/bref`).

use std::collections::{BTreeMap, HashSet};

use super::tree::Node;

/// A line of the tree: a folder holding items, or an item.
#[derive(Debug, PartialEq)]
pub struct Row {
    /// From the root of the disk.
    pub path: Vec<usize>,
    /// The names from the line above, joined: `projets/bref`.
    pub label: String,
    pub depth: usize,
    pub size: u64,
    /// The items it holds: itself for an item.
    pub items: Vec<Vec<usize>>,
    /// `None` for an item; whether a folder shows what it holds.
    pub open: Option<bool>,
}

#[derive(Default)]
struct Branch {
    item: bool,
    kids: BTreeMap<usize, Branch>,
}

impl Branch {
    fn items(&self, path: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if self.item {
            out.push(path.clone());
        }
        for (&i, kid) in &self.kids {
            path.push(i);
            kid.items(path, out);
            path.pop();
        }
    }
}

/// The lines shown of the tree of `items` (paths from `root`, none inside another), largest first.
/// `flipped` holds the folders the user opened; a folder alone at the top is open unless flipped.
pub fn outline(root: &Node, items: &[Vec<usize>], flipped: &HashSet<Vec<usize>>) -> Vec<Row> {
    let mut top = Branch::default();
    for path in items {
        let mut at = &mut top;
        for &i in path {
            at = at.kids.entry(i).or_default();
        }
        at.item = true;
    }
    let mut rows = Vec::new();
    level(root, &top, &[], 0, flipped, &mut rows);
    rows
}

fn level(root: &Node, branch: &Branch, path: &[usize], depth: usize, flipped: &HashSet<Vec<usize>>, rows: &mut Vec<Row>) {
    let alone = depth == 0 && branch.kids.len() == 1;
    let mut lines: Vec<(Row, &Branch)> = branch
        .kids
        .iter()
        .map(|(&i, mut kid)| {
            let mut at = path.to_vec();
            at.push(i);
            let mut names = vec![root.at(&at).map_or("", |n| n.name.as_str())];
            // A folder with a single way down is one line.
            while !kid.item && kid.kids.len() == 1 {
                let (&j, next) = kid.kids.iter().next().unwrap();
                at.push(j);
                names.push(root.at(&at).map_or("", |n| n.name.as_str()));
                kid = next;
            }
            let mut items = Vec::new();
            kid.items(&mut at.clone(), &mut items);
            let size = items.iter().filter_map(|p| root.at(p)).map(|n| n.size).sum();
            let open = (!kid.item).then(|| flipped.contains(&at) != alone);
            (Row { label: names.join("/"), path: at, depth, size, items, open }, kid)
        })
        .collect();
    lines.sort_by(|a, b| b.0.size.cmp(&a.0.size).then_with(|| a.0.path.cmp(&b.0.path)));
    for (row, kid) in lines {
        let (open, at) = (row.open == Some(true), row.path.clone());
        rows.push(row);
        if open {
            level(root, kid, &at, depth + 1, flipped, rows);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::tree::Category::Dev;

    fn f(name: &str, size: u64) -> Node {
        Node::file(name, size, Dev)
    }

    /// home/{projets/{bref/target, zoned/node_modules, seul/x/target}, .cache}
    fn home() -> Node {
        let mut root = Node::dir(
            "home",
            Dev,
            vec![
                Node::dir(
                    "projets",
                    Dev,
                    vec![
                        Node::dir("bref", Dev, vec![f("target", 50)]),
                        Node::dir("zoned", Dev, vec![f("node_modules", 30)]),
                        Node::dir("seul", Dev, vec![Node::dir("x", Dev, vec![f("target", 5)])]),
                    ],
                ),
                f(".cache", 100),
            ],
        );
        root.finish();
        root
    }

    fn items(root: &Node) -> Vec<Vec<usize>> {
        root.suggestions_paths(&["target", "node_modules", ".cache"])
    }

    fn shown(rows: &[Row]) -> Vec<(usize, &str, u64, Option<bool>)> {
        rows.iter().map(|r| (r.depth, r.label.as_str(), r.size, r.open)).collect()
    }

    impl Node {
        fn suggestions_paths(&self, names: &[&str]) -> Vec<Vec<usize>> {
            fn walk(n: &Node, path: &mut Vec<usize>, names: &[&str], out: &mut Vec<Vec<usize>>) {
                for (i, c) in n.children.iter().enumerate() {
                    path.push(i);
                    if names.contains(&c.name.as_str()) { out.push(path.clone()) } else { walk(c, path, names, out) }
                    path.pop();
                }
            }
            let mut out = Vec::new();
            walk(self, &mut Vec::new(), names, &mut out);
            out
        }
    }

    #[test]
    fn folders_are_closed_and_largest_first() {
        let root = home();
        let rows = outline(&root, &items(&root), &HashSet::new());
        assert_eq!(shown(&rows), [(0, ".cache", 100, None), (0, "projets", 85, Some(false))]);
        assert_eq!(rows[1].items.len(), 3, "a folder holds every item below it");
    }

    #[test]
    fn an_open_folder_shows_its_lines_and_single_ways_are_one_line() {
        let root = home();
        let projets = outline(&root, &items(&root), &HashSet::new())[1].path.clone();
        let rows = outline(&root, &items(&root), &HashSet::from([projets]));
        assert_eq!(
            shown(&rows),
            [(0, ".cache", 100, None), (0, "projets", 85, Some(true)), (1, "bref/target", 50, None), (1, "zoned/node_modules", 30, None), (1, "seul/x/target", 5, None)]
        );
    }

    #[test]
    fn a_folder_alone_at_the_top_is_open_until_closed() {
        let root = home();
        let items: Vec<_> = items(&root).into_iter().filter(|p| root.children[p[0]].name == "projets").collect();
        let rows = outline(&root, &items, &HashSet::new());
        assert_eq!(rows[0].open, Some(true));
        assert_eq!(rows.len(), 4);
        let rows = outline(&root, &items, &HashSet::from([rows[0].path.clone()]));
        assert_eq!(shown(&rows), [(0, "projets", 85, Some(false))]);
    }

    #[test]
    fn nothing_one_and_a_thousand() {
        let root = home();
        assert!(outline(&root, &[], &HashSet::new()).is_empty());
        let one = outline(&root, &[vec![0]], &HashSet::new());
        assert_eq!(shown(&one), [(0, ".cache", 100, None)]);
        let mut many = Node::dir("m", Dev, (0..1_000).map(|i| Node::dir(format!("p{i}"), Dev, vec![f("node_modules", 1 + i)])).collect());
        many.finish();
        let paths = many.suggestions_paths(&["node_modules"]);
        let rows = outline(&many, &paths, &HashSet::new());
        assert_eq!(rows.len(), 1_000, "a thousand folders of one item each are a thousand lines");
        assert_eq!(rows[0].label, "p999/node_modules");
    }
}
