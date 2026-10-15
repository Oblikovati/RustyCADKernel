// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_Mesh` at OCCT 3d097a0: the interactive object of a mesh data source, drawn by its
//! presentation builders. The overridable members are the [`MeshVSMeshSource`] trait over the
//! embedded [`MeshVSMeshBase`]; the native bodies of its AIS overrides are the
//! `meshvs_mesh_*` functions, which a derived mesh may call as `MeshVS_Mesh::Member`.
//!
//! ```
//! use martensite_tkmeshvs::{MeshVSMesh, MeshVSMeshMembers};
//! use rustycad_tkv3d::prs_mgr::PrsMgrPresentableObjectMembers;
//! let mesh = MeshVSMesh::new(false).unwrap();
//! assert_eq!(mesh.display_mode(), 1);
//! assert_eq!(mesh.get_builders_count(), 0);
//! assert!(mesh.is_hidden_node(1));
//! ```
mod builders;
mod compute;
mod defaults;
mod highlight;
mod members;
mod selection;
mod selection_elements;
mod selection_whole;
mod visibility;

pub use compute::{meshvs_mesh_accept_display_mode, meshvs_mesh_compute};
pub use highlight::{
    meshvs_mesh_clear_selected, meshvs_mesh_hilight_owner_with_color, meshvs_mesh_hilight_selected,
};
pub use members::MeshVSMeshMembers;
pub use selection::meshvs_mesh_compute_selection;

use crate::data_source::MeshVSDataSource;
use crate::drawer::MeshVSDrawerSource;
use crate::prs_builder::MeshVSPrsBuilder;
use crate::selection_policy::MeshVSMeshSelectionMethod;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{
    Failure, Handle, HandleReference, SharedState, StandardTransient,
};
use rustycad_tkernel::tcol_std::HPackedMapOfInteger;
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use rustycad_tkv3d::ais::{AisInteractiveObject, AisInteractiveObjectBase};
use rustycad_tkv3d::prs_mgr::{
    PrsMgrPresentableObject, PrsMgrPresentableObjectBase, PrsMgrPresentationManager,
    PrsMgrTypeOfPresentation3d, prs_mgr_presentable_bind,
};
use rustycad_tkv3d::prs3d::Prs3dDrawer;
use rustycad_tkv3d::select_mgr::{
    SelectMgrEntityOwnerSource, SelectMgrSelectableObject, SelectMgrSelectableObjectBase,
    SelectMgrSelection,
};

/// The native `NCollection_DataMap<int, occ::handle<SelectMgr_EntityOwner>>` of owners.
pub type MeshVSOwnerMap = DataMap<i32, Handle<dyn SelectMgrEntityOwnerSource>>;

/// The fields of `MeshVS_Mesh`: the protected owner maps and drawers, then the private ones.
pub struct MeshVSMeshFields {
    /// `myNodeOwners`.
    pub node_owners: MeshVSOwnerMap,
    /// `myElementOwners`.
    pub element_owners: MeshVSOwnerMap,
    /// `my0DOwners`.
    pub owners_0d: MeshVSOwnerMap,
    /// `myLinkOwners`.
    pub link_owners: MeshVSOwnerMap,
    /// `myFaceOwners`.
    pub face_owners: MeshVSOwnerMap,
    /// `myVolumeOwners`.
    pub volume_owners: MeshVSOwnerMap,
    /// `myGroupOwners`.
    pub group_owners: MeshVSOwnerMap,
    /// `myMeshOwners`.
    pub mesh_owners: MeshVSOwnerMap,
    /// `myCurrentDrawer`.
    pub current_drawer: Handle<dyn MeshVSDrawerSource>,
    /// `mySelectionDrawer`.
    pub selection_drawer: Handle<dyn MeshVSDrawerSource>,
    /// `myHilightDrawer`.
    pub hilight_drawer: Handle<dyn MeshVSDrawerSource>,
    /// `myWholeMeshOwner`.
    pub whole_mesh_owner: Handle<dyn SelectMgrEntityOwnerSource>,
    /// `myBuilders`, ordered by decreasing priority.
    pub builders: Vec<Handle<dyn MeshVSPrsBuilder>>,
    /// `myHilighter`.
    pub hilighter: Handle<dyn MeshVSPrsBuilder>,
    /// `myHiddenElements`.
    pub hidden_elements: Handle<HPackedMapOfInteger>,
    /// `myHiddenNodes`.
    pub hidden_nodes: Handle<HPackedMapOfInteger>,
    /// `mySelectableNodes`.
    pub selectable_nodes: Handle<HPackedMapOfInteger>,
    /// `myDataSource`.
    pub data_source: Handle<dyn MeshVSDataSource>,
    /// `mySelectionMethod`.
    pub selection_method: MeshVSMeshSelectionMethod,
}

