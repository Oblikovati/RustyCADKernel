// SPDX-License-Identifier: GPL-2.0-only
//! The non-virtual members of `MeshVS_Mesh`: data source, drawer, owner maps and selection
//! method. The builder members are in builders.rs, the hidden and selectable maps in
//! visibility.rs.
use super::{MeshVSMeshSource, MeshVSOwnerMap, builders, visibility};
use crate::data_source::MeshVSDataSource;
use crate::drawer::MeshVSDrawerSource;
use crate::prs_builder::MeshVSPrsBuilder;
use crate::selection_policy::MeshVSMeshSelectionMethod;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Handle, RuntimeType};
use rustycad_tkernel::tcol_std::HPackedMapOfInteger;

/// The non-virtual members of `MeshVS_Mesh`.
pub trait MeshVSMeshMembers: MeshVSMeshSource {
    /// `GetBuildersCount`; `mesh.get_builders_count()`.
    fn get_builders_count(&self) -> i32 {
        self.mesh_base().fields.read().builders.len() as i32
    }
    /// `GetBuilder`: the builder of a one-based index, null outside; `mesh.get_builder(1)`.
    fn get_builder(&self, index: i32) -> Handle<dyn MeshVSPrsBuilder> {
        builders::builder_at(self.mesh_base(), index)
    }
    /// `GetBuilderById`: the first builder of an id, null without; `mesh.get_builder_by_id(0)`.
    fn get_builder_by_id(&self, id: i32) -> Handle<dyn MeshVSPrsBuilder> {
        builders::builder_by_id(self.mesh_base(), id)
    }
    /// `GetFreeId`: the smallest non-negative id of no builder; `mesh.get_free_id()`.
    fn get_free_id(&self) -> i32 {
        builders::free_id(self.mesh_base())
    }
    /// `AddBuilder`: inserts before the first builder of a lower priority; a null builder is
    /// ignored. Example: `mesh.add_builder(&builder, true)`.
    fn add_builder(&self, builder: &Handle<dyn MeshVSPrsBuilder>, treat_as_hilighter: bool) {
        builders::add_builder(self.mesh_base(), builder, treat_as_hilighter);
    }
    /// `SetHilighter(Builder)`; `mesh.set_hilighter(&builder)`.
    fn set_hilighter(&self, builder: &Handle<dyn MeshVSPrsBuilder>) {
        self.mesh_base().fields.write().hilighter = builder.clone();
    }
    /// `SetHilighter(Index)`: the builder of an index, false when null; `mesh.set_hilighter_at(1)`.
    fn set_hilighter_at(&self, index: i32) -> bool {
        let builder = self.get_builder(index);
        let found = !builder.is_null();
        if found {
            self.set_hilighter(&builder);
        }
        found
    }
    /// `SetHilighterById`; `mesh.set_hilighter_by_id(0)`.
    fn set_hilighter_by_id(&self, id: i32) -> bool {
        let builder = self.get_builder_by_id(id);
        let found = !builder.is_null();
        if found {
            self.set_hilighter(&builder);
        }
        found
    }
    /// `GetHilighter`; `mesh.get_hilighter()`.
    fn get_hilighter(&self) -> Handle<dyn MeshVSPrsBuilder> {
        self.mesh_base().fields.read().hilighter.clone()
    }
    /// `RemoveBuilder`: removes the builder of an index, forgetting it as highlighter.
    fn remove_builder(&self, index: i32) {
        builders::remove_builder(self.mesh_base(), index);
    }
    /// `RemoveBuilderById`: removes the first builder of an id.
    fn remove_builder_by_id(&self, id: i32) {
        builders::remove_builder_by_id(self.mesh_base(), id);
    }
    /// `FindBuilder(const Handle(Standard_Type)&)`: the first builder of a kind, null without.
    fn find_builder(&self, kind: &RuntimeType) -> Handle<dyn MeshVSPrsBuilder> {
        builders::find_builder(self.mesh_base(), kind)
    }
    /// `GetOwnerMaps`: the mesh owners with advanced selection, else the element owners
    /// (merged on first use) or the node owners. Example: `mesh.get_owner_maps(true)`.
    fn get_owner_maps(&self, is_element: bool) -> MeshVSOwnerMap {
        visibility::owner_maps(self, is_element)
    }
    /// `GetDataSource`; `mesh.get_data_source()`.
    fn get_data_source(&self) -> Handle<dyn MeshVSDataSource> {
        self.mesh_base().fields.read().data_source.clone()
    }
    /// `SetDataSource`; `mesh.set_data_source(&source)`.
    fn set_data_source(&self, data_source: &Handle<dyn MeshVSDataSource>) {
        self.mesh_base().fields.write().data_source = data_source.clone();
    }
    /// `GetDrawer`; `mesh.get_drawer()`.
    fn get_drawer(&self) -> Handle<dyn MeshVSDrawerSource> {
        self.mesh_base().fields.read().current_drawer.clone()
    }
    /// `SetDrawer`; `mesh.set_drawer(&drawer)`.
    fn set_drawer(&self, drawer: &Handle<dyn MeshVSDrawerSource>) {
        self.mesh_base().fields.write().current_drawer = drawer.clone();
    }
    /// `IsHiddenElem`: listed in the hidden elements; `mesh.is_hidden_elem(3)`.
    fn is_hidden_elem(&self, id: i32) -> bool {
        let hidden = self.get_hidden_elems();
        hidden.get().is_some_and(|map| map.map().contains(id))
    }
    /// `IsHiddenNode`: listed in the hidden nodes; without that map every node is hidden.
    fn is_hidden_node(&self, id: i32) -> bool {
        let hidden = self.get_hidden_nodes();
        hidden.get().is_none_or(|map| map.map().contains(id))
    }
    /// `IsSelectableElem`: not hidden; `mesh.is_selectable_elem(3)`.
    fn is_selectable_elem(&self, id: i32) -> bool {
        !self.is_hidden_elem(id)
    }
    /// `IsSelectableNode`: in the selectable nodes, else not hidden; `mesh.is_selectable_node(3)`.
    fn is_selectable_node(&self, id: i32) -> bool {
        let selectable = self.get_selectable_nodes();
        match selectable.get() {
            Some(map) => map.map().contains(id),
            None => !self.is_hidden_node(id),
        }
    }
    /// `GetHiddenNodes`; `mesh.get_hidden_nodes()`.
    fn get_hidden_nodes(&self) -> Handle<HPackedMapOfInteger> {
        self.mesh_base().fields.read().hidden_nodes.clone()
    }
    /// `SetHiddenNodes`: then the selectable nodes when the drawer asks `MeshVS_DA_SelectableAuto`.
    fn set_hidden_nodes(&self, ids: &Handle<HPackedMapOfInteger>) {
        self.mesh_base().fields.write().hidden_nodes = ids.clone();
        visibility::auto_update_selectable(self);
    }
    /// `GetHiddenElems`; `mesh.get_hidden_elems()`.
    fn get_hidden_elems(&self) -> Handle<HPackedMapOfInteger> {
        self.mesh_base().fields.read().hidden_elements.clone()
    }
    /// `SetHiddenElems`: then the selectable nodes when the drawer asks `MeshVS_DA_SelectableAuto`.
    fn set_hidden_elems(&self, ids: &Handle<HPackedMapOfInteger>) {
        self.mesh_base().fields.write().hidden_elements = ids.clone();
        visibility::auto_update_selectable(self);
    }
    /// `GetSelectableNodes`; `mesh.get_selectable_nodes()`.
    fn get_selectable_nodes(&self) -> Handle<HPackedMapOfInteger> {
        self.mesh_base().fields.read().selectable_nodes.clone()
    }
    /// `SetSelectableNodes`; `mesh.set_selectable_nodes(&ids)`.
    fn set_selectable_nodes(&self, ids: &Handle<HPackedMapOfInteger>) {
        self.mesh_base().fields.write().selectable_nodes = ids.clone();
    }
    /// `UpdateSelectableNodes`: the visible nodes and the nodes of the visible elements.
    fn update_selectable_nodes(&self) {
        visibility::update_selectable_nodes(self);
    }
    /// `GetMeshSelMethod`; `mesh.get_mesh_sel_method()`.
    fn get_mesh_sel_method(&self) -> MeshVSMeshSelectionMethod {
        self.mesh_base().fields.read().selection_method
    }
    /// `SetMeshSelMethod`; `mesh.set_mesh_sel_method(MeshVSMeshSelectionMethod::Box)`.
    fn set_mesh_sel_method(&self, method: MeshVSMeshSelectionMethod) {
        self.mesh_base().fields.write().selection_method = method;
    }
}
impl<T: MeshVSMeshSource + ?Sized> MeshVSMeshMembers for T {}

/// The native `AddToMap` helper: binds or replaces every owner of `addition` in `result`.
pub(super) fn add_to_map(result: &mut MeshVSOwnerMap, addition: &MeshVSOwnerMap) {
    for (key, value) in addition.items() {
        match result.change_find(key) {
            Ok(slot) => *slot = value.clone(),
            Err(_) => {
                result.bind(*key, value.clone());
            }
        }
    }
}

/// A copy of an owner map (the native const reference, read once).
pub(super) fn copy_map(map: &MeshVSOwnerMap) -> MeshVSOwnerMap {
    let mut copy = DataMap::default();
    copy.assign(map);
    copy
}
