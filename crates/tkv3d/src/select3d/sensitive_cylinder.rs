//! Martensite sensitive cylinders/cones retain hollow and transformed selection policies.
use super::quadric_selection::{AnalyticQuadricSelection, match_analytic_quadric};
use super::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityMembers as _,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkernel::ncollection::{Mat4, Vec3};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::bvh::{BvhBox, std_max};
use rustycad_tkmath::gp::{Point3, Transform3};
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
        // Return the stored placement without recomputing geometry.
        self.trsf
    }
    /// `TopRadius`; `cylinder.top_radius()`.
    pub fn top_radius(&self) -> f64 {
        // The cone cap radius stays distinct from its bottom radius.
        self.top_radius
    }
    /// `BottomRadius`; `cylinder.bottom_radius()`.
    pub fn bottom_radius(&self) -> f64 {
        // The bottom cap remains anchored at local z = 0.
        self.bottom_radius
    }
    /// `Height`; `cylinder.height()`.
    pub fn height(&self) -> f64 {
        // Height remains the native local axis extent.
        self.height
    }
    /// `IsHollow`; `cylinder.is_hollow()`.
    pub fn is_hollow(&self) -> bool {
        // Hollow policy is retained by this original entity.
        self.is_hollow
    }
}

impl Select3DSensitiveEntity for Select3DSensitiveCylinder {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        // Ownership remains shared by the native connected-copy policy.
        &self.base
    }
    /// `Matches`: as the sphere, without remembering the picked point.
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
        // Selection distance uses the analytic center without tessellation averaging.
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
        // Analytic selection is direct and needs no primitive BVH.
        false
    }
}

impl AnalyticQuadricSelection for Select3DSensitiveCylinder {
    fn quadric_point_overlap(
        &self,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        // Forward stored analytic geometry without tessellating it for point selection.
        volume.overlaps_cylinder(
            self.bottom_radius,
            self.top_radius,
            self.height,
            &self.trsf,
            self.is_hollow,
            pick,
        )
    }
    fn quadric_inside_overlap(
        &self,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        // Preserve the caller's inclusion flag: native volume families update it differently.
        volume.overlaps_cylinder_inside(
            self.bottom_radius,
            self.top_radius,
            self.height,
            &self.trsf,
            self.is_hollow,
            inside,
        )
    }
}
