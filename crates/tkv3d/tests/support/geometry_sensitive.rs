//! A real point with an observable native initial-location override.
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::gp::{GeneralTransform3, Point3};
use rustycad_tkv3d::{select_basics::*, select3d::*};
pub struct GeometrySensitivePoint {
    transient: StandardTransient,
    point: Select3DSensitivePoint,
    initial: bool,
    inverse: GeneralTransform3,
}
rustycad_tkernel::impl_transient!(GeometrySensitivePoint,"GeometrySensitivePoint",dyn Select3DSensitiveEntity,transient,interfaces:[dyn Select3DSensitiveEntity]);
impl GeometrySensitivePoint {
    pub fn new(initial: bool, inverse: GeneralTransform3) -> Handle<Self> {
        Handle::new(Self {
            transient: StandardTransient::default(),
            point: Select3DSensitivePoint::new(&Handle::default(), Point3::default()),
            initial,
            inverse,
        })
    }
}
impl Select3DSensitiveEntity for GeometrySensitivePoint {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        self.point.sensitive_base()
    }
    fn matches(
        &self,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.point.matches(volume, pick)
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
    fn has_init_location(&self) -> bool {
        self.initial
    }
    fn inv_init_location(&self) -> GeneralTransform3 {
        self.inverse
    }
}
pub fn geometry_transform(code: i32) -> GeneralTransform3 {
    use rustycad_tkmath::gp::{Matrix3, Transform3, Vector3, Xyz};
    match code {
        0 => GeneralTransform3::default(),
        1 => {
            let mut transform = Transform3::default();
            transform.set_translation(Vector3::new(2., -3., 4.));
            transform.into()
        }
        2 => GeneralTransform3::new(
            Matrix3::new([[2., 0., 0.], [0., 3., 0.], [0., 0., 4.]]),
            Xyz::new(1., 2., 3.),
        ),
        3 => GeneralTransform3::new(
            Matrix3::new([[1., 0.5, 0.], [0., 2., 0.25], [0., 0., 3.]]),
            Xyz::new(-2., 1., 4.),
        ),
        4 => GeneralTransform3::new(
            Matrix3::new([[0., 0., 0.], [0., 0., 0.], [0., 0., 0.]]),
            Xyz::new(1., 2., 3.),
        ),
        _ => unreachable!(),
    }
}
