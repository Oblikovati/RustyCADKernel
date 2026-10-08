//! `Select3D_SensitiveSphere` and `Select3D_SensitiveCylinder`: sensitive quadrics.
use super::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityMembers as _,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelectionType};
use rustycad_tkernel::ncollection::{Mat4, Vec3};
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::bvh::{BvhBox, std_max};
use rustycad_tkmath::gp::{Point3, Transform3};

/// A sensitive sphere; the point detected by the last point pick is remembered.
///
/// Example: `let sphere = Select3DSensitiveSphere::new(&owner, center, 2.0); sphere.radius();`.
pub struct Select3DSensitiveSphere {
    transient: StandardTransient,
    base: Select3DSensitiveEntityBase,
    center: Point3,
    last_detected_point: SharedState<Point3>,
    radius: f64,
}
rustycad_tkernel::impl_transient!(Select3DSensitiveSphere, "Select3D_SensitiveSphere", dyn Select3DSensitiveEntity, transient,
    interfaces: [dyn Select3DSensitiveEntity]);

/// The point `(RealLast, RealLast, RealLast)` of an undetected sphere.
fn undetected() -> Point3 {
    Point3::new(real_last(), real_last(), real_last())
}

impl Select3DSensitiveSphere {
    /// A sphere of an owner; `Select3DSensitiveSphere::new(&owner, center, 1.0)`.
    pub fn new(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        center: Point3,
        radius: f64,
    ) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: Select3DSensitiveEntityBase::new(owner),
            center,
            last_detected_point: SharedState::new(undetected()),
            radius,
        }
    }
    /// `Radius`; `sphere.radius()`.
    pub fn radius(&self) -> f64 {
        self.radius
    }
    /// `LastDetectedPoint`; `sphere.last_detected_point()`.
    pub fn last_detected_point(&self) -> Point3 {
        *self.last_detected_point.read()
    }
    /// `ResetLastDetectedPoint`; `sphere.reset_last_detected_point()`.
    pub fn reset_last_detected_point(&self) {
        *self.last_detected_point.write() = undetected();
    }
}

impl Select3DSensitiveEntity for Select3DSensitiveSphere {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        &self.base
    }
    /// `Matches`: a box or polyline volume tests inclusion or overlap without depth; a point
    /// pick remembers the picked point and computes the distance to the center.
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.reset_last_detected_point();
        if manager.get_active_selection_type() != SelectMgrSelectionType::Point {
            if !manager.is_overlap_allowed() {
                let mut is_inside = true;
                return Ok(manager.overlaps_sphere_inside(
                    &self.center,
                    self.radius,
                    Some(&mut is_inside),
                ) && is_inside);
            }
            return Ok(manager.overlaps_sphere_inside(&self.center, self.radius, None));
        }
        if !manager.overlaps_sphere(&self.center, self.radius, pick_result) {
            return Ok(false);
        }
        *self.last_detected_point.write() = *pick_result.picked_point();
        pick_result.set_dist_to_geom_center(manager.dist_to_geometry_center(&self.center));
        Ok(true)
    }
    fn nb_sub_elements(&self) -> i32 {
        1
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        let (c, r) = (self.center, self.radius);
        BvhBox::from_corners(
            Vec3::new(c.x() - r, c.y() - r, c.z() - r),
            Vec3::new(c.x() + r, c.y() + r, c.z() + r),
        )
    }
    fn center_of_geometry(&self) -> Point3 {
        self.center
    }
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        Ok(Handle::new(Self::new(&self.owner_id(), self.center, self.radius)).cast())
    }
    fn to_build_bvh(&self) -> bool {
        false
    }
}

