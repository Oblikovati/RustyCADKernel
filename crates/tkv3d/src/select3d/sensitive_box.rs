//! `Select3D_SensitiveBox`: a sensitive axis-aligned box.
use super::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityMembers as _, select3d_sensitive_entity_dump_json,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::bvh::BvhBox;
use rustycad_tkmath::gp::{Point3, Xyz};
use rustycad_tkservice::json_dump::{base_class, class_begin, field_dumped};
use std::io;

/// A sensitive axis-aligned box.
///
/// Example: `let entity = Select3DSensitiveBox::from_bounds(&owner, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0);`.
pub struct Select3DSensitiveBox {
    transient: StandardTransient,
    base: Select3DSensitiveEntityBase,
    bounds: Select3DBndBox3d,
    center3d: Point3,
}
rustycad_tkernel::impl_transient!(Select3DSensitiveBox, "Select3D_SensitiveBox", dyn Select3DSensitiveEntity, transient,
    interfaces: [dyn Select3DSensitiveEntity]);

impl Select3DSensitiveBox {
    /// `Select3D_SensitiveBox(owner, Bnd_Box)`; a void box raises the `Standard_ConstructionError`
    /// of `Bnd_Box::Get`. Example: `Select3DSensitiveBox::new(&owner, &bounds)?`.
    pub fn new(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        bounds: &BoundingBox3,
    ) -> Result<Self, Failure> {
        let [x0, y0, z0, x1, y1, z1] = bounds.get()?;
        Ok(Self::from_bounds(owner, x0, y0, z0, x1, y1, z1))
    }
    /// `Select3D_SensitiveBox(owner, XMin, YMin, ZMin, XMax, YMax, ZMax)`.
    /// Example: `Select3DSensitiveBox::from_bounds(&owner, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0)`.
    pub fn from_bounds(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        x_min: f64,
        y_min: f64,
        z_min: f64,
        x_max: f64,
        y_max: f64,
        z_max: f64,
    ) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: Select3DSensitiveEntityBase::new(owner),
            bounds: BvhBox::from_corners(
                Vec3::new(x_min, y_min, z_min),
                Vec3::new(x_max, y_max, z_max),
            ),
            center3d: Point3::from(
                (Xyz::new(x_min, y_min, z_min) + Xyz::new(x_max, y_max, z_max)) * (1.0 / 2.0),
            ),
        }
    }
    /// `Box`: the box as a `Bnd_Box`; `entity.bnd_box()`.
    pub fn bnd_box(&self) -> BoundingBox3 {
        let (min, max) = (self.bounds.corner_min(), self.bounds.corner_max());
        let mut bounds = BoundingBox3::default();
        bounds.update_bounds([min.x, min.y, min.z], [max.x, max.y, max.z]);
        bounds
    }
}

impl Select3DSensitiveEntity for Select3DSensitiveBox {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        &self.base
    }
    /// `Matches`: fully inside for the inclusion test, else the box overlaps and the distance
    /// to the center is computed.
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        let (min, max) = (self.bounds.corner_min(), self.bounds.corner_max());
        if !manager.is_overlap_allowed() {
            let mut is_inside = true;
            return Ok(manager.overlaps_box_inside(min, max, Some(&mut is_inside))? && is_inside);
        }
        if !manager.overlaps_box(min, max, pick_result)? {
            return Ok(false);
        }
        pick_result.set_dist_to_geom_center(manager.dist_to_geometry_center(&self.center3d)?);
        Ok(true)
    }
    fn nb_sub_elements(&self) -> i32 {
        1
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        self.bounds
    }
    fn center_of_geometry(&self) -> Point3 {
        self.center3d
    }
    /// `GetConnected`: a new box of the same owner, through a `Bnd_Box` (which widens the box
    /// by its gap, zero here).
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        Ok(Handle::new(Self::new(&self.owner_id(), &self.bnd_box())?).cast())
    }
    fn to_build_bvh(&self) -> bool {
        false
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Select3D_SensitiveBox")?;
        base_class(stream, depth, &|s, d| {
            select3d_sensitive_entity_dump_json(self, s, d)
        })?;
        let bounds = self.bounds;
        field_dumped(stream, depth, "&myBox", &|s, _| bounds.dump_json(s))
    }
}
