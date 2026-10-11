//! Martensite AIS_AnimationObject replaces local transforms with native part-wise NLerp.
use super::{
    AisAnimationBase, AisAnimationProgress, AisAnimationSource, AisBaseAnimationObjectBase,
    AisBaseAnimationObjectSource, AisInteractiveContext, AisInteractiveObject,
    ais_base_animation_object_update_transform,
};
use rustycad_tkernel::{
    standard::{Failure, Handle, StandardTransient},
    tcollection::AsciiString,
};
use rustycad_tkmath::gp::{Transform3, TransformNLerp};
/// Animate an object's exact local placement; `AisAnimationObject::new("move", &context, &object, start, end)`.
pub struct AisAnimationObject {
    transient: StandardTransient,
    base: AisBaseAnimationObjectBase,
    transform_lerp: TransformNLerp,
}
rustycad_tkernel::impl_transient!(AisAnimationObject,"AIS_AnimationObject",dyn AisBaseAnimationObjectSource,transient,
    interfaces:[dyn AisAnimationSource,dyn AisBaseAnimationObjectSource]);
impl AisAnimationObject {
    /// Retain targets and copy exact local endpoints; `AisAnimationObject::new("move", &context, &object, start, end)`.
    pub fn new(
        name: impl Into<AsciiString>,
        context: &Handle<AisInteractiveContext>,
        object: &Handle<dyn AisInteractiveObject>,
        start: Transform3,
        end: Transform3,
    ) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: AisBaseAnimationObjectBase::new(name, context, object),
            transform_lerp: TransformNLerp::new(start, end),
        }
    }
}
impl AisAnimationSource for AisAnimationObject {
    fn animation_base(&self) -> &AisAnimationBase {
        &self.base.animation
    }
    fn update_own(&self, progress: &AisAnimationProgress) -> Result<(), Failure> {
        if self.base.fields.read().object.is_null() {
            return Ok(());
        }
        let transform = self.transform_lerp.interpolate(progress.local_normalized)?;
        ais_base_animation_object_update_transform(self, &transform)
    }
}
impl AisBaseAnimationObjectSource for AisAnimationObject {
    fn object_animation_base(&self) -> &AisBaseAnimationObjectBase {
        &self.base
    }
}
