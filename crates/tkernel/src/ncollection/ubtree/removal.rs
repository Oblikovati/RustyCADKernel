use super::{BoundingNode, BoundingVolume, UBTree};
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
        let mut survivor: BoundingNode<T, B> = self.node(kept).clone();
        survivor.parent = self.node(id).parent;
        *self.node_mut(id) = survivor;
        self.nodes[kept] = None;
        self.release_subtree(killed);
        self.reparent_children(id);
    }
    /// Frees a detached sub-branch iteratively, like native `delNode`, so deep trees
    /// cannot overflow the call stack.
    fn release_subtree(&mut self, top: usize) {
        let mut pending: Vec<usize> = vec![top];
        while let Some(index) = pending.pop() {
            if let Some(node) = self.nodes[index].take() {
                pending.extend(node.children.into_iter().flatten());
            }
        }
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
