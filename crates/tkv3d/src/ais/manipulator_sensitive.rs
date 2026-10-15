//! The rotation ring sensitive of `AIS_Manipulator` (the anonymous `ManipSensCircle`): a
//! circle boundary detected only by point picking along a ray not lying in its plane.
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelectionType};
use crate::select3d::Select3DSensitiveEntityBase;
use crate::select3d::{Select3DBndBox3d, Select3DSensitiveCircle, Select3DSensitiveEntity};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Circle3, Direction3, Point3};

/// `ManipSensRotation::myAngleTol`: 10 degrees.
const ANGLE_TOL: f64 = 10.0 * std::f64::consts::PI / 180.0;

/// The rotation ring sensitive: a `Select3D_SensitiveCircle` boundary checking the view ray
/// first; `AisManipulatorSensCircle::new(&owner, &circle)`.
pub struct AisManipulatorSensCircle {
    circle: Select3DSensitiveCircle,
    plane_normal: Direction3,
}
// The native class has no RTTI of its own: it reports `Select3D_SensitiveCircle`, which it
// embeds; this descriptor only serves the interface casts of the override.
rustycad_tkernel::impl_transient!(AisManipulatorSensCircle, "AIS_Manipulator_ManipSensCircle",
    Select3DSensitiveCircle, circle, interfaces: [dyn Select3DSensitiveEntity]);
impl AisManipulatorSensCircle {
    /// The boundary of a circle; `AisManipulatorSensCircle::new(&owner, &circle)`.
    pub fn new(owner: &Handle<dyn SelectMgrEntityOwnerSource>, circle: &Circle3) -> Self {
        Self {
            circle: Select3DSensitiveCircle::new(owner, circle, false),
            plane_normal: circle.position().direction(),
        }
    }
    /// `isValidRay`: point picking with a ray not within 10 degrees of the circle plane.
    /// Example: `sensitive.is_valid_ray(&manager)?`.
    pub fn is_valid_ray(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
    ) -> Result<bool, Failure> {
        if manager.get_active_selection_type() != SelectMgrSelectionType::Point {
            return Ok(false);
        }
        let ray = manager.get_view_ray_direction()?;
        Ok(!ray.is_normal(self.plane_normal, ANGLE_TOL))
    }
}
impl Select3DSensitiveEntity for AisManipulatorSensCircle {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        self.circle.sensitive_base()
    }
    /// `Matches`: the circle, for a valid ray only.
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        Ok(self.is_valid_ray(manager)? && self.circle.matches(manager, pick_result)?)
    }
    fn nb_sub_elements(&self) -> i32 {
        self.circle.nb_sub_elements()
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        self.circle.bounding_box()
    }
    fn center_of_geometry(&self) -> Point3 {
        self.circle.center_of_geometry()
    }
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        // Not overridden natively: a plain circle.
        self.circle.get_connected()
    }
    fn to_build_bvh(&self) -> bool {
        self.circle.to_build_bvh()
    }
}
