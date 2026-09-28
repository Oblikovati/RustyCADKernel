//! NCollection trees probe operations (twin of `tkernel_ncollection_trees_probe.cxx`).
use rustycad_tkernel::ncollection::{
    BoundingNode, BoundingVolume, EBTree, TreeSelector, UBTree, UBTreeFiller,
};
use std::str::SplitWhitespace;

#[path = "trees_cell.rs"]
mod trees_cell;

/// The output of a trees operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    match op {
        "ncoll_trees_ubtree" => Some(ubtree(input)),
        "ncoll_trees_kill" => Some(kill(input)),
        "ncoll_trees_gemmate" => Some(gemmate(input)),
        "ncoll_trees_ebtree" => Some(ebtree(input)),
        "ncoll_trees_filler" => Some(filler(input)),
        _ => trees_cell::evaluate(op, input),
    }
}

/// 2D box with the minimal UBTree bound interface, mirroring the native probe's `Box`.
#[derive(Clone, Default)]
struct ProbeBox {
    min: [f64; 2],
    max: [f64; 2],
}
impl BoundingVolume for ProbeBox {
    fn add(&mut self, other: &Self) {
        for axis in 0..2 {
            self.min[axis] = if other.min[axis] < self.min[axis] {
                other.min[axis]
            } else {
                self.min[axis]
            };
            self.max[axis] = if other.max[axis] > self.max[axis] {
                other.max[axis]
            } else {
                self.max[axis]
            };
        }
    }
    fn is_out(&self, other: &Self) -> bool {
        other.min[0] > self.max[0]
            || other.max[0] < self.min[0]
            || other.min[1] > self.max[1]
            || other.max[1] < self.min[1]
    }
    fn square_extent(&self) -> f64 {
        let dx: f64 = self.max[0] - self.min[0];
        let dy: f64 = self.max[1] - self.min[1];
        dx * dx + dy * dy
    }
}

pub(super) fn number<T: std::str::FromStr>(input: &mut SplitWhitespace<'_>) -> T {
    match input.next().expect("probe field").parse() {
        Ok(value) => value,
        Err(_) => panic!("malformed probe field"),
    }
}
fn read_box(input: &mut SplitWhitespace<'_>) -> ProbeBox {
    let values: [f64; 4] = std::array::from_fn(|_| number(input));
    ProbeBox {
        min: [values[0], values[1]],
        max: [values[2], values[3]],
    }
}
fn read_items(input: &mut SplitWhitespace<'_>) -> Vec<(i32, ProbeBox)> {
    let count: usize = number(input);
    (0..count)
        .map(|_| {
            let object: i32 = number(input);
            (object, read_box(input))
        })
        .collect()
}
fn flag(value: bool) -> char {
    if value { '1' } else { '0' }
}

type Tree = UBTree<i32, ProbeBox>;

fn shape(out: &mut String, tree: &Tree, node: &BoundingNode<i32, ProbeBox>) {
    let bounds: &ProbeBox = node.bounds();
    out.push_str(&format!(
        " {}{}[{},{},{},{}]",
        if node.is_leaf() { 'L' } else { 'B' },
        node.object(),
        bounds.min[0].to_bits(),
        bounds.min[1].to_bits(),
        bounds.max[0].to_bits(),
        bounds.max[1].to_bits()
    ));
    for index in 0..2 {
        if let Some(child) = tree.child(node, index) {
            shape(out, tree, child);
        }
    }
}
fn print_tree(out: &mut String, tree: &Tree) {
    match tree.root() {
        Some(root) => shape(out, tree, root),
        None => out.push_str(" empty"),
    }
}

