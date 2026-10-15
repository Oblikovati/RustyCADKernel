// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_PrsBuilder` at OCCT 3d097a0: the abstract builder of the mesh presentations. The
//! virtual members are the [`MeshVSPrsBuilder`] trait over the embedded
//! [`MeshVSPrsBuilderBase`]; the non-virtual members are [`MeshVSPrsBuilderMembers`].
//!
//! The parent mesh is a raw pointer natively; it is a weak handle here, so a builder never
//! keeps its mesh alive. A builder without own data source or drawer reads those of its
//! mesh; without mesh either (a null native dereference) it reads null.
use crate::data_source::{MeshVSDataSource, MeshVSPackedMap};
use crate::drawer::MeshVSDrawerSource;
use crate::mesh::{MeshVSMeshMembers, MeshVSMeshSource};
use rustycad_tkernel::standard::{
    Failure, Handle, SharedState, StandardTransient, Transient, WeakHandle,
};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use rustycad_tkv3d::prs_mgr::PrsMgrPresentationManager;
use rustycad_tkv3d::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkv3d::select3d::Select3DSensitiveEntity;

/// The fields of `MeshVS_PrsBuilder` (`myParentMesh` is protected, the others private).
#[derive(Clone)]
pub struct MeshVSPrsBuilderFields {
    /// `myParentMesh`.
    pub parent_mesh: WeakHandle<dyn MeshVSMeshSource>,
    /// `myIsExcluding`.
    pub is_excluding: bool,
    /// `myDataSource`.
    pub data_source: Handle<dyn MeshVSDataSource>,
    /// `myDrawer`.
    pub drawer: Handle<dyn MeshVSDrawerSource>,
    /// `myFlags`.
    pub flags: i32,
    /// `myId`.
    pub id: i32,
    /// `myPriority`.
    pub priority: i32,
    /// `myPrsMgr`.
    pub prs_mgr: Handle<PrsMgrPresentationManager>,
}

/// The state every presentation builder embeds.
pub struct MeshVSPrsBuilderBase {
    /// The fields; `base.fields.read().id`.
    pub fields: SharedState<MeshVSPrsBuilderFields>,
}

impl MeshVSPrsBuilderBase {
    /// The protected constructor: a negative id takes the first free id of the parent.
    /// Example: `MeshVSPrsBuilderBase::new(&mesh, MESHVS_DMF_OCC_MASK, &Handle::default(), -1, MESHVS_BP_MESH)`.
    pub fn new(
        parent: &Handle<dyn MeshVSMeshSource>,
        flags: i32,
        data_source: &Handle<dyn MeshVSDataSource>,
        id: i32,
        priority: i32,
    ) -> Self {
        let id = match parent.get() {
            Some(mesh) if id < 0 => mesh.get_free_id(),
            _ => id,
        };
        Self {
            fields: SharedState::new(MeshVSPrsBuilderFields {
                parent_mesh: parent.downgrade(),
                is_excluding: false,
                data_source: data_source.clone(),
                drawer: Handle::default(),
                flags,
                id,
                priority,
                prs_mgr: Handle::default(),
            }),
        }
    }
    /// A copy of the fields; `base.read().flags`.
    pub fn read(&self) -> MeshVSPrsBuilderFields {
        self.fields.read().clone()
    }
}

