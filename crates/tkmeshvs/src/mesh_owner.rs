// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_MeshOwner` at OCCT 3d097a0: the single owner of a mesh with advanced selection,
//! holding the entities its data source detected and selected. The overridable members are
//! [`MeshVSMeshOwnerSource`] over the embedded [`MeshVSMeshOwnerBase`].
use crate::data_source::MeshVSDataSource;
use crate::mesh_entity_owner::hilight_by_mesh;
use rustycad_tkernel::ncollection::packed_map_algo;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcol_std::HPackedMapOfInteger;
use rustycad_tkv3d::prs_mgr::PrsMgrPresentationManager;
use rustycad_tkv3d::prs3d::Prs3dDrawer;
use rustycad_tkv3d::select_mgr::{
    SelectMgrEntityOwnerBase, SelectMgrEntityOwnerMembers, SelectMgrEntityOwnerSource,
    SelectMgrSelectableObject, select_mgr_entity_owner_bind,
};

/// The fields of `MeshVS_MeshOwner`: the protected selected maps, then the private ones.
#[derive(Clone, Default)]
pub struct MeshVSMeshOwnerFields {
    /// `mySelectedNodes`.
    pub selected_nodes: Handle<HPackedMapOfInteger>,
    /// `mySelectedElems`.
    pub selected_elems: Handle<HPackedMapOfInteger>,
    /// `myDataSource`.
    pub data_source: Handle<dyn MeshVSDataSource>,
    /// `myDetectedNodes`.
    pub detected_nodes: Handle<HPackedMapOfInteger>,
    /// `myDetectedElems`.
    pub detected_elems: Handle<HPackedMapOfInteger>,
    /// `myLastID`.
    pub last_id: i32,
}

/// The state every mesh owner embeds.
pub struct MeshVSMeshOwnerBase {
    /// The inherited owner state.
    pub owner: SelectMgrEntityOwnerBase,
    /// The mesh owner fields.
    pub fields: SharedState<MeshVSMeshOwnerFields>,
}

impl MeshVSMeshOwnerBase {
    /// `MeshVS_MeshOwner(theSelObj, theDS, thePriority)`: no entity and last id -1.
    /// Example: `MeshVSMeshOwnerBase::new(&selectable, &source, 5)`.
    pub fn new(
        selectable: &Handle<dyn SelectMgrSelectableObject>,
        data_source: &Handle<dyn MeshVSDataSource>,
        priority: i32,
    ) -> Self {
        Self {
            owner: SelectMgrEntityOwnerBase::new(selectable, priority),
            fields: SharedState::new(MeshVSMeshOwnerFields {
                data_source: data_source.clone(),
                last_id: -1,
                ..Default::default()
            }),
        }
    }
}

