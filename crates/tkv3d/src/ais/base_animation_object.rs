//! Martensite's object animation base preserves OCCT 3d097a0 context/view update policy.
use super::{AisAnimationBase, AisAnimationSource, AisInteractiveContext, AisInteractiveObject};
use rustycad_tkernel::{
    standard::{Handle, SharedState, StandardTransient},
    tcollection::AsciiString,
};
/// Native protected handles, retained independently of context membership; `base.fields.read().object`.
pub struct AisBaseAnimationObjectFields {
    /// Context is optional for detached objects.
    pub context: Handle<AisInteractiveContext>,
    /// Object receiving exact local transformations.
    pub object: Handle<dyn AisInteractiveObject>,
}
/// Storage embedded by derived object animations; `AisBaseAnimationObjectBase::new("move", &context, &object)`.
pub struct AisBaseAnimationObjectBase {
    /// Inherited timeline state.
    pub animation: AisAnimationBase,
    /// Native protected context and object handles.
    pub fields: SharedState<AisBaseAnimationObjectFields>,
}
impl AisBaseAnimationObjectBase {
    /// Retain the target handles and initialize a stopped timeline; `AisBaseAnimationObjectBase::new("move", &context, &object)`.
    pub fn new(
        name: impl Into<AsciiString>,
        context: &Handle<AisInteractiveContext>,
        object: &Handle<dyn AisInteractiveObject>,
    ) -> Self {
        Self {
            animation: AisAnimationBase::new(name),
            fields: SharedState::new(AisBaseAnimationObjectFields {
                context: context.clone(),
                object: object.clone(),
            }),
        }
    }
}
/// Derived atoms expose their native protected base; `animation.object_animation_base()`.
pub trait AisBaseAnimationObjectSource: AisAnimationSource {
    /// Embedded object/timeline state; `animation.object_animation_base().fields.read()`.
    fn object_animation_base(&self) -> &AisBaseAnimationObjectBase;
}
rustycad_tkernel::impl_transient_interface!(
    dyn AisBaseAnimationObjectSource,
    "AIS_BaseAnimationObject",
    dyn AisAnimationSource
);
/// Native base class with the inherited no-op frame hook; `AisBaseAnimationObject::new("move", &context, &object)`.
pub struct AisBaseAnimationObject {
    transient: StandardTransient,
    base: AisBaseAnimationObjectBase,
}
rustycad_tkernel::impl_transient!(AisBaseAnimationObject,"AIS_BaseAnimationObject",dyn AisAnimationSource,transient,
    interfaces:[dyn AisAnimationSource,dyn AisBaseAnimationObjectSource]);
impl AisBaseAnimationObject {
    /// Expose native protected construction for derived atoms; `AisBaseAnimationObject::new("move", &context, &object)`.
    pub fn new(
        name: impl Into<AsciiString>,
        context: &Handle<AisInteractiveContext>,
        object: &Handle<dyn AisInteractiveObject>,
    ) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: AisBaseAnimationObjectBase::new(name, context, object),
        }
    }
}
impl AisAnimationSource for AisBaseAnimationObject {
    fn animation_base(&self) -> &AisAnimationBase {
        &self.base.animation
    }
}
impl AisBaseAnimationObjectSource for AisBaseAnimationObject {
    fn object_animation_base(&self) -> &AisBaseAnimationObjectBase {
        &self.base
    }
}