impl MeshVSMeshFields {
    /// The constructor state: no owner, drawer, builder, hidden map nor data source.
    fn empty() -> Self {
        Self {
            node_owners: DataMap::default(),
            element_owners: DataMap::default(),
            owners_0d: DataMap::default(),
            link_owners: DataMap::default(),
            face_owners: DataMap::default(),
            volume_owners: DataMap::default(),
            group_owners: DataMap::default(),
            mesh_owners: DataMap::default(),
            current_drawer: Handle::default(),
            selection_drawer: Handle::default(),
            hilight_drawer: Handle::default(),
            whole_mesh_owner: Handle::default(),
            builders: Vec::new(),
            hilighter: Handle::default(),
            hidden_elements: Handle::default(),
            hidden_nodes: Handle::default(),
            selectable_nodes: Handle::default(),
            data_source: Handle::default(),
            selection_method: MeshVSMeshSelectionMethod::Nodes,
        }
    }
}

/// The state every mesh embeds: the AIS base and the mesh fields.
pub struct MeshVSMeshBase {
    /// The inherited interactive state.
    pub interactive: AisInteractiveObjectBase,
    /// The mesh fields; `base.fields.read().builders.len()`.
    pub fields: SharedState<MeshVSMeshFields>,
}

impl MeshVSMeshBase {
    /// The AIS base of every view and the default selection method `MeshVS_MSM_NODES`; the
    /// drawers and the appearance come from [`meshvs_mesh_initialize`] once the object exists.
    /// Example: `MeshVSMeshBase::new()`.
    pub fn new() -> Self {
        Self {
            interactive: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            fields: SharedState::new(MeshVSMeshFields::empty()),
        }
    }
}

impl Default for MeshVSMeshBase {
    fn default() -> Self {
        Self::new()
    }
}

pub use defaults::meshvs_mesh_initialize;

/// A mesh interactive object; the default bodies are the native ones.
pub trait MeshVSMeshSource: AisInteractiveObject {
    /// The embedded base; `mesh.mesh_base()`.
    fn mesh_base(&self) -> &MeshVSMeshBase;
    /// `IsWholeMeshOwner`: the owner is the whole-mesh owner of the mode `MeshVS_SMF_Mesh`.
    fn is_whole_mesh_owner(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> bool {
        let whole = self
            .mesh_base()
            .fields
            .read()
            .whole_mesh_owner
            .object_address();
        !owner.is_null() && owner.object_address() == whole
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn MeshVSMeshSource,
    "MeshVS_Mesh",
    dyn AisInteractiveObject
);

/// The concrete `MeshVS_Mesh`; `MeshVSMesh::new(false)?`.
pub struct MeshVSMesh {
    transient: StandardTransient,
    base: MeshVSMeshBase,
}
rustycad_tkernel::impl_transient!(MeshVSMesh, "MeshVS_Mesh", dyn AisInteractiveObject, transient,
    interfaces: [dyn MeshVSMeshSource, dyn AisInteractiveObject, dyn SelectMgrSelectableObject, dyn PrsMgrPresentableObject]);

impl MeshVSMesh {
    /// `MeshVS_Mesh(theIsAllowOverlapped)`: the native drawers and appearance.
    /// Example: `MeshVSMesh::new(false)?`.
    pub fn new(allow_overlapped: bool) -> Result<Handle<Self>, Failure> {
        let mesh = Handle::new(Self {
            transient: StandardTransient::default(),
            base: MeshVSMeshBase::new(),
        });
        prs_mgr_presentable_bind(&mesh.clone().cast());
        meshvs_mesh_initialize(&*mesh, allow_overlapped)?;
        Ok(mesh)
    }
}

impl MeshVSMeshSource for MeshVSMesh {
    fn mesh_base(&self) -> &MeshVSMeshBase {
        &self.base
    }
}

impl AisInteractiveObject for MeshVSMesh {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base.interactive
    }
}

impl PrsMgrPresentableObject for MeshVSMesh {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.interactive.presentable
    }
    fn compute(
        &self,
        manager: &dyn HandleReference<PrsMgrPresentationManager>,
        prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        meshvs_mesh_compute(self, &manager.observe(), &prs.observe(), mode)
    }
    fn accept_display_mode(&self, mode: i32) -> bool {
        meshvs_mesh_accept_display_mode(self, mode)
    }
    rustycad_tkv3d::ais_presentable_overrides!();
}

impl SelectMgrSelectableObject for MeshVSMesh {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.interactive.selectable
    }
    fn compute_selection(
        &self,
        selection: &dyn HandleReference<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        meshvs_mesh_compute_selection(self, &selection.observe(), mode)
    }
    fn hilight_selected(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        owners: &[Handle<dyn SelectMgrEntityOwnerSource>],
    ) -> Result<(), Failure> {
        meshvs_mesh_hilight_selected(self, manager, owners)
    }
    fn hilight_owner_with_color(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        style: &Handle<Prs3dDrawer>,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    ) -> Result<(), Failure> {
        meshvs_mesh_hilight_owner_with_color(self, manager, style, owner)
    }
    fn clear_selected(&self) {
        meshvs_mesh_clear_selected(self);
    }
}
