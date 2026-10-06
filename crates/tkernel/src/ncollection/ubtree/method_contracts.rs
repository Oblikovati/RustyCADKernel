//! Protected native tree operations exposed through checked arena identities.
use super::{BoundingNode, BoundingVolume, TreeSelector, UBTree};
use crate::standard::Failure;

impl<T: Clone + Default, B: BoundingVolume> UBTree<T, B> {
    /// Resolves the last inserted live node, native ChangeLastNode; `tree.last_node()`.
    pub fn last_node(&self) -> Option<&BoundingNode<T, B>> {
        self.node_by_id(self.last?)
    }

    /// Mutably resolves the last inserted live node; `tree.change_last_node()?.bounds_mut()`.
    pub fn change_last_node(&mut self) -> Option<&mut BoundingNode<T, B>> {
        self.change_node_by_id(self.last?)
    }

    /// Traverses one branch using native Select ordering; `tree.select_branch(root_id, &mut selector)?`.
    pub fn select_branch<S: TreeSelector<T, B> + ?Sized>(
        &self,
        id: usize,
        selector: &mut S,
    ) -> Result<i32, Failure> {
        if self.node_by_id(id).is_none() {
            return Err(Failure::with_type(
                "Standard_NoSuchObject",
                &format!(
                    "UBTree node {id}; expected a live node id below {}",
                    self.nodes.len()
                ),
                "",
            ));
        }
        Ok(self.select_nodes(Some(id), selector))
    }

    /// Preserves native move self-assignment without aliased references; `tree.move_assign_self()`.
    pub fn move_assign_self(&mut self) -> &mut Self {
        self
    }
}
