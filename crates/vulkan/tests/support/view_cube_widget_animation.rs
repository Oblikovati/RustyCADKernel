//! Real derived camera atom exercises the widget's dynamic animation transport during GPU captures.
use super::view_cube_widget::CubeWidgetClock;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkv3d::ais::*;
use std::sync::Arc;
/// Capture subclass keeps native timeline storage and delegates its real camera interpolation.
/// Example: `scene.cube.set_view_animation(&CubeWidgetAnimation::new(&clock).cast())`.
pub struct CubeWidgetAnimation {
    transient: StandardTransient,
    base: AisAnimationCameraBase,
    clock: Arc<CubeWidgetClock>,
}
rustycad_tkernel::impl_transient!(CubeWidgetAnimation,"CaptureSeekAnimation",dyn AisAnimationCameraSource,transient,
    interfaces:[dyn AisAnimationCameraSource,dyn AisAnimationSource]);
impl CubeWidgetAnimation {
    /// Own a stopped native half-second camera timeline; `CubeWidgetAnimation::new(&clock)`.
    pub fn new(clock: &Arc<CubeWidgetClock>) -> Handle<Self> {
        let animation = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisAnimationCameraBase::new("AIS_ViewCube", &Handle::default()),
            clock: clock.clone(),
        });
        animation.set_own_duration(0.5);
        animation
    }
}
impl AisAnimationCameraSource for CubeWidgetAnimation {
    fn camera_animation_base(&self) -> &AisAnimationCameraBase {
        &self.base
    }
}
impl AisAnimationSource for CubeWidgetAnimation {
    fn animation_base(&self) -> &AisAnimationBase {
        &self.base.animation
    }
    fn start_timer(&self, pts: f64, speed: f64, update: bool, _: bool) -> Result<(), Failure> {
        self.clock.record_dispatch(0);
        ais_animation_start_timer(self, pts, speed, update, true)
    }
    fn update_timer(&self) -> Result<f64, Failure> {
        self.clock.record_dispatch(1);
        let timer = self.timer();
        timer.stop();
        timer.seek(rustycad_tkernel::osd::WallClock::now(&*self.clock));
        ais_animation_update_timer(self)
    }
    fn update_own(&self, progress: &AisAnimationProgress) -> Result<(), Failure> {
        self.clock.record_dispatch(2);
        ais_animation_camera_update(self, progress)
    }
}
