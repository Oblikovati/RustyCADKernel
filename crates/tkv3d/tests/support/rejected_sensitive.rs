//! A named fake exposing the native runtime-type rejection branch.
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::gp::Point3;
use rustycad_tkv3d::{
    select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager},
    select_mgr::SelectMgrEntityOwnerSource,
    select3d::{
        Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
        Select3DSensitivePoint,
    },
};
pub struct RejectedSensitivePoint {
    transient: StandardTransient,
    point: Select3DSensitivePoint,
}
rustycad_tkernel::impl_transient!(RejectedSensitivePoint,"RejectedSensitivePoint",StandardTransient,transient,
    interfaces:[dyn Select3DSensitiveEntity]);
impl RejectedSensitivePoint {
    /// Exercise rejection despite implementing the sensitive interface.
    /// Example: `RejectedSensitivePoint::new(&owner, Point3::origin())`.
    pub fn new(owner: &Handle<dyn SelectMgrEntityOwnerSource>, point: Point3) -> Self {
        Self {
            transient: StandardTransient::default(),
            point: Select3DSensitivePoint::new(owner, point),
        }
    }
}
impl Select3DSensitiveEntity for RejectedSensitivePoint {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        self.point.sensitive_base()
    }
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.point.matches(manager, pick)
    }
    fn nb_sub_elements(&self) -> i32 {
        self.point.nb_sub_elements()
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        self.point.bounding_box()
    }
    fn center_of_geometry(&self) -> Point3 {
        self.point.center_of_geometry()
    }
}
