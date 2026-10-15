// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_SensitiveMesh` at OCCT 3d097a0: the sensitive entity of a mesh with advanced
//! selection. It never matches by itself (the data source detects the entities); it only
//! carries the selection mode and the bounding box of the data source.
use crate::data_source::MeshVSDataSource;
use crate::mesh_owner::MeshVSMeshOwnerSource;
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::gp::Point3;
use rustycad_tkv3d::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use rustycad_tkv3d::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkv3d::select3d::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityMembers,
};

/// The sensitive entity of a mesh owner; `MeshVSSensitiveMesh::new(&owner, 0)?`.
pub struct MeshVSSensitiveMesh {
    transient: StandardTransient,
    base: Select3DSensitiveEntityBase,
    mode: i32,
    bnd_box: Select3DBndBox3d,
}
rustycad_tkernel::impl_transient!(MeshVSSensitiveMesh, "MeshVS_SensitiveMesh", dyn Select3DSensitiveEntity, transient,
    interfaces: [dyn Select3DSensitiveEntity]);

impl MeshVSSensitiveMesh {
    /// `MeshVS_SensitiveMesh(theOwner, theMode)`: the box of the data source of a mesh owner;
    /// a data source without node raises the `Standard_ConstructionError` of `Bnd_Box::Get`.
    pub fn new(owner: &Handle<dyn SelectMgrEntityOwnerSource>, mode: i32) -> Result<Self, Failure> {
        Ok(Self {
            transient: StandardTransient::default(),
            base: Select3DSensitiveEntityBase::new(owner),
            mode,
            bnd_box: source_box(&data_source_of(owner))?,
        })
    }
    /// `GetMode`; `entity.get_mode()`.
    pub fn get_mode(&self) -> i32 {
        self.mode
    }
}

fn data_source_of(owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> Handle<dyn MeshVSDataSource> {
    let owner = owner.cast::<dyn MeshVSMeshOwnerSource>();
    owner.get().map(|o| o.get_data_source()).unwrap_or_default()
}

fn source_box(source: &Handle<dyn MeshVSDataSource>) -> Result<Select3DBndBox3d, Failure> {
    let Some(source) = source.get() else {
        return Ok(Select3DBndBox3d::default());
    };
    let [x0, y0, z0, x1, y1, z1] = source.get_bounding_box().get()?;
    Ok(Select3DBndBox3d::from_corners(
        Vec3::new(x0, y0, z0),
        Vec3::new(x1, y1, z1),
    ))
}

impl Select3DSensitiveEntity for MeshVSSensitiveMesh {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        &self.base
    }
    /// `Matches`: never.
    fn matches(
        &self,
        _: &dyn SelectBasicsSelectingVolumeManager,
        _: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        Ok(false)
    }
    /// `NbSubElements`: the node count of the owner's data source, -1 without.
    fn nb_sub_elements(&self) -> i32 {
        let source = data_source_of(&self.owner_id());
        source.get().map_or(-1, |s| s.get_all_nodes().extent())
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        self.bnd_box
    }
    /// `CenterOfGeometry`: the box center, the origin for an invalid box.
    fn center_of_geometry(&self) -> Point3 {
        if !self.bnd_box.is_valid() {
            return Point3::new(0.0, 0.0, 0.0);
        }
        let center = (*self.bnd_box.corner_max() + *self.bnd_box.corner_min()) * 0.5;
        Point3::new(center.x, center.y, center.z)
    }
    /// `GetConnected`: a mesh entity of the owner with the default mode 0.
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        Ok(Handle::new(Self::new(&self.owner_id(), 0)?).cast())
    }
}
