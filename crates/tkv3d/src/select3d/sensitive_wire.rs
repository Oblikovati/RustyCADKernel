//! `Select3D_SensitiveWire`: sensitive edges kept in a BVH tree, sharing the owner of the wire.
use super::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityMembers as _, Select3DSensitiveSet, Select3DSensitiveSetBase,
    select3d_set_build_bvh, select3d_set_dump_json, select3d_set_matches,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::exceptions::null_object;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::json_dump::{base_class, class_begin, field_dumped};
use std::io;

/// The mutable state of a wire.
#[derive(Clone, Default)]
struct WireState {
    entities: Vec<Handle<dyn Select3DSensitiveEntity>>,
    entity_indexes: Vec<i32>,
    center: Point3,
    bnd_box: Select3DBndBox3d,
}

/// A sensitive wire.
///
/// Example: `let wire = Select3DSensitiveWire::new(&owner); wire.add(&edge)?;`.
pub struct Select3DSensitiveWire {
    transient: StandardTransient,
    entity: Select3DSensitiveEntityBase,
    set: Select3DSensitiveSetBase,
    state: SharedState<WireState>,
}
rustycad_tkernel::impl_transient!(Select3DSensitiveWire, "Select3D_SensitiveWire", dyn Select3DSensitiveSet, transient,
    interfaces: [dyn Select3DSensitiveEntity, dyn Select3DSensitiveSet]);

impl Select3DSensitiveWire {
    /// An empty wire of an owner; `Select3DSensitiveWire::new(&owner)`.
    pub fn new(owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> Self {
        Self {
            transient: StandardTransient::default(),
            entity: Select3DSensitiveEntityBase::new(owner),
            set: Select3DSensitiveSetBase::default(),
            state: SharedState::new(WireState::default()),
        }
    }
    /// `Add`: an edge, combined into the box; the center is the running half-sum of the edge
    /// centers (natively not a mean); a null edge raises `Standard_NullObject` (a null
    /// dereference natively). Example: `wire.add(&edge)?`.
    pub fn add(&self, sensitive: &Handle<dyn Select3DSensitiveEntity>) -> Result<(), Failure> {
        let Some(value) = sensitive.get() else {
            return Err(null_object(
                "Select3D_SensitiveWire::Add: null sensitive entity; expected an edge",
            ));
        };
        let (edge_box, edge_center) = (value.bounding_box(), value.center_of_geometry());
        let mut state = self.state.write();
        state.entities.push(sensitive.clone());
        state.bnd_box.combine(&edge_box);
        let mut center = state.center.xyz() + edge_center.xyz();
        if state.entities.len() != 1 {
            center /= 2.0;
        }
        state.center = Point3::from(center);
        let last = state.entities.len() as i32 - 1;
        state.entity_indexes.push(last);
        Ok(())
    }
    /// `GetEdges`; `wire.get_edges()`.
    pub fn get_edges(&self) -> Vec<Handle<dyn Select3DSensitiveEntity>> {
        self.state.read().entities.clone()
    }
    /// `GetLastDetected`: the edge detected by the last match, null without.
    /// Example: `wire.get_last_detected()`.
    pub fn get_last_detected(&self) -> Handle<dyn Select3DSensitiveEntity> {
        let detected = self.set.detected_idx();
        let state = self.state.read();
        if detected < 0 || detected as usize >= state.entities.len() {
            return Handle::default();
        }
        let index = state.entity_indexes[detected as usize];
        state.entities[index as usize].clone()
    }
    /// The edge at a BVH index.
    fn edge(&self, idx: i32) -> Option<Handle<dyn Select3DSensitiveEntity>> {
        let state = self.state.read();
        let index = *state.entity_indexes.get(idx as usize)?;
        state.entities.get(index as usize).cloned()
    }
}

impl Select3DSensitiveEntity for Select3DSensitiveWire {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        &self.entity
    }
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        select3d_set_matches(self, manager, pick_result, false)
    }
    fn nb_sub_elements(&self) -> i32 {
        self.state.read().entities.len() as i32
    }
    /// `BoundingBox`: the combined box, computed again when invalid.
    fn bounding_box(&self) -> Select3DBndBox3d {
        let entities = {
            let state = self.state.read();
            if state.bnd_box.is_valid() {
                return state.bnd_box;
            }
            state.entities.clone()
        };
        let mut bounds = self.state.read().bnd_box;
        for edge in entities.iter().filter_map(Handle::get) {
            bounds.combine(&edge.bounding_box());
        }
        self.state.write().bnd_box = bounds;
        bounds
    }
    fn center_of_geometry(&self) -> Point3 {
        self.state.read().center
    }
    /// `Set`: the owner of the wire and of every edge.
    fn set(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) {
        self.entity.fields.write().owner_id = owner.clone();
        for edge in self.get_edges().iter().filter_map(Handle::get) {
            edge.set(owner);
        }
    }
    /// `GetConnected`: a wire of the connected edges.
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        let wire = Self::new(&self.owner_id());
        for edge in self.get_edges().iter().filter_map(Handle::get) {
            wire.add(&edge.get_connected()?)?;
        }
        Ok(Handle::new(wire).cast())
    }
    fn bvh(&self) -> Result<(), Failure> {
        select3d_set_build_bvh(self)
    }
    fn to_build_bvh(&self) -> bool {
        self.set.is_dirty()
    }
    fn clear(&self) {}
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Select3D_SensitiveWire")?;
        base_class(stream, depth, &|s, d| select3d_set_dump_json(self, s, d))?;
        let state = self.state.read().clone();
        for edge in state.entities.iter().filter_map(Handle::get) {
            field_dumped(stream, depth, "anEntity.get()", &|s, d| {
                edge.dump_json(s, d)
            })?;
        }
        let bounds = state.bnd_box;
        field_dumped(stream, depth, "&myBndBox", &|s, _| bounds.dump_json(s))
    }
}

impl Select3DSensitiveSet for Select3DSensitiveWire {
    fn set_base(&self) -> &Select3DSensitiveSetBase {
        &self.set
    }
    fn size(&self) -> i32 {
        self.state.read().entities.len() as i32
    }
    fn element_box(&self, idx: i32) -> Select3DBndBox3d {
        self.edge(idx)
            .and_then(|e| e.get().map(|v| v.bounding_box()))
            .unwrap_or_default()
    }
    fn center(&self, idx: i32, axis: i32) -> Result<f64, Failure> {
        let center = self
            .edge(idx)
            .and_then(|e| e.get().map(|v| v.center_of_geometry()))
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
            .entity_indexes
            .swap(idx1 as usize, idx2 as usize);
    }
    /// `overlapsElement`: the edge matches.
    fn overlaps_element(
        &self,
        pick_result: &mut SelectBasicsPickResult,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        _is_full_inside: bool,
    ) -> Result<bool, Failure> {
        match self.edge(elem_idx) {
            Some(edge) => edge
                .get()
                .map_or(Ok(false), |e| e.matches(manager, pick_result)),
            None => Ok(false),
        }
    }
    fn element_is_inside(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        let mut result = SelectBasicsPickResult::default();
        self.overlaps_element(&mut result, manager, elem_idx, is_full_inside)
    }
    fn distance_to_cog(&self, manager: &dyn SelectBasicsSelectingVolumeManager) -> f64 {
        manager.dist_to_geometry_center(&self.center_of_geometry())
    }
}
