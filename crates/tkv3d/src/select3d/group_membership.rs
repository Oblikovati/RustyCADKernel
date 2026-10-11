//! Stable allocated membership nodes preserve native IndexedMap reference identity.
use super::{Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveGroup};
use rustycad_tkernel::standard::exceptions::out_of_range;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Point3;
use std::sync::Arc;

pub(super) struct GroupMemberNode {
    pub(super) sensitive: Handle<dyn Select3DSensitiveEntity>,
}
#[derive(Clone)]
pub(super) struct GroupMembershipState {
    pub(super) nodes: Vec<Arc<GroupMemberNode>>,
    pub(super) must_match_all: bool,
    pub(super) to_check_overlap_all: bool,
    pub(super) center: Point3,
    pub(super) bnd_box: Select3DBndBox3d,
    pub(super) bvh_prim_indexes: Vec<i32>,
}
impl GroupMembershipState {
    pub(super) fn new(must_match_all: bool) -> Self {
        Self {
            nodes: Vec::new(),
            must_match_all,
            to_check_overlap_all: false,
            center: Point3::default(),
            bnd_box: Select3DBndBox3d::default(),
            bvh_prim_indexes: Vec::new(),
        }
    }
    /// `myEntities.Add`: appends a new entity; false when it is already in the group.
    pub(super) fn add_entity(&mut self, sensitive: &Handle<dyn Select3DSensitiveEntity>) -> bool {
        if self.nodes.iter().any(|node| node.sensitive == *sensitive) {
            return false;
        }
        self.nodes.push(Arc::new(GroupMemberNode {
            sensitive: sensitive.clone(),
        }));
        true
    }
    pub(super) fn node(&self, index: i32) -> Result<Arc<GroupMemberNode>, Failure> {
        let offset = usize::try_from(index)
            .ok()
            .and_then(|value| value.checked_sub(1));
        offset
            .and_then(|value| self.nodes.get(value))
            .cloned()
            .ok_or_else(|| {
                out_of_range(&format!(
                    "Select3D_SensitiveGroup::SubEntity index {index}; expected 1..={}",
                    self.nodes.len()
                ))
            })
    }
}
impl Select3DSensitiveGroup {
    /// Copies member handles for convenient iteration; `group.entities()`.
    pub fn entities(&self) -> Vec<Handle<dyn Select3DSensitiveEntity>> {
        // The native const container reference is provided separately by entities_ref().
        self.state
            .read()
            .nodes
            .iter()
            .map(|node| node.sensitive.clone())
            .collect()
    }
    /// Copies a member handle, returning null out of range; `group.sub_entity(1)`.
    pub fn sub_entity(&self, index: i32) -> Handle<dyn Select3DSensitiveEntity> {
        // This compatibility convenience is unclaimed; sub_entity_ref() retains checked semantics.
        self.state
            .read()
            .node(index)
            .map(|node| node.sensitive.clone())
            .unwrap_or_default()
    }
    /// Tests identity membership; `group.is_in(&entity)`.
    pub fn is_in(&self, sensitive: &Handle<dyn Select3DSensitiveEntity>) -> bool {
        // Equal geometry in distinct handles remains distinct native map keys.
        self.state
            .read()
            .nodes
            .iter()
            .any(|node| node.sensitive == *sensitive)
    }
}
