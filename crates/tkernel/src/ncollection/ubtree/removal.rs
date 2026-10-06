use super::{BoundingVolume, UBTree};
use crate::ncollection::{destructor_unwind::DeferredDestructorPanic, node_owner::NativeNodeOwner};
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
    /// Destroy detached payloads in native preorder without recursive stack growth.
    fn release_subtree(&mut self, top: usize, unwind: &mut DeferredDestructorPanic) {
        let mut pending: Vec<usize> = vec![top];
        while let Some(index) = pending.pop() {
            if let Some([first, second]) = self.node(index).children {
                pending.extend([second, first]);
            }
            unwind.capture(|| self.release_node(index));
        }
    }
    fn release_node(&mut self, index: usize) {
        let owner = self.nodes[index].take().unwrap();
        // The shared owner deallocates even if a payload destructor unwinds.
        drop(NativeNodeOwner::from_allocated(owner));
    }
    pub(in crate::ncollection) fn remove_leaf(&mut self, index: usize) -> Option<usize> {
        let Some(parent) = self.node(index).parent else {
            self.clear();
            return None;
        };
        let first: usize = self.node(parent).children.unwrap()[0];
        self.kill(parent, if first == index { 0 } else { 1 });
        self.update_ancestors(parent);
        self.node(parent).is_leaf().then_some(parent)
    }
    fn update_ancestors(&mut self, mut index: usize) {
        while let Some(parent) = self.node(index).parent {
            let [first, second] = self.node(parent).children.unwrap();
            let mut bounds: B = self.node(first).bounds.clone();
            bounds.add(&self.node(second).bounds);
            self.node_mut(parent).bounds = bounds;
            index = parent;
        }
    }
}
