//! Composite mutations preserve native partial writes and deliberately do not invalidate BVH.
//! OCCT 3d097a0 leaves Add/Remove/Clear trees clean until explicit MarkDirty repairs them.
use super::triangulation_failure::native_checked;
use super::{Select3DSensitiveEntity, Select3DSensitiveGroup};
use crate::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Point3, Xyz};
impl Select3DSensitiveGroup {
    /// A group of distinct entities centered on their mean; `Select3DSensitiveGroup::with_entities(&owner, &entities, true)`.
    pub fn with_entities(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        entities: &[Handle<dyn Select3DSensitiveEntity>],
        must_match_all: bool,
    ) -> Self {
        let group = Self::new(owner, must_match_all);
        for sensitive in entities {
            group.add_initial_member(sensitive);
        }
        group.divide_group_center();
        group.set.mark_dirty();
        group
    }
    fn add_initial_member(&self, sensitive: &Handle<dyn Select3DSensitiveEntity>) {
        if !self.state.write().add_entity(sensitive) {
            return;
        }
        self.combine_member_bounds(sensitive);
        self.append_member_index();
        let center = sensitive.get().unwrap().center_of_geometry();
        self.accumulate_group_center(center);
    }
    /// `Add(sequence)`: the new entities, the center moved halfway to their mean.
    /// Example: `group.add_all(&entities)`.
    pub fn add_all(&self, entities: &[Handle<dyn Select3DSensitiveEntity>]) {
        if entities.is_empty() {
            return;
        }
        let mut center = Xyz::default();
        for sensitive in entities {
            center += self.add_list_member(sensitive);
        }
        let mut state = self.state.write();
        center /= state.nodes.len() as f64;
        state.center = Point3::from((state.center.xyz() + center) * 0.5);
    }
    fn add_list_member(&self, sensitive: &Handle<dyn Select3DSensitiveEntity>) -> Xyz {
        if !self.state.write().add_entity(sensitive) {
            return Xyz::default();
        }
        self.combine_member_bounds(sensitive);
        self.append_member_index();
        sensitive.get().unwrap().center_of_geometry().xyz()
    }
    /// `Add(entity)`: a new entity; the center is the running half-sum (natively not a mean).
    /// Example: `group.add(&entity)`.
    pub fn add(&self, sensitive: &Handle<dyn Select3DSensitiveEntity>) {
        if !self.state.write().add_entity(sensitive) {
            return;
        }
        self.append_member_index();
        self.combine_member_bounds(sensitive);
        let center = sensitive.get().unwrap().center_of_geometry();
        let mut state = self.state.write();
        state.center = Point3::from(state.center.xyz() + center.xyz());
        if state.nodes.len() >= 2 {
            state.center = Point3::from(state.center.xyz() * 0.5);
        }
    }
    /// `Remove`: the entity (the last one takes its index, as `NCollection_IndexedMap`), then
    /// the box, the mean center and the indexes are rebuilt. Example: `group.remove(&entity)`.
    pub fn remove(&self, sensitive: &Handle<dyn Select3DSensitiveEntity>) {
        if !self.erase_member(sensitive) {
            return;
        }
        self.reset_group_geometry();
        for (offset, sensitive) in self.entities().iter().enumerate() {
            self.rebuild_member_geometry(sensitive, offset as i32 + 1);
        }
        self.divide_group_center();
    }
    fn erase_member(&self, sensitive: &Handle<dyn Select3DSensitiveEntity>) -> bool {
        let mut state = self.state.write();
        let Some(index) = state
            .nodes
            .iter()
            .position(|node| node.sensitive == *sensitive)
        else {
            return false;
        };
        state.nodes.swap_remove(index);
        true
    }
    pub(super) fn reset_group_geometry(&self) {
        let mut state = self.state.write();
        state.bnd_box.clear();
        state.center = Point3::default();
        state.bvh_prim_indexes.clear();
    }
    fn rebuild_member_geometry(&self, sensitive: &Handle<dyn Select3DSensitiveEntity>, index: i32) {
        self.combine_member_bounds(sensitive);
        let center = sensitive.get().unwrap().center_of_geometry();
        let mut state = self.state.write();
        state.center = Point3::from(state.center.xyz() + center.xyz());
        state.bvh_prim_indexes.push(index);
    }
    fn accumulate_group_center(&self, center: Point3) {
        let mut state = self.state.write();
        // Preserve each native center write before a later child callback can fail.
        state.center = Point3::from(state.center.xyz() + center.xyz());
    }
    fn divide_group_center(&self) {
        let mut state = self.state.write();
        // Native floating-point division by zero preserves NaNs for empty sequence/removal.
        state.center = Point3::from(state.center.xyz() / state.nodes.len() as f64);
    }
    fn append_member_index(&self) {
        let mut state = self.state.write();
        let extent = state.nodes.len() as i32;
        state.bvh_prim_indexes.push(extent);
    }
    pub(super) fn combine_member_bounds(&self, sensitive: &Handle<dyn Select3DSensitiveEntity>) {
        // The callback precedes each cache write; earlier combined bounds survive a later failure.
        let bounds = sensitive.get().unwrap().bounding_box();
        self.state.write().bnd_box.combine(&bounds);
    }
    pub(super) fn primitive_member(&self, index: i32) -> Handle<dyn Select3DSensitiveEntity> {
        let member = self.state.read().bvh_prim_indexes[index as usize];
        // DynamicArray indices are unchecked natively; only valid primitive indices are proved.
        native_checked(self.sub_entity_ref(member)).entity()
    }
}
