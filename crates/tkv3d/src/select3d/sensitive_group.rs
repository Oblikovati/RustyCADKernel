//! `Select3D_SensitiveGroup`: distinct sensitive entities matched together (all of them for a
//! box or polyline when asked) or through a BVH tree.
use super::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityMembers as _, Select3DSensitiveSet, Select3DSensitiveSetBase,
    select3d_set_build_bvh, select3d_set_dump_json, select3d_set_matches,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelectionType};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::gp::{Point3, Xyz};
use rustycad_tkservice::json_dump::{base_class, class_begin, field_dumped, field_integer};
use std::io;

/// The mutable state of a group.
#[derive(Clone)]
struct GroupState {
    entities: Vec<Handle<dyn Select3DSensitiveEntity>>,
    must_match_all: bool,
    to_check_overlap_all: bool,
    center: Point3,
    bnd_box: Select3DBndBox3d,
    bvh_prim_indexes: Vec<i32>,
}

impl GroupState {
    /// `myEntities.Add`: appends a new entity; false when it is already in the group.
    fn add_entity(&mut self, entity: &Handle<dyn Select3DSensitiveEntity>) -> bool {
        if self.entities.iter().any(|e| e == entity) {
            return false;
        }
        self.entities.push(entity.clone());
        true
    }
}

/// A group of sensitive entities.
///
/// Example: `let group = Select3DSensitiveGroup::new(&owner, true); group.add(&entity);`.
pub struct Select3DSensitiveGroup {
    transient: StandardTransient,
    entity: Select3DSensitiveEntityBase,
    set: Select3DSensitiveSetBase,
    state: SharedState<GroupState>,
}
rustycad_tkernel::impl_transient!(Select3DSensitiveGroup, "Select3D_SensitiveGroup", dyn Select3DSensitiveSet, transient,
    interfaces: [dyn Select3DSensitiveEntity, dyn Select3DSensitiveSet]);

