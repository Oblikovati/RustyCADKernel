//! Native const map and node references remain live without retaining a mutation-blocking lock.
use super::group_membership::GroupMemberNode;
use super::{Select3DSensitiveEntity, Select3DSensitiveGroup};
use rustycad_tkernel::standard::{Failure, Handle};
use std::sync::Arc;

/// A borrowed live map view; `let members = group.entities_ref(); members.len();`.
#[derive(Clone, Copy)]
pub struct Select3DGroupEntitiesRef<'a> {
    group: &'a Select3DSensitiveGroup,
}
/// Native equivalence requires that this referenced map node survives; deleted-node aliases are undefined upstream.
/// A borrowed stable node reference; `let member = group.sub_entity_ref(1)?; member.entity();`.
#[derive(Clone)]
pub struct Select3DGroupSubEntityRef<'a> {
    group: &'a Select3DSensitiveGroup,
    node: Arc<GroupMemberNode>,
}
impl Select3DSensitiveGroup {
    /// Retains the native map reference across additions/removals; `group.entities_ref().len()`.
    pub fn entities_ref(&self) -> Select3DGroupEntitiesRef<'_> {
        // OCCT 3d097a0 returns the map member itself, not a copied membership snapshot.
        Select3DGroupEntitiesRef { group: self }
    }
    /// Retains the allocated node at a checked one-based index; `group.sub_entity_ref(1)?.entity()`.
    pub fn sub_entity_ref(&self, index: i32) -> Result<Select3DGroupSubEntityRef<'_>, Failure> {
        // Removing another key changes indices but never changes this surviving node's key.
        let node = self.state.read().node(index)?;
        Ok(Select3DGroupSubEntityRef { group: self, node })
    }
}
impl<'a> Select3DGroupEntitiesRef<'a> {
    /// Observes the current membership count; `group.entities_ref().len()`.
    pub fn len(&self) -> usize {
        // No retained lock prevents native-style mutation through the borrowed group.
        self.group.state.read().nodes.len()
    }
    /// Checks current emptiness; `group.entities_ref().is_empty()`.
    pub fn is_empty(&self) -> bool {
        // This observation reflects later Clear operations on the same container.
        self.len() == 0
    }
    /// Retains a node from the current membership; `group.entities_ref().sub_entity(1)?.entity()`.
    pub fn sub_entity(&self, index: i32) -> Result<Select3DGroupSubEntityRef<'a>, Failure> {
        // The returned view binds the stable node instead of repeatedly resolving its old index.
        self.group.sub_entity_ref(index)
    }
}
impl Select3DGroupSubEntityRef<'_> {
    /// Observes the referenced key handle; `group.sub_entity_ref(1)?.entity()`.
    pub fn entity(&self) -> Handle<dyn Select3DSensitiveEntity> {
        // The group borrow bounds validity; deleted native node references remain outside proof.
        let _borrowed_group = self.group;
        self.node.sensitive.clone()
    }
}
