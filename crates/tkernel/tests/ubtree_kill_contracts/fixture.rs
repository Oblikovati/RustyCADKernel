#[path = "../support/bounding_tree_trace.rs"]
mod trace;
use rustycad_tkernel::ncollection::{BaseAllocator, UBTree};
pub use trace::{Item, Trace, caught};
pub type Tree = UBTree<Item<'O'>, Item<'B'>>;
pub fn populated(trace: &Trace, allocator: BaseAllocator) -> Tree {
    trace.install_defaults();
    let mut tree = Tree::with_allocator(allocator);
    tree.add(trace.item(1), trace.item(1));
    tree.gemmate(0, trace.item(10), trace.item(2), trace.item(2));
    tree.gemmate(1, trace.item(30), trace.item(3), trace.item(3));
    tree.gemmate(2, trace.item(40), trace.item(4), trace.item(4));
    trace.clear();
    tree
}