impl Select3DSensitiveGroup {
    /// An empty group; `Select3DSensitiveGroup::new(&owner, true)`.
    pub fn new(owner: &Handle<dyn SelectMgrEntityOwnerSource>, is_must_match_all: bool) -> Self {
        let state = GroupState {
            entities: Vec::new(),
            must_match_all: is_must_match_all,
            to_check_overlap_all: false,
            center: Point3::new(0.0, 0.0, 0.0),
            bnd_box: Select3DBndBox3d::default(),
            bvh_prim_indexes: Vec::new(),
        };
        Self {
            transient: StandardTransient::default(),
            entity: Select3DSensitiveEntityBase::new(owner),
            set: Select3DSensitiveSetBase::default(),
            state: SharedState::new(state),
        }
    }
    /// A group of the distinct entities, centered on their mean; `Select3DSensitiveGroup::with_entities(&owner, &entities, true)`.
    pub fn with_entities(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        entities: &[Handle<dyn Select3DSensitiveEntity>],
        is_must_match_all: bool,
    ) -> Self {
        let group = Self::new(owner, is_must_match_all);
        {
            let mut state = group.state.write();
            let mut center = Xyz::new(0.0, 0.0, 0.0);
            for entity in entities {
                if !state.add_entity(entity) {
                    continue;
                }
                let Some(value) = entity.get() else { continue };
                state.bnd_box.combine(&value.bounding_box());
                let extent = state.entities.len() as i32;
                state.bvh_prim_indexes.push(extent);
                center += value.center_of_geometry().xyz();
            }
            state.center = Point3::from(center / state.entities.len() as f64);
        }
        group.set.mark_dirty();
        group
    }
    /// `Entities`; `group.entities()`.
    pub fn entities(&self) -> Vec<Handle<dyn Select3DSensitiveEntity>> {
        self.state.read().entities.clone()
    }
    /// `SubEntity`: the entity at a one-based index, null out of range; `group.sub_entity(1)`.
    pub fn sub_entity(&self, index: i32) -> Handle<dyn Select3DSensitiveEntity> {
        let state = self.state.read();
        usize::try_from(index - 1)
            .ok()
            .and_then(|i| state.entities.get(i).cloned())
            .unwrap_or_default()
    }
    /// `LastDetectedEntity`; `group.last_detected_entity()`.
    pub fn last_detected_entity(&self) -> Handle<dyn Select3DSensitiveEntity> {
        let index = self.last_detected_entity_index();
        if index == -1 {
            Handle::default()
        } else {
            self.sub_entity(index)
        }
    }
    /// `LastDetectedEntityIndex`: the one-based index of the detected entity, -1 without.
    /// Example: `group.last_detected_entity_index()`.
    pub fn last_detected_entity_index(&self) -> i32 {
        let detected = self.set.detected_idx();
        if detected == -1 {
            return -1;
        }
        self.state
            .read()
            .bvh_prim_indexes
            .get(detected as usize)
            .copied()
            .unwrap_or(-1)
    }
    /// `Add(sequence)`: the new entities, the center moved halfway to their mean.
    /// Example: `group.add_all(&entities)`.
    pub fn add_all(&self, entities: &[Handle<dyn Select3DSensitiveEntity>]) {
        if entities.is_empty() {
            return;
        }
        let mut state = self.state.write();
        let mut center = Xyz::new(0.0, 0.0, 0.0);
        for entity in entities {
            if !state.add_entity(entity) {
                continue;
            }
            let Some(value) = entity.get() else { continue };
            state.bnd_box.combine(&value.bounding_box());
            let extent = state.entities.len() as i32;
            state.bvh_prim_indexes.push(extent);
            center += value.center_of_geometry().xyz();
        }
        center /= state.entities.len() as f64;
        state.center = Point3::from((state.center.xyz() + center) * 0.5);
    }
    /// `Add(entity)`: a new entity; the center is the running half-sum (natively not a mean).
    /// Example: `group.add(&entity)`.
    pub fn add(&self, sensitive: &Handle<dyn Select3DSensitiveEntity>) {
        let mut state = self.state.write();
        if !state.add_entity(sensitive) {
            return;
        }
        let extent = state.entities.len() as i32;
        state.bvh_prim_indexes.push(extent);
        let Some(value) = sensitive.get() else { return };
        state.bnd_box.combine(&value.bounding_box());
        let mut center = state.center.xyz() + value.center_of_geometry().xyz();
        if extent >= 2 {
            center *= 0.5;
        }
        state.center = Point3::from(center);
    }
    /// `Remove`: the entity (the last one takes its index, as `NCollection_IndexedMap`), then
    /// the box, the mean center and the indexes are rebuilt. Example: `group.remove(&entity)`.
    pub fn remove(&self, sensitive: &Handle<dyn Select3DSensitiveEntity>) {
        let mut state = self.state.write();
        let Some(index) = state.entities.iter().position(|e| e == sensitive) else {
            return;
        };
        state.entities.swap_remove(index);
        state.bnd_box.clear();
        state.bvh_prim_indexes.clear();
        let mut center = Xyz::new(0.0, 0.0, 0.0);
        let entities = state.entities.clone();
        for (offset, entity) in entities.iter().enumerate() {
            if let Some(value) = entity.get() {
                state.bnd_box.combine(&value.bounding_box());
                center += value.center_of_geometry().xyz();
            }
            state.bvh_prim_indexes.push(offset as i32 + 1);
        }
        state.center = Point3::from(center / state.entities.len() as f64);
    }
    /// `IsIn`; `group.is_in(&entity)`.
    pub fn is_in(&self, sensitive: &Handle<dyn Select3DSensitiveEntity>) -> bool {
        self.state.read().entities.iter().any(|e| e == sensitive)
    }
    /// `SetMatchType`; `group.set_match_type(false)`.
    pub fn set_match_type(&self, is_must_match_all: bool) {
        self.state.write().must_match_all = is_must_match_all;
    }
    /// `MustMatchAll`; `group.must_match_all()`.
    pub fn must_match_all(&self) -> bool {
        self.state.read().must_match_all
    }
    /// `ToCheckOverlapAll`; `group.to_check_overlap_all()`.
    pub fn to_check_overlap_all(&self) -> bool {
        self.state.read().to_check_overlap_all
    }
    /// `SetCheckOverlapAll`; `group.set_check_overlap_all(true)`.
    pub fn set_check_overlap_all(&self, to_check_all: bool) {
        self.state.write().to_check_overlap_all = to_check_all;
    }
    /// The entity at a BVH index.
    fn element(&self, idx: i32) -> Handle<dyn Select3DSensitiveEntity> {
        let index = self
            .state
            .read()
            .bvh_prim_indexes
            .get(idx as usize)
            .copied()
            .unwrap_or(0);
        self.sub_entity(index)
    }
    /// The match of every entity (`Matches` for a box or polyline with `MustMatchAll` or
    /// `ToCheckOverlapAll`), keeping the nearest pick.
    fn matches_all(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
        to_match_all: bool,
        to_check_all: bool,
    ) -> Result<bool, Failure> {
        let mut pick = SelectBasicsPickResult::default();
        let mut is_failed = false;
        for child in self.entities().iter().filter_map(Handle::get) {
            if !child.matches(manager, &mut pick)? {
                if to_match_all {
                    is_failed = true;
                    if !to_check_all {
                        break;
                    }
                }
            } else {
                *pick_result = *SelectBasicsPickResult::min(pick_result, &pick);
            }
        }
        if is_failed {
            return Ok(false);
        }
        pick_result.set_dist_to_geom_center(self.distance_to_cog(manager));
        Ok(true)
    }
}

