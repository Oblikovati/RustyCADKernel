//! `Select3D_TypeOfSensitivity` and `Select3D_SensitiveTriangle`.
use super::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityMembers as _, select3d_sensitive_entity_dump_json,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelectionType};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::bvh::{BvhBox, std_max, std_min};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::json_dump::{base_class, class_begin, field_dumped, field_integer};
use std::io;

/// `Select3D_TypeOfSensitivity`: whether the interior or the boundary of a face is sensitive.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Select3DTypeOfSensitivity {
    /// `Select3D_TOS_INTERIOR`.
    Interior = 0,
    /// `Select3D_TOS_BOUNDARY`.
    Boundary = 1,
}

/// The minimum and maximum corners of points with the native `std::min`/`std::max` nesting.
pub(crate) fn points_box(points: &[Point3; 3]) -> Select3DBndBox3d {
    let [a, b, c] = points;
    let min = Vec3::new(
        std_min(a.x(), std_min(b.x(), c.x())),
        std_min(a.y(), std_min(b.y(), c.y())),
        std_min(a.z(), std_min(b.z(), c.z())),
    );
    let max = Vec3::new(
        std_max(a.x(), std_max(b.x(), c.x())),
        std_max(a.y(), std_max(b.y(), c.y())),
        std_max(a.z(), std_max(b.z(), c.z())),
    );
    BvhBox::from_corners(min, max)
}

/// A sensitive triangle.
///
/// Example: `let tri = Select3DSensitiveTriangle::new(&owner, a, b, c, Select3DTypeOfSensitivity::Interior);`.
pub struct Select3DSensitiveTriangle {
    transient: StandardTransient,
    base: Select3DSensitiveEntityBase,
    sens_type: Select3DTypeOfSensitivity,
    centroid: Point3,
    points: [Point3; 3],
}
rustycad_tkernel::impl_transient!(Select3DSensitiveTriangle, "Select3D_SensitiveTriangle", dyn Select3DSensitiveEntity, transient,
    interfaces: [dyn Select3DSensitiveEntity]);

impl Select3DSensitiveTriangle {
    /// A triangle of an owner; `Select3DSensitiveTriangle::new(&owner, a, b, c, Select3DTypeOfSensitivity::Interior)`.
    pub fn new(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        p0: Point3,
        p1: Point3,
        p2: Point3,
        sens_type: Select3DTypeOfSensitivity,
    ) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: Select3DSensitiveEntityBase::new(owner),
            sens_type,
            centroid: Point3::from((p0.xyz() + p1.xyz() + p2.xyz()) * (1.0 / 3.0)),
            points: [p0, p1, p2],
        }
    }
    /// `Points3D`; `triangle.points_3d()`.
    pub fn points_3d(&self) -> (Point3, Point3, Point3) {
        (self.points[0], self.points[1], self.points[2])
    }
    /// `Center3D`; `triangle.center_3d()`.
    pub fn center_3d(&self) -> Point3 {
        self.centroid
    }
}

impl Select3DSensitiveEntity for Select3DSensitiveTriangle {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        &self.base
    }
    /// `Matches`: the inclusion test needs the three points inside (the triangle for a
    /// polyline); the overlap test computes the distance to the centroid.
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        let [a, b, c] = &self.points;
        if !manager.is_overlap_allowed() {
            if manager.get_active_selection_type() == SelectMgrSelectionType::Polyline {
                let mut dummy = SelectBasicsPickResult::default();
                return manager.overlaps_triangle(a, b, c, self.sens_type as i32, &mut dummy);
            }
            return Ok(manager.overlaps_point_only(a)?
                && manager.overlaps_point_only(b)?
                && manager.overlaps_point_only(c)?);
        }
        if !manager.overlaps_triangle(a, b, c, self.sens_type as i32, pick_result)? {
            return Ok(false);
        }
        pick_result.set_dist_to_geom_center(manager.dist_to_geometry_center(&self.centroid));
        Ok(true)
    }
    fn nb_sub_elements(&self) -> i32 {
        3
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        points_box(&self.points)
    }
    fn center_of_geometry(&self) -> Point3 {
        self.centroid
    }
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        let [a, b, c] = self.points;
        Ok(Handle::new(Self::new(&self.owner_id(), a, b, c, self.sens_type)).cast())
    }
    fn to_build_bvh(&self) -> bool {
        false
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Select3D_SensitiveTriangle")?;
        base_class(stream, depth, &|s, d| {
            select3d_sensitive_entity_dump_json(self, s, d)
        })?;
        field_integer(stream, "mySensType", self.sens_type as i32)?;
        for (name, point) in ["&myPoints[0]", "&myPoints[1]", "&myPoints[2]"]
            .into_iter()
            .zip(self.points)
        {
            field_dumped(stream, depth, name, &|s, _| point.dump_json(s))?;
        }
        let bounds = self.bounding_box();
        field_dumped(stream, depth, "&aBoundingBox", &|s, _| bounds.dump_json(s))
    }
}