/// A sensitive cylinder (a cone when the radii differ), placed by a transformation.
///
/// Example: `let cylinder = Select3DSensitiveCylinder::new(&owner, 1.0, 1.0, 2.0, trsf, false);`.
pub struct Select3DSensitiveCylinder {
    transient: StandardTransient,
    base: Select3DSensitiveEntityBase,
    trsf: Transform3,
    bottom_radius: f64,
    top_radius: f64,
    height: f64,
    is_hollow: bool,
}
rustycad_tkernel::impl_transient!(Select3DSensitiveCylinder, "Select3D_SensitiveCylinder", dyn Select3DSensitiveEntity, transient,
    interfaces: [dyn Select3DSensitiveEntity]);

impl Select3DSensitiveCylinder {
    /// A cylinder of an owner; `Select3DSensitiveCylinder::new(&owner, 1.0, 0.5, 2.0, trsf, false)`.
    pub fn new(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        bottom_rad: f64,
        top_rad: f64,
        height: f64,
        trsf: Transform3,
        is_hollow: bool,
    ) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: Select3DSensitiveEntityBase::new(owner),
            trsf,
            bottom_radius: bottom_rad,
            top_radius: top_rad,
            height,
            is_hollow,
        }
    }
    /// `Transformation`; `cylinder.transformation()`.
    pub fn transformation(&self) -> Transform3 {
        self.trsf
    }
    /// `TopRadius`; `cylinder.top_radius()`.
    pub fn top_radius(&self) -> f64 {
        self.top_radius
    }
    /// `BottomRadius`; `cylinder.bottom_radius()`.
    pub fn bottom_radius(&self) -> f64 {
        self.bottom_radius
    }
    /// `Height`; `cylinder.height()`.
    pub fn height(&self) -> f64 {
        self.height
    }
    /// `IsHollow`; `cylinder.is_hollow()`.
    pub fn is_hollow(&self) -> bool {
        self.is_hollow
    }
}

impl Select3DSensitiveEntity for Select3DSensitiveCylinder {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        &self.base
    }
    /// `Matches`: as the sphere, without remembering the picked point.
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        let (b, t, h, trsf, hollow) = (
            self.bottom_radius,
            self.top_radius,
            self.height,
            &self.trsf,
            self.is_hollow,
        );
        if manager.get_active_selection_type() != SelectMgrSelectionType::Point {
            if !manager.is_overlap_allowed() {
                let mut is_inside = true;
                return Ok(manager.overlaps_cylinder_inside(
                    b,
                    t,
                    h,
                    trsf,
                    hollow,
                    Some(&mut is_inside),
                ) && is_inside);
            }
            return Ok(manager.overlaps_cylinder_inside(b, t, h, trsf, hollow, None));
        }
        if !manager.overlaps_cylinder(b, t, h, trsf, hollow, pick_result) {
            return Ok(false);
        }
        pick_result
            .set_dist_to_geom_center(manager.dist_to_geometry_center(&self.center_of_geometry()));
        Ok(true)
    }
    fn nb_sub_elements(&self) -> i32 {
        1
    }
    /// `BoundingBox`: the box of the widest radius and the height, transformed.
    fn bounding_box(&self) -> Select3DBndBox3d {
        let max_rad = std_max(self.bottom_radius, self.top_radius);
        let mut matrix = Mat4::<f64>::default();
        self.trsf.get_mat4(&mut matrix);
        let mut bounds = BvhBox::from_corners(
            Vec3::new(-max_rad, -max_rad, 0.0),
            Vec3::new(max_rad, max_rad, self.height),
        );
        bounds.transform(&matrix);
        bounds
    }
    fn center_of_geometry(&self) -> Point3 {
        Point3::new(0.0, 0.0, self.height / 2.0).transformed(self.trsf)
    }
    /// `GetConnected`: a new cylinder of the same owner; natively the copy is never hollow.
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        Ok(Handle::new(Self::new(
            &self.owner_id(),
            self.bottom_radius,
            self.top_radius,
            self.height,
            self.trsf,
            false,
        ))
        .cast())
    }
    fn to_build_bvh(&self) -> bool {
        false
    }
}
