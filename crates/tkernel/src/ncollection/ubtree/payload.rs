//! Native TreeNode member construction, assignment and destruction phases over the arena facade.
use super::BoundingNode;
impl<T: Clone, B: Clone> BoundingNode<T, B> {
    /// Copies bounds before object, as native `TreeNode(theObj,theBnd)` does.
    /// Example: `let leaf = BoundingNode::new_ref(&object, &bounds);`.
    pub fn new_ref(object: &T, bounds: &B) -> Self {
        Self {
            bounds: bounds.clone(),
            object: object.clone(),
            children: None,
            parent: None,
        }
    }
}
impl<T: Clone, B: Clone> Clone for BoundingNode<T, B> {
    fn clone(&self) -> Self {
        let mut copied = Self::new_ref(&self.object, &self.bounds);
        copied.children = self.children;
        copied.parent = self.parent;
        copied
    }
    fn clone_from(&mut self, source: &Self) {
        // OCCT 3d097a0328 implicitly assigns fields before links; a throwing member leaves links.
        self.bounds.clone_from(&source.bounds);
        self.object.clone_from(&source.object);
        self.children = source.children;
        self.parent = source.parent;
    }
}