impl Select3DSensitiveEntity for Select3DSensitiveGroup {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        &self.entity
    }
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        let is_volume = manager.get_active_selection_type() != SelectMgrSelectionType::Point;
        let (must_all, check_all) = {
            let state = self.state.read();
            (state.must_match_all, state.to_check_overlap_all)
        };
        let (to_match_all, to_check_all) = (is_volume && must_all, is_volume && check_all);
        if !to_match_all && !to_check_all {
            return select3d_set_matches(self, manager, pick_result, false);
        }
        self.matches_all(manager, pick_result, to_match_all, to_check_all)
    }
    fn nb_sub_elements(&self) -> i32 {
        self.state.read().entities.len() as i32
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        let (mut bounds, entities) = {
            let state = self.state.read();
            if state.bnd_box.is_valid() {
                return state.bnd_box;
            }
            (state.bnd_box, state.entities.clone())
        };
        for entity in entities.iter().filter_map(Handle::get) {
            bounds.combine(&entity.bounding_box());
        }
        self.state.write().bnd_box = bounds;
        bounds
    }
    fn center_of_geometry(&self) -> Point3 {
        self.state.read().center
    }
    fn set(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) {
        self.entity.fields.write().owner_id = owner.clone();
        for entity in self.entities().iter().filter_map(Handle::get) {
            entity.set(owner);
        }
    }
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        let group = Self::new(&self.owner_id(), self.must_match_all());
        let mut connected = Vec::new();
        for entity in self.entities().iter().filter_map(Handle::get) {
            connected.push(entity.get_connected()?);
        }
        group.add_all(&connected);
        Ok(Handle::new(group).cast())
    }
    fn bvh(&self) -> Result<(), Failure> {
        select3d_set_build_bvh(self)
    }
    fn to_build_bvh(&self) -> bool {
        self.set.is_dirty()
    }
    /// `Clear`: forgets every entity (the owner is kept).
    fn clear(&self) {
        let mut state = self.state.write();
        state.entities.clear();
        state.bnd_box.clear();
        state.center = Point3::new(0.0, 0.0, 0.0);
        state.bvh_prim_indexes.clear();
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Select3D_SensitiveGroup")?;
        base_class(stream, depth, &|s, d| select3d_set_dump_json(self, s, d))?;
        let state = self.state.read().clone();
        for entity in state.entities.iter().filter_map(Handle::get) {
            field_dumped(stream, depth, "anEntity.get()", &|s, d| {
                entity.dump_json(s, d)
            })?;
        }
        field_integer(stream, "myMustMatchAll", i32::from(state.must_match_all))?;
        field_integer(
            stream,
            "myToCheckOverlapAll",
            i32::from(state.to_check_overlap_all),
        )?;
        let bounds = state.bnd_box;
        field_dumped(stream, depth, "&myBndBox", &|s, _| bounds.dump_json(s))
    }
}

impl Select3DSensitiveSet for Select3DSensitiveGroup {
    fn set_base(&self) -> &Select3DSensitiveSetBase {
        &self.set
    }
    fn size(&self) -> i32 {
        self.state.read().bvh_prim_indexes.len() as i32
    }
    fn element_box(&self, idx: i32) -> Select3DBndBox3d {
        self.element(idx)
            .get()
            .map(|e| e.bounding_box())
            .unwrap_or_default()
    }
    fn center(&self, idx: i32, axis: i32) -> Result<f64, Failure> {
        let center = self
            .element(idx)
            .get()
            .map(|e| e.center_of_geometry())
            .unwrap_or_default();
        Ok(match axis {
            0 => center.x(),
            1 => center.y(),
            _ => center.z(),
        })
    }
    fn swap(&self, idx1: i32, idx2: i32) {
        self.state
            .write()
            .bvh_prim_indexes
            .swap(idx1 as usize, idx2 as usize);
    }
    fn overlaps_element(
        &self,
        pick_result: &mut SelectBasicsPickResult,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        _is_full_inside: bool,
    ) -> Result<bool, Failure> {
        self.element(elem_idx)
            .get()
            .map_or(Ok(false), |e| e.matches(manager, pick_result))
    }
    fn element_is_inside(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        let mut dummy = SelectBasicsPickResult::default();
        self.overlaps_element(&mut dummy, manager, elem_idx, is_full_inside)
    }
    fn distance_to_cog(&self, manager: &dyn SelectBasicsSelectingVolumeManager) -> f64 {
        manager.dist_to_geometry_center(&self.center_of_geometry())
    }
}
