//! A sensitive-set fake exposes the scratch pick supplied by shared traversal.
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select3d::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitivePrimitiveArray, Select3DSensitiveSet, Select3DSensitiveSetBase,
};
use rustycad_tkernel::standard::{Failure, StandardTransient};
use rustycad_tkmath::gp::Point3;

pub(super) struct CallbackSensitiveSet {
    transient: StandardTransient,
    primitive: Select3DSensitivePrimitiveArray,
}
rustycad_tkernel::impl_transient!(CallbackSensitiveSet, "Martensite_CallbackSensitiveSet", dyn Select3DSensitiveSet, transient,
    interfaces: [dyn Select3DSensitiveEntity, dyn Select3DSensitiveSet]);
impl CallbackSensitiveSet {
    pub(super) fn new(primitive: Select3DSensitivePrimitiveArray) -> Self {
        Self {
            transient: StandardTransient::default(),
            primitive,
        }
    }
}
// Geometry and lifecycle stay on the real primitive; only element callbacks expose scratch state.
macro_rules! delegate_sensitive_operation {
    ($name:ident($($arg:ident: $ty:ty),* $(,)?) -> $ret:ty) => {
        fn $name(&self, $($arg: $ty),*) -> $ret {
            // Sharing the real primitive preserves its bounds and BVH ordering.
            self.primitive.$name($($arg),*)
        }
    };
}
impl Select3DSensitiveEntity for CallbackSensitiveSet {
    delegate_sensitive_operation!(sensitive_base() -> &Select3DSensitiveEntityBase);
    delegate_sensitive_operation!(
        matches(
            manager: &dyn SelectBasicsSelectingVolumeManager,
            pick: &mut SelectBasicsPickResult,
        ) -> Result<bool, Failure>
    );
    delegate_sensitive_operation!(nb_sub_elements() -> i32);
    delegate_sensitive_operation!(bounding_box() -> Select3DBndBox3d);
    delegate_sensitive_operation!(center_of_geometry() -> Point3);
}
impl Select3DSensitiveSet for CallbackSensitiveSet {
    delegate_sensitive_operation!(set_base() -> &Select3DSensitiveSetBase);
    delegate_sensitive_operation!(size() -> i32);
    delegate_sensitive_operation!(element_box(idx: i32) -> Select3DBndBox3d);
    delegate_sensitive_operation!(center(idx: i32, axis: i32) -> Result<f64, Failure>);
    delegate_sensitive_operation!(
        distance_to_cog(manager: &dyn SelectBasicsSelectingVolumeManager) -> Result<f64, Failure>
    );
    fn swap(&self, first: i32, second: i32) {
        // Building the test tree must retain the real primitive's permutation.
        self.primitive.swap(first, second);
    }
    fn overlaps_element(
        &self,
        pick: &mut SelectBasicsPickResult,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        _idx: i32,
        _inside: bool,
    ) -> Result<bool, Failure> {
        // Unlike primitive arrays, this callback leaves incoming scratch state intact.
        manager.overlaps_point(&Point3::default(), pick)
    }
    fn element_is_inside(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        _idx: i32,
        _inside: bool,
    ) -> Result<bool, Failure> {
        // The fake volume records whether policy changes reached the inclusion callback.
        manager.overlaps_point_only(&Point3::default())
    }
}
