use super::{BoundingVolume, UBTree};
use crate::ncollection::destructor_unwind::DeferredDestructorPanic;
impl<T: Clone + Default, B: BoundingVolume> UBTree<T, B> {
    /// Kill child `index` of branch `id` (the whole sub-branch), as native `TreeNode::Kill`:
    /// the branch takes over the other child's bounds, object and children. Ancestor bounds
    /// are not updated, as natively. A leaf, a dead id or an index other than 0/1 (undefined
    /// natively) leaves the tree unchanged. Example: `tree.kill(root, 1)`.
    pub fn kill(&mut self, id: usize, index: i32) {
        let Some(children) = self.node_by_id(id).and_then(|node| node.children) else {
            return;
        };
        let (killed, kept) = match index {
            0 => (children[0], children[1]),
            1 => (children[1], children[0]),
            _ => return,
        };
        self.assign_survivor(id, kept);
        self.reparent_children(id);
        let mut unwind = DeferredDestructorPanic::default();
        unwind.capture(|| self.release_node(kept));
        self.release_subtree(killed, &mut unwind);
        unwind.finish(Ok(()));
    }
    fn assign_survivor(&mut self, parent: usize, kept: usize) {
        let [target, source] = self.nodes.get_disjoint_mut([parent, kept]).unwrap();
        let target = target.as_deref_mut().unwrap();
        let source = source.as_deref().unwrap();
        // OCCT 3d097a0328 Kill commits links only after both member assignments succeed.
        target.bounds.clone_from(&source.bounds);
        target.object.clone_from(&source.object);
        target.children = source.children;
    }
    pub(in crate::ncollection) fn update_ancestors(&mut self, mut index: usize) {
        while let Some(parent) = self.node(index).parent {
            self.assign_child_union(parent);
            index = parent;
        }
    }
    fn assign_child_union(&mut self, parent: usize) {
        let [first, second] = self.node(parent).children.unwrap();
        let [target, left, right] = self
            .nodes
            .get_disjoint_mut([parent, first, second])
            .unwrap();
        let target = target.as_deref_mut().unwrap();
        let left = left.as_deref().unwrap();
        let right = right.as_deref().unwrap();
        // OCCT 3d097a0328 Remove assigns into the retained bound before expanding it.
        target.bounds.clone_from(&left.bounds);
        target.bounds.add(&right.bounds);
    }
}