struct ProbeSelector {
    query: ProbeBox,
    limit: i32,
    accepted: Vec<i32>,
    stopped: bool,
}
impl TreeSelector<i32, ProbeBox> for ProbeSelector {
    fn reject(&self, bounds: &ProbeBox) -> bool {
        self.query.is_out(bounds)
    }
    fn accept(&mut self, object: &i32) -> bool {
        self.accepted.push(*object);
        self.stopped = self.limit > 0 && self.accepted.len() as i32 >= self.limit;
        object % 3 != 0
    }
    fn stop(&self) -> bool {
        self.stopped
    }
}
fn select(
    out: &mut String,
    input: &mut SplitWhitespace<'_>,
    run: impl FnOnce(&mut ProbeSelector) -> i32,
) {
    let mut selector: ProbeSelector = ProbeSelector {
        query: read_box(input),
        limit: number(input),
        accepted: Vec::new(),
        stopped: false,
    };
    let count: i32 = run(&mut selector);
    out.push_str(&format!(" select {count}"));
    for object in selector.accepted {
        out.push_str(&format!(" {object}"));
    }
}

fn ubtree(input: &mut SplitWhitespace<'_>) -> String {
    let mut tree: Tree = UBTree::default();
    let mut out: String = String::new();
    for (object, bounds) in read_items(input) {
        out.push(flag(tree.add(object, bounds)));
    }
    print_tree(&mut out, &tree);
    select(&mut out, input, |selector| tree.select(selector));
    out
}
fn built(input: &mut SplitWhitespace<'_>) -> Tree {
    let mut tree: Tree = UBTree::default();
    for (object, bounds) in read_items(input) {
        tree.add(object, bounds);
    }
    tree
}
fn walk(input: &mut SplitWhitespace<'_>, tree: &Tree) -> usize {
    let depth: usize = number(input);
    let mut id: usize = tree.root_id().unwrap();
    for _ in 0..depth {
        let child: i32 = number(input);
        // A leaf has no children to descend into; the walk stops there.
        id = tree.node_by_id(id).unwrap().child_id(child).unwrap_or(id);
    }
    id
}
fn kill(input: &mut SplitWhitespace<'_>) -> String {
    let mut tree: Tree = built(input);
    let id: usize = walk(input, &tree);
    tree.kill(id, number(input));
    let mut out: String = String::new();
    print_tree(&mut out, &tree);
    out
}
fn gemmate(input: &mut SplitWhitespace<'_>) -> String {
    let mut tree: Tree = built(input);
    let id: usize = walk(input, &tree);
    let new_bounds: ProbeBox = read_box(input);
    let object: i32 = number(input);
    let leaf: usize = tree
        .gemmate(id, new_bounds, object, read_box(input))
        .unwrap();
    let linked: bool = tree.node_by_id(leaf).unwrap().parent_id() == Some(id);
    let mut out: String = String::from(if linked { " linked" } else { " broken" });
    print_tree(&mut out, &tree);
    out
}
fn ebtree(input: &mut SplitWhitespace<'_>) -> String {
    let mut tree: EBTree<i32, ProbeBox> = EBTree::default();
    let mut out: String = String::new();
    for (object, bounds) in read_items(input) {
        out.push(flag(tree.add(object, bounds)));
    }
    let removals: usize = number(input);
    out.push_str(" remove");
    let removed: Vec<i32> = (0..removals).map(|_| number(input)).collect();
    for object in &removed {
        out.push(flag(tree.remove(object)));
    }
    print_tree(&mut out, tree.as_ubtree());
    for object in &removed {
        out.push_str(if tree.contains(object) { " +" } else { " -" });
    }
    select(&mut out, input, |selector| tree.select(selector));
    out
}
fn filler(input: &mut SplitWhitespace<'_>) -> String {
    let full: i32 = number(input);
    let mut tree: Tree = UBTree::default();
    let mut out: String = String::new();
    {
        let mut filler: UBTreeFiller<'_, i32, ProbeBox> =
            UBTreeFiller::new(&mut tree, None, full != 0);
        for (object, bounds) in read_items(input) {
            filler.add(object, bounds);
        }
        out.push_str(&filler.fill().to_string());
        let mut check: String = String::new();
        let leaves: i32 = filler.check_tree(&mut check).unwrap();
        out.push_str(&format!(" {leaves} {}", check.replace('\n', "|")));
        // Items queued after fill are added when the filler is dropped.
        filler.add(-1, ProbeBox::default());
    }
    print_tree(&mut out, &tree);
    out
}
