//! `Select3D_SensitiveGroup`: distinct sensitive entities matched together (all of them for a
//! box or polyline when asked) or through a BVH tree.
//! Martensite preserves native membership, callback order and retained nodes.
use super::group_membership::GroupMembershipState;
use super::{Select3DSensitiveEntity, Select3DSensitiveEntityBase, Select3DSensitiveSetBase};
use crate::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};

/// A group of sensitive entities sharing a composite owner.
/// Example: `let group = Select3DSensitiveGroup::new(&owner, true); group.add(&entity);`.
pub struct Select3DSensitiveGroup {
    pub(super) transient: StandardTransient,
    pub(super) entity: Select3DSensitiveEntityBase,
    pub(super) set: Select3DSensitiveSetBase,
    pub(super) state: SharedState<GroupMembershipState>,
}
rustycad_tkernel::impl_transient!(Select3DSensitiveGroup, "Select3D_SensitiveGroup", dyn super::Select3DSensitiveSet, transient,
    interfaces: [dyn Select3DSensitiveEntity, dyn super::Select3DSensitiveSet, dyn super::Select3DSensitiveGroupSource]);
impl Select3DSensitiveGroup {
    /// Constructs an empty group; `Select3DSensitiveGroup::new(&owner, true)`.
    pub fn new(owner: &Handle<dyn SelectMgrEntityOwnerSource>, must_match_all: bool) -> Self {
        Self {
            transient: StandardTransient::default(),
            entity: Select3DSensitiveEntityBase::new(owner),
            set: Select3DSensitiveSetBase::default(),
            state: SharedState::new(GroupMembershipState::new(must_match_all)),
        }
    }
}

impl Select3DSensitiveGroup {
    /// Applies the native default match-all argument; `Select3DSensitiveGroup::with_default_match_type(&owner)`.
    pub fn with_default_match_type(owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> Self {
        // OCCT 3d097a0 declares both constructor bool arguments with default true.
        Self::new(owner, true)
    }
    /// Constructs native default match-all membership; `Select3DSensitiveGroup::with_entities_default_match_type(&owner,&members)`.
    pub fn with_entities_default_match_type(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        members: &[Handle<dyn Select3DSensitiveEntity>],
    ) -> Self {
        // Keep the native sequence constructor's mean, dirty state and empty NaNs unchanged.
        Self::with_entities(owner, members, true)
    }
}
