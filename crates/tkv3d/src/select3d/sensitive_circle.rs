//! Martensite analytic sensitive circle preserves exact native selecting-volume geometry.
use super::quadric_selection::{AnalyticQuadricSelection, match_analytic_quadric};
use super::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityMembers as _, Select3DTypeOfSensitivity,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkernel::ncollection::{Mat4, Vec3};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::bvh::BvhBox;
use rustycad_tkmath::gp::{Axis3, Circle3, Point3, Transform3, xoy};
/// An exact sensitive circle, the circle of radius `radius` in `gp::XOY` placed by `trsf`.
///
/// Example: `let circle = Select3DSensitiveCircle::new(&owner, &circ, false);`.
pub struct Select3DSensitiveCircle {
    transient: StandardTransient,
    base: Select3DSensitiveEntityBase,
    sens_type: Select3DTypeOfSensitivity,
    trsf: Transform3,
    radius: f64,
}
rustycad_tkernel::impl_transient!(Select3DSensitiveCircle, "Select3D_SensitiveCircle", dyn Select3DSensitiveEntity, transient,
    interfaces: [dyn Select3DSensitiveEntity]);

impl Select3DSensitiveCircle {
    /// A circle of an owner, filled or a boundary of sensitivity 6.
    /// Example: `Select3DSensitiveCircle::new(&owner, &circ, false)`.
    pub fn new(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        circle: &Circle3,
        is_filled: bool,
    ) -> Self {
        let mut trsf = Transform3::default();
        trsf.set_transformation_between(
            Axis3::from_axis2(circle.position()),
            Axis3::from_axis2(xoy()),
        );
        Self::positioned_circle(owner, trsf, circle.radius(), is_filled)
    }
    fn positioned_circle(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        trsf: Transform3,
        radius: f64,
        filled: bool,
    ) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: circle_entity_base(owner, filled),
            sens_type: circle_sensitivity(filled),
            trsf,
            radius,
        }
    }
    /// Retains the deprecated native constructor's ignored point-count argument.
    /// Example: `Select3DSensitiveCircle::with_ignored_point_count(&owner, &circle, false, 64)`.
    pub fn with_ignored_point_count(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        circle: &Circle3,
        filled: bool,
        _point_count: i32,
    ) -> Self {
        // OCCT 3d097a0 delegates directly; even negative point counts do not discretize the circle.
        Self::new(owner, circle, filled)
    }
    /// `Transformation`; `circle.transformation()`.
    pub fn transformation(&self) -> Transform3 {
        // Return the stored placement without recomputing geometry.
        self.trsf
    }
    /// `Circle`: the circle of `gp::XOY` transformed; the radius check of `gp_Circ` cannot fail
    /// for a stored radius. Example: `circle.circle()?`.
    pub fn circle(&self) -> Result<Circle3, Failure> {
        // Reconstruct native orientation for connected copies, retaining the stored radius.
        Circle3::new(xoy().transformed(self.trsf)?, self.radius)
    }
    /// `Radius`; `circle.radius()`.
    pub fn radius(&self) -> f64 {
        // The native constructor retains its radius unchanged.
        self.radius
    }
}

impl Select3DSensitiveEntity for Select3DSensitiveCircle {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        // Ownership remains shared by the native connected-copy policy.
        &self.base
    }
    /// `Matches`: as the sphere, with the filled flag of the sensitivity.
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        // Shared dispatch keeps nonpoint outputs and point-hit assignment order native.
        match_analytic_quadric(self, manager, pick_result)
    }
    fn nb_sub_elements(&self) -> i32 {
        // An analytic primitive does not expose tessellation vertices.
        1
    }
    /// `BoundingBox`: the square of the radius in XOY, transformed.
    fn bounding_box(&self) -> Select3DBndBox3d {
        let mut matrix = Mat4::<f64>::default();
        self.trsf.get_mat4(&mut matrix);
        let mut bounds = BvhBox::from_corners(
            Vec3::new(-self.radius, -self.radius, 0.0),
            Vec3::new(self.radius, self.radius, 0.0),
        );
        bounds.transform(&matrix);
        bounds
    }
    fn center_of_geometry(&self) -> Point3 {
        // The native circle placement anchors its analytic center at this translation.
        Point3::from(self.trsf.translation_part())
    }
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        let is_filled = self.sens_type == Select3DTypeOfSensitivity::Interior;
        Ok(Handle::new(Self::new(&self.owner_id(), &self.circle()?, is_filled)).cast())
    }
    fn to_build_bvh(&self) -> bool {
        // Analytic selection is direct and needs no primitive BVH.
        false
    }
}

impl AnalyticQuadricSelection for Select3DSensitiveCircle {
    fn quadric_point_overlap(
        &self,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        // Forward stored analytic geometry without tessellating it for point selection.
        volume.overlaps_circle(
            self.radius,
            &self.trsf,
            self.sens_type == Select3DTypeOfSensitivity::Interior,
            pick,
        )
    }
    fn quadric_inside_overlap(
        &self,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        // Preserve the caller's inclusion flag: native volume families update it differently.
        volume.overlaps_circle_inside(
            self.radius,
            &self.trsf,
            self.sens_type == Select3DTypeOfSensitivity::Interior,
            inside,
        )
    }
}

fn circle_entity_base(
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    filled: bool,
) -> Select3DSensitiveEntityBase {
    let base = Select3DSensitiveEntityBase::new(owner);
    if !filled {
        base.fields.write().sensitivity_factor = 6;
    }
    base
}
fn circle_sensitivity(filled: bool) -> Select3DTypeOfSensitivity {
    if filled {
        return Select3DTypeOfSensitivity::Interior;
    }
    Select3DTypeOfSensitivity::Boundary
}
