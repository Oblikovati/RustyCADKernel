// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_MeshEntityOwner` at OCCT 3d097a0: the owner of one node, element or group of a
//! mesh. It never reports itself highlighted; in immediate mode it asks its mesh to
//! highlight it.
use crate::MeshVSEntityType;
use crate::mesh::MeshVSMeshSource;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkv3d::prs_mgr::PrsMgrPresentationManager;
use rustycad_tkv3d::prs3d::Prs3dDrawer;
use rustycad_tkv3d::select_mgr::{
    SelectMgrEntityOwnerBase, SelectMgrEntityOwnerMembers, SelectMgrEntityOwnerSource,
    SelectMgrSelectableObject, select_mgr_entity_owner_bind,
};

/// The private fields of `MeshVS_MeshEntityOwner`.
#[derive(Clone, Copy, Debug)]
pub struct MeshVSMeshEntityOwnerFields {
    /// `myAddr`: the opaque address of the entity.
    pub addr: usize,
    /// `myType`.
    pub kind: MeshVSEntityType,
    /// `myID`.
    pub id: i32,
    /// `myIsGroup`.
    pub is_group: bool,
}

/// The owner of a mesh entity; `MeshVSMeshEntityOwner::new(&mesh, 3, 0, MESHVS_ET_FACE, 2, false)`.
pub struct MeshVSMeshEntityOwner {
    transient: StandardTransient,
    base: SelectMgrEntityOwnerBase,
    fields: MeshVSMeshEntityOwnerFields,
}
rustycad_tkernel::impl_transient!(MeshVSMeshEntityOwner, "MeshVS_MeshEntityOwner", dyn SelectMgrEntityOwnerSource, transient,
    interfaces: [dyn SelectMgrEntityOwnerSource]);

impl MeshVSMeshEntityOwner {
    /// `MeshVS_MeshEntityOwner(SelObj, ID, MeshEntity, Type, Priority, IsGroup)`.
    /// Example: `MeshVSMeshEntityOwner::new(&selectable, 3, 0, MESHVS_ET_NODE, 5, false)`.
    pub fn new(
        selectable: &Handle<dyn SelectMgrSelectableObject>,
        id: i32,
        mesh_entity: usize,
        kind: MeshVSEntityType,
        priority: i32,
        is_group: bool,
    ) -> Handle<Self> {
        let owner = Handle::new(Self {
            transient: StandardTransient::default(),
            base: SelectMgrEntityOwnerBase::new(selectable, priority),
            fields: MeshVSMeshEntityOwnerFields {
                addr: mesh_entity,
                kind,
                id,
                is_group,
            },
        });
        select_mgr_entity_owner_bind(&owner.clone().cast());
        owner
    }
    /// `Owner`: the opaque entity address; `owner.owner()`.
    pub fn owner(&self) -> usize {
        self.fields.addr
    }
    /// `Type`; `owner.kind()`.
    pub fn kind(&self) -> MeshVSEntityType {
        self.fields.kind
    }
    /// `ID`; `owner.id()`.
    pub fn id(&self) -> i32 {
        self.fields.id
    }
    /// `IsGroup`; `owner.is_group()`.
    pub fn is_group(&self) -> bool {
        self.fields.is_group
    }
}

impl SelectMgrEntityOwnerSource for MeshVSMeshEntityOwner {
    fn owner_base(&self) -> &SelectMgrEntityOwnerBase {
        &self.base
    }
    /// `IsHilighted`: always false.
    fn is_hilighted(&self, _: &Handle<PrsMgrPresentationManager>, _: i32) -> bool {
        false
    }
    /// `HilightWithColor`: in immediate mode, the mesh highlights this owner.
    fn hilight_with_color(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        style: &Handle<Prs3dDrawer>,
        _: i32,
    ) -> Result<(), Failure> {
        hilight_by_mesh(self, manager, style)
    }
    /// `Unhilight`: nothing.
    fn unhilight(&self, _: &Handle<PrsMgrPresentationManager>, _: i32) {}
    /// `Clear`: nothing.
    fn clear(&self, _: &Handle<PrsMgrPresentationManager>, _: i32) {}
}

/// The immediate-mode delegation of the MeshVS owners to `MeshVS_Mesh::HilightOwnerWithColor`;
/// a selectable object that is not a mesh is ignored (natively, a null one is dereferenced).
pub(crate) fn hilight_by_mesh<O: SelectMgrEntityOwnerSource + ?Sized>(
    owner: &O,
    manager: &Handle<PrsMgrPresentationManager>,
    style: &Handle<Prs3dDrawer>,
) -> Result<(), Failure> {
    let selectable = if owner.has_selectable() {
        owner.selectable()
    } else {
        Handle::default()
    };
    let immediate = manager.get().is_some_and(|m| m.is_immediate_mode_on());
    let mesh = selectable.cast::<dyn MeshVSMeshSource>();
    match mesh.get() {
        Some(mesh) if immediate => {
            mesh.hilight_owner_with_color(manager, style, &owner.owner_base().this())
        }
        _ => Ok(()),
    }
}