/// A builder of mesh presentations; the default bodies are the native ones.
pub trait MeshVSPrsBuilder: Transient {
    /// The embedded base; `builder.prs_builder_base()`.
    fn prs_builder_base(&self) -> &MeshVSPrsBuilderBase;
    /// `Build` (pure virtual): adds the presentation of the ids (nodes or elements) of a
    /// display mode; the ids drawn may be added to `ids_to_exclude` for the next builders.
    fn build(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        ids: &MeshVSPackedMap,
        ids_to_exclude: &mut MeshVSPackedMap,
        is_element: bool,
        display_mode: i32,
    ) -> Result<(), Failure>;
    /// `CustomBuild`: the presentation of the entities of no known type; none by default.
    fn custom_build(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        ids: &MeshVSPackedMap,
        ids_to_exclude: &mut MeshVSPackedMap,
        display_mode: i32,
    ) -> Result<(), Failure> {
        let _ = (prs, ids, ids_to_exclude, display_mode);
        Ok(())
    }
    /// `CustomSensitiveEntity`: the sensitive entity of an entity of no known type; null by default.
    fn custom_sensitive_entity(
        &self,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        select_mode: i32,
    ) -> Handle<dyn Select3DSensitiveEntity> {
        let _ = (owner, select_mode);
        Handle::default()
    }
    /// `TestFlags`: the display mode shares a flag with the builder; `builder.test_flags(1)`.
    fn test_flags(&self, display_mode: i32) -> bool {
        (display_mode & self.get_flags()) > 0
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn MeshVSPrsBuilder,
    "MeshVS_PrsBuilder",
    StandardTransient
);

/// The non-virtual members of `MeshVS_PrsBuilder`.
pub trait MeshVSPrsBuilderMembers: MeshVSPrsBuilder {
    /// `GetFlags`; `builder.get_flags()`.
    fn get_flags(&self) -> i32 {
        self.prs_builder_base().fields.read().flags
    }
    /// `GetId`; `builder.get_id()`.
    fn get_id(&self) -> i32 {
        self.prs_builder_base().fields.read().id
    }
    /// `GetPriority`; `builder.get_priority()`.
    fn get_priority(&self) -> i32 {
        self.prs_builder_base().fields.read().priority
    }
    /// The parent mesh (null once dropped); `builder.parent_mesh()`.
    fn parent_mesh(&self) -> Handle<dyn MeshVSMeshSource> {
        self.prs_builder_base().fields.read().parent_mesh.upgrade()
    }
    /// `GetDataSource`: the own data source, else the data source of the mesh.
    fn get_data_source(&self) -> Handle<dyn MeshVSDataSource> {
        let own = self.data_source();
        if !own.is_null() {
            return own;
        }
        let parent = self.parent_mesh();
        parent
            .get()
            .map(|mesh| mesh.get_data_source())
            .unwrap_or_default()
    }
    /// `SetDataSource`; `builder.set_data_source(&source)`.
    fn set_data_source(&self, data_source: &Handle<dyn MeshVSDataSource>) {
        self.prs_builder_base().fields.write().data_source = data_source.clone();
    }
    /// `GetDrawer`: the own drawer, else the drawer of the mesh.
    fn get_drawer(&self) -> Handle<dyn MeshVSDrawerSource> {
        let own = self.drawer();
        if !own.is_null() {
            return own;
        }
        let parent = self.parent_mesh();
        parent
            .get()
            .map(|mesh| mesh.get_drawer())
            .unwrap_or_default()
    }
    /// `SetDrawer`; `builder.set_drawer(&drawer)`.
    fn set_drawer(&self, drawer: &Handle<dyn MeshVSDrawerSource>) {
        self.prs_builder_base().fields.write().drawer = drawer.clone();
    }
    /// `SetExcluding`; `builder.set_excluding(true)`.
    fn set_excluding(&self, state: bool) {
        self.prs_builder_base().fields.write().is_excluding = state;
    }
    /// `IsExcludingOn`; `builder.is_excluding_on()`.
    fn is_excluding_on(&self) -> bool {
        self.prs_builder_base().fields.read().is_excluding
    }
    /// `SetPresentationManager`; `builder.set_presentation_manager(&manager)`.
    fn set_presentation_manager(&self, manager: &Handle<PrsMgrPresentationManager>) {
        self.prs_builder_base().fields.write().prs_mgr = manager.clone();
    }
    /// `GetPresentationManager`; `builder.get_presentation_manager()`.
    fn get_presentation_manager(&self) -> Handle<PrsMgrPresentationManager> {
        self.prs_builder_base().fields.read().prs_mgr.clone()
    }
    /// `DataSource` (protected): the own data source only.
    fn data_source(&self) -> Handle<dyn MeshVSDataSource> {
        self.prs_builder_base().fields.read().data_source.clone()
    }
    /// `Drawer` (protected): the own drawer only.
    fn drawer(&self) -> Handle<dyn MeshVSDrawerSource> {
        self.prs_builder_base().fields.read().drawer.clone()
    }
}
impl<T: MeshVSPrsBuilder + ?Sized> MeshVSPrsBuilderMembers for T {}

/// The ids minus a hidden map and minus the excluded ids (the native `anIDs` copies that
/// "minimize allocated memory"). Example: `remaining_ids(&ids, &hidden, &excluded)`.
pub(crate) fn remaining_ids(
    ids: &MeshVSPackedMap,
    hidden: &Handle<rustycad_tkernel::tcol_std::HPackedMapOfInteger>,
    ids_to_exclude: &MeshVSPackedMap,
) -> MeshVSPackedMap {
    let mut remaining = MeshVSPackedMap::default();
    remaining.assign(ids);
    if let Some(hidden) = hidden.get() {
        rustycad_tkernel::ncollection::packed_map_algo::subtract(&mut remaining, &hidden.map());
    }
    rustycad_tkernel::ncollection::packed_map_algo::subtract(&mut remaining, ids_to_exclude);
    remaining
}

/// The hidden elements of the parent mesh (null without parent).
pub(crate) fn parent_hidden_elems<B: MeshVSPrsBuilder + ?Sized>(
    builder: &B,
) -> Handle<rustycad_tkernel::tcol_std::HPackedMapOfInteger> {
    let parent = builder.parent_mesh();
    parent
        .get()
        .map(|mesh| mesh.get_hidden_elems())
        .unwrap_or_default()
}

/// A one-based coordinate array of `3 * max_nodes` values.
pub(crate) fn coords_of(
    max_nodes: i32,
) -> Result<rustycad_tkernel::ncollection::Array1<'static, f64>, Failure> {
    crate::data_source::normals::coords_buffer(max_nodes)
}
