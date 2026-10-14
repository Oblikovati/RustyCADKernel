//! `AIS_AnimationAxisRotation`: rotates an object about an axis between two angles.
use super::{
    AisAnimationBase, AisAnimationProgress, AisAnimationSource, AisBaseAnimationObjectBase,
    AisBaseAnimationObjectSource, AisInteractiveContext, AisInteractiveObject,
    ais_base_animation_object_update_transform,
};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::{Axis1, Transform3};

/// The rotation animation of an object; the angle interpolates linearly with the local
/// normalized progress. Example: `AisAnimationAxisRotation::new("spin", &context, &object, axis, 0., PI)`.
pub struct AisAnimationAxisRotation {
    transient: StandardTransient,
    base: AisBaseAnimationObjectBase,
    rot_axis: Axis1,
    angle_start: f64,
    angle_end: f64,
}
rustycad_tkernel::impl_transient!(AisAnimationAxisRotation, "AIS_AnimationAxisRotation", dyn AisBaseAnimationObjectSource, transient,
    interfaces: [dyn AisAnimationSource, dyn AisBaseAnimationObjectSource]);

impl AisAnimationAxisRotation {
    /// The animation of an object of a context about an axis from a start to an end angle.
    /// Example: `AisAnimationAxisRotation::new("spin", &context, &object, axis, 0., PI)`.
    pub fn new(
        name: impl Into<AsciiString>,
        context: &Handle<AisInteractiveContext>,
        object: &Handle<dyn AisInteractiveObject>,
        axis: Axis1,
        angle_start: f64,
        angle_end: f64,
    ) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: AisBaseAnimationObjectBase::new(name, context, object),
            rot_axis: axis,
            angle_start,
            angle_end,
        }
    }
    /// `myRotAxis`; `animation.rot_axis()`.
    pub fn rot_axis(&self) -> Axis1 {
        self.rot_axis
    }
    /// `myAngleStart` and `myAngleEnd`; `animation.angles()`.
    pub fn angles(&self) -> (f64, f64) {
        (self.angle_start, self.angle_end)
    }
}

impl AisAnimationSource for AisAnimationAxisRotation {
    fn animation_base(&self) -> &AisAnimationBase {
        &self.base.animation
    }
    /// `update`: the rotation by `(1 - t) start + t end` about the axis.
    fn update_own(&self, progress: &AisAnimationProgress) -> Result<(), Failure> {
        if self.base.fields.read().object.is_null() {
            return Ok(());
        }
        let t = progress.local_normalized;
        let angle = (1.0 - t) * self.angle_start + t * self.angle_end;
        let mut transform = Transform3::default();
        transform.set_rotation(self.rot_axis, angle)?;
        ais_base_animation_object_update_transform(self, &transform)
    }
}

impl AisBaseAnimationObjectSource for AisAnimationAxisRotation {
    fn object_animation_base(&self) -> &AisBaseAnimationObjectBase {
        &self.base
    }
}
