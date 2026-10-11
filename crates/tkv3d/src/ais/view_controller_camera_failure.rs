//! Named failing child proves native partial animation stop precedes camera mutation.
use super::view_controller_camera_fixture::PrimitiveScene;
use super::{AisAnimationBase, AisAnimationMembers as _, AisAnimationSource};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
pub(super) struct PrimitiveStopFailure {
    transient: StandardTransient,
    base: AisAnimationBase,
}
rustycad_tkernel::impl_transient!(PrimitiveStopFailure,"PrimitiveStopFailure",dyn AisAnimationSource,transient,interfaces:[dyn AisAnimationSource]);
impl PrimitiveStopFailure {
    pub(super) fn new() -> Self {
        let transient = StandardTransient::default();
        Self {
            transient,
            base: AisAnimationBase::new("primitive stop failure"),
        }
    }
}
impl AisAnimationSource for PrimitiveStopFailure {
    fn animation_base(&self) -> &AisAnimationBase {
        // This is an actual child timeline, so parent start/stop state remains observable.
        &self.base
    }
    fn stop(&self) -> Result<(), Failure> {
        // Rejection occurs after parent state changes and before camera dispatch.
        Err(Failure::new(
            "primitive child stop rejected; expected successful stop",
        ))
    }
}
pub(super) fn attach_primitive_stop_failure(scene: &PrimitiveScene) {
    let child = Handle::new(PrimitiveStopFailure::new());
    let child: Handle<dyn AisAnimationSource> = child.cast();
    scene.controller.view_animation().add(&child).unwrap();
    scene.controller.view_animation().start(false).unwrap();
}