/// The mesh owner interface; the default bodies are the native ones.
pub trait MeshVSMeshOwnerSource: SelectMgrEntityOwnerSource {
    /// The embedded base; `owner.mesh_owner_base()`.
    fn mesh_owner_base(&self) -> &MeshVSMeshOwnerBase;
    /// `AddSelectedEntities`: the maps when none, else united into the selected maps.
    fn add_selected_entities(
        &self,
        nodes: &Handle<HPackedMapOfInteger>,
        elems: &Handle<HPackedMapOfInteger>,
    ) {
        let mut fields = self.mesh_owner_base().fields.write();
        unite_selected(&mut fields.selected_nodes, nodes);
        unite_selected(&mut fields.selected_elems, elems);
    }
    /// `ClearSelectedEntities`: forgets the selected maps.
    fn clear_selected_entities(&self) {
        let mut fields = self.mesh_owner_base().fields.write();
        fields.selected_nodes = Handle::default();
        fields.selected_elems = Handle::default();
    }
    /// `GetDataSource`; `owner.get_data_source()`.
    fn get_data_source(&self) -> Handle<dyn MeshVSDataSource> {
        self.mesh_owner_base().fields.read().data_source.clone()
    }
    /// `GetSelectedNodes`; `owner.get_selected_nodes()`.
    fn get_selected_nodes(&self) -> Handle<HPackedMapOfInteger> {
        self.mesh_owner_base().fields.read().selected_nodes.clone()
    }
    /// `GetSelectedElements`; `owner.get_selected_elements()`.
    fn get_selected_elements(&self) -> Handle<HPackedMapOfInteger> {
        self.mesh_owner_base().fields.read().selected_elems.clone()
    }
    /// `GetDetectedNodes`; `owner.get_detected_nodes()`.
    fn get_detected_nodes(&self) -> Handle<HPackedMapOfInteger> {
        self.mesh_owner_base().fields.read().detected_nodes.clone()
    }
    /// `GetDetectedElements`; `owner.get_detected_elements()`.
    fn get_detected_elements(&self) -> Handle<HPackedMapOfInteger> {
        self.mesh_owner_base().fields.read().detected_elems.clone()
    }
    /// `SetDetectedEntities`: the detected maps; a selected owner becomes unselected.
    fn set_detected_entities(
        &self,
        nodes: &Handle<HPackedMapOfInteger>,
        elems: &Handle<HPackedMapOfInteger>,
    ) {
        {
            let mut fields = self.mesh_owner_base().fields.write();
            fields.detected_nodes = nodes.clone();
            fields.detected_elems = elems.clone();
        }
        if self.is_selected() {
            self.set_selected(false);
        }
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn MeshVSMeshOwnerSource,
    "MeshVS_MeshOwner",
    dyn SelectMgrEntityOwnerSource
);

fn unite_selected(selected: &mut Handle<HPackedMapOfInteger>, added: &Handle<HPackedMapOfInteger>) {
    let Some(current) = selected.get() else {
        *selected = added.clone();
        return;
    };
    if let Some(added) = added.get() {
        if std::ptr::eq(current, added) {
            return;
        }
        packed_map_algo::unite(&mut current.change_map(), &added.map());
    }
}

/// The single key of a one-entity map; `None` for another count.
fn single_key(map: &Handle<HPackedMapOfInteger>) -> Option<i32> {
    let map = map.get()?.map();
    if map.extent() != 1 {
        return None;
    }
    map.iter().next()
}

/// The native `MeshVS_MeshOwner::HilightWithColor` body: records the last single detected
/// entity, then the mesh highlights the owner in immediate mode.
pub fn meshvs_mesh_owner_hilight_with_color<O: MeshVSMeshOwnerSource + ?Sized>(
    owner: &O,
    manager: &Handle<PrsMgrPresentationManager>,
    style: &Handle<Prs3dDrawer>,
) -> Result<(), Failure> {
    let immediate = manager.get().is_some_and(|m| m.is_immediate_mode_on());
    let is_mesh = owner
        .selectable()
        .cast::<dyn crate::mesh::MeshVSMeshSource>()
        .get()
        .is_some();
    if immediate && is_mesh {
        let last = single_key(&owner.get_detected_nodes())
            .or_else(|| single_key(&owner.get_detected_elements()));
        if let Some(last) = last {
            owner.mesh_owner_base().fields.write().last_id = last;
        }
    }
    hilight_by_mesh(owner, manager, style)
}

/// The native `MeshVS_MeshOwner::Unhilight` body: the base unhighlight, then the last id is
/// forgotten unless a detected map misses it.
pub fn meshvs_mesh_owner_unhilight<O: MeshVSMeshOwnerSource + ?Sized>(
    owner: &O,
    manager: &Handle<PrsMgrPresentationManager>,
) {
    let selectable = owner.selectable();
    if let (Some(_), Some(manager)) = (selectable.get(), manager.get()) {
        manager.unhighlight(&selectable.cast());
    }
    let last = owner.mesh_owner_base().fields.read().last_id;
    let misses =
        |map: Handle<HPackedMapOfInteger>| map.get().is_some_and(|m| !m.map().contains(last));
    if misses(owner.get_detected_nodes()) || misses(owner.get_detected_elements()) {
        return;
    }
    owner.mesh_owner_base().fields.write().last_id = -1;
}

/// The native `MeshVS_MeshOwner::IsForcedHilight` body: forced unless the single detected
/// entity is the last one highlighted.
pub fn meshvs_mesh_owner_is_forced_hilight<O: MeshVSMeshOwnerSource + ?Sized>(owner: &O) -> bool {
    let last = owner.mesh_owner_base().fields.read().last_id;
    if last <= 0 {
        return true;
    }
    let same_node = single_key(&owner.get_detected_nodes()) == Some(last);
    let same_elem = single_key(&owner.get_detected_elements()) == Some(last);
    !(same_node || same_elem)
}

/// The concrete `MeshVS_MeshOwner`; `MeshVSMeshOwner::new(&selectable, &source, 5)`.
pub struct MeshVSMeshOwner {
    transient: StandardTransient,
    base: MeshVSMeshOwnerBase,
}
rustycad_tkernel::impl_transient!(MeshVSMeshOwner, "MeshVS_MeshOwner", dyn SelectMgrEntityOwnerSource, transient,
    interfaces: [dyn MeshVSMeshOwnerSource, dyn SelectMgrEntityOwnerSource]);

impl MeshVSMeshOwner {
    /// `MeshVS_MeshOwner(theSelObj, theDS, thePriority)`; the bound handle.
    pub fn new(
        selectable: &Handle<dyn SelectMgrSelectableObject>,
        data_source: &Handle<dyn MeshVSDataSource>,
        priority: i32,
    ) -> Handle<Self> {
        let owner = Handle::new(Self {
            transient: StandardTransient::default(),
            base: MeshVSMeshOwnerBase::new(selectable, data_source, priority),
        });
        select_mgr_entity_owner_bind(&owner.clone().cast());
        owner
    }
}

impl MeshVSMeshOwnerSource for MeshVSMeshOwner {
    fn mesh_owner_base(&self) -> &MeshVSMeshOwnerBase {
        &self.base
    }
}

impl SelectMgrEntityOwnerSource for MeshVSMeshOwner {
    fn owner_base(&self) -> &SelectMgrEntityOwnerBase {
        &self.base.owner
    }
    fn hilight_with_color(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        style: &Handle<Prs3dDrawer>,
        _: i32,
    ) -> Result<(), Failure> {
        meshvs_mesh_owner_hilight_with_color(self, manager, style)
    }
    fn unhilight(&self, manager: &Handle<PrsMgrPresentationManager>, _: i32) {
        meshvs_mesh_owner_unhilight(self, manager);
    }
    fn is_forced_hilight(&self) -> bool {
        meshvs_mesh_owner_is_forced_hilight(self)
    }
}
