//! `Select3D_SensitivePoint`: a sensitive point (sensitivity 12).
use super::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityMembers as _, select3d_sensitive_entity_dump_json,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::bvh::BvhBox;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::json_dump::{base_class, class_begin};
use std::io;

/// A sensitive point.
///
/// Example: `let point = Select3DSensitivePoint::new(&owner, Point3::new(1.0, 2.0, 3.0));`.
pub struct Select3DSensitivePoint {
    transient: StandardTransient,
    base: Select3DSensitiveEntityBase,
    point: Point3,
}
rustycad_tkernel::impl_transient!(Select3DSensitivePoint, "Select3D_SensitivePoint", dyn Select3DSensitiveEntity, transient,
    interfaces: [dyn Select3DSensitiveEntity]);

impl Select3DSensitivePoint {
    /// A point of an owner, with the sensitivity factor 12; `Select3DSensitivePoint::new(&owner, p)`.
    pub fn new(owner: &Handle<dyn SelectMgrEntityOwnerSource>, point: Point3) -> Self {
        let entity = Self {
            transient: StandardTransient::default(),
            base: Select3DSensitiveEntityBase::new(owner),
            point,
        };
        entity.base.fields.write().sensitivity_factor = 12;
        entity
    }
    /// `Point`; `entity.point()`.
    pub fn point(&self) -> Point3 {
        self.point
    }
}

impl Select3DSensitiveEntity for Select3DSensitivePoint {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        &self.base
    }
    /// `Matches`: the point overlaps; the distance to the center is the depth.
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        if !manager.overlaps_point(&self.point, pick_result)? {
            return Ok(false);
        }
        pick_result.set_dist_to_geom_center(pick_result.depth());
        Ok(true)
    }
    fn nb_sub_elements(&self) -> i32 {
        1
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        let p = Vec3::new(self.point.x(), self.point.y(), self.point.z());
        BvhBox::from_corners(p, p)
    }
    fn center_of_geometry(&self) -> Point3 {
        self.point
    }
    /// `GetConnected`: a new point of the same owner (sensitivity 12).
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        Ok(Handle::new(Self::new(&self.owner_id(), self.point)).cast())
    }
    fn to_build_bvh(&self) -> bool {
        false
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Select3D_SensitivePoint")?;
        base_class(stream, depth, &|s, d| {
            select3d_sensitive_entity_dump_json(self, s, d)
        })
    }
}
