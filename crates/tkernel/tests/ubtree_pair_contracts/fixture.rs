pub use super::allocation_trace::*;
use rustycad_tkernel::ncollection::BaseAllocator;
/// Builds a leaf or an existing branch on an observed allocator; `populated(true)`.
pub fn populated(branch: bool) -> (Tree, BaseAllocator) {
    let allocator = BaseAllocator::accumulating(65536).unwrap();
    let mut tree = Tree::with_allocator(allocator.clone());
    tree.add(Payload(1), Payload(10));
    if branch {
        tree.gemmate_ref(0, &Payload(30), &Payload(2), &Payload(20));
    }
    (tree, allocator)
}
/// Finds an exact native pair row; `native_row("layout ")`.
pub fn native_row(prefix: &str) -> &'static str {
    include_str!("../../../../tools/oracle/tkernel_ubtree_pair_contracts.expected")
        .lines()
        .find(|line| line.starts_with(prefix))
        .unwrap()
}
/// Captures one deliberate callback rejection; `caught(|| operation())`.
pub fn caught(operation: impl FnOnce()) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).is_err()
}
/// Checks actual typed child spacing; `pair_is_contiguous(&tree)`.
pub fn pair_is_contiguous(tree: &Tree) -> bool {
    let first = tree.child(tree.root().unwrap(), 0).unwrap();
    let second = tree.child(tree.root().unwrap(), 1).unwrap();
    std::ptr::from_ref(second).addr() - std::ptr::from_ref(first).addr()
        == std::mem::size_of_val(first)
}
