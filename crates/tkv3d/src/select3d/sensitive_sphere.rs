//! Martensite sensitive spheres retain native detection/reset semantics.
use super::quadric_selection::{AnalyticQuadricSelection, match_analytic_quadric};
use super::sphere_detection_reference::Select3DSphereDetectionRef;
use super::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityMembers as _,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::bvh::BvhBox;
use rustycad_tkmath::gp::Point3;

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
    // OCCT 3d097a0 uses RealLast coordinates instead of an optional detection flag.
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
        // The native constructor retains its radius unchanged.
        self.radius
    }
    /// `LastDetectedPoint`; `sphere.last_detected_point()`.
    pub fn last_detected_point(&self) -> Point3 {
        // The native sentinel remains visible until a point pick succeeds.
        self.last_detected_point_ref().point()
    }
    /// Retains the native const-reference alias while allowing later selection/reset updates.
    /// Example: `let alias = sphere.last_detected_point_ref(); sphere.reset_last_detected_point(); alias.point();`.
    pub fn last_detected_point_ref(&self) -> Select3DSphereDetectionRef<'_> {
        // OCCT 3d097a0 returns its mutable member by const reference, not a frozen point snapshot.
        Select3DSphereDetectionRef::new(&self.last_detected_point)
    }
    /// `ResetLastDetectedPoint`; `sphere.reset_last_detected_point()`.
    pub fn reset_last_detected_point(&self) {
        // Misses and nonpoint matches must discard the previous point.
        *self.last_detected_point.write() = undetected();
    }
}

impl Select3DSensitiveEntity for Select3DSensitiveSphere {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        // Ownership remains shared by the native connected-copy policy.
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
        // Shared dispatch keeps nonpoint outputs and point-hit assignment order native.
        match_analytic_quadric(self, manager, pick_result)
    }
    fn nb_sub_elements(&self) -> i32 {
        // An analytic primitive does not expose tessellation vertices.
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
        // Selection distance uses the analytic center without tessellation averaging.
        self.center
    }
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        // A native connected sphere starts with an undetected sentinel and default sensitivity.
        Ok(Handle::new(Self::new(&self.owner_id(), self.center, self.radius)).cast())
    }
    fn to_build_bvh(&self) -> bool {
        // Analytic selection is direct and needs no primitive BVH.
        false
    }
}

impl AnalyticQuadricSelection for Select3DSensitiveSphere {
    fn quadric_point_overlap(
        &self,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        // Forward stored analytic geometry without tessellating it for point selection.
        volume.overlaps_sphere(&self.center, self.radius, pick)
    }
    fn quadric_inside_overlap(
        &self,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        // Preserve the caller's inclusion flag: native volume families update it differently.
        volume.overlaps_sphere_inside(&self.center, self.radius, inside)
    }
    fn retain_quadric_point(&self, pick: &SelectBasicsPickResult) {
        // Native assignment precedes DistToGeometryCenter, including its checked failures.
        *self.last_detected_point.write() = *pick.picked_point();
    }
}
