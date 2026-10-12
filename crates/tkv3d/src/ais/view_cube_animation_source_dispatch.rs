//! Native override bodies delegate real interpolation and mutate the live widget slot at exact seams.
use super::super::*;
use super::view_cube_animation_source_fixture::*;
use rustycad_tkernel::standard::{Failure, Handle, raise_native_failure};
use std::sync::atomic::Ordering;
impl AisAnimationSource for CubeTransportAnimation {
    fn animation_base(&self) -> &AisAnimationBase {
        &self.base.animation
    }
    fn start_timer(&self, pts: f64, speed: f64, update: bool, _: bool) -> Result<(), Failure> {
        self.trace.record("timer-start", Some(self));
        if self.trace.scenario == 3 {
            return Err(self.trace.reject("start"));
        }
        ais_animation_start_timer(self, pts, speed, update, true)
    }
    fn start(&self, update: bool) -> Result<(), Failure> {
        self.trace.record("start", Some(self));
        if self.trace.scenario == 12 {
            self.trace.replace(1);
        }
        ais_animation_start(self, update)
    }
    fn update(&self, pts: f64) -> Result<bool, Failure> {
        self.trace.record("update", Some(self));
        ais_animation_update(self, pts)
    }
    fn update_timer(&self) -> Result<f64, Failure> {
        self.trace.record("timer-update", Some(self));
        if self.trace.scenario == 4 {
            return Err(self.trace.reject("timer"));
        }
        if matches!(self.trace.scenario, 1 | 2) {
            self.trace.replace(1);
            return Ok(0.6);
        }
        let timer = self.timer();
        timer.stop();
        timer.seek(f64::from_bits(self.next.load(Ordering::SeqCst)));
        ais_animation_update_timer(self)
    }
    fn stop(&self) -> Result<(), Failure> {
        self.trace.record("stop", Some(self));
        if self.trace.scenario == 5 {
            return Err(self.trace.reject("stop"));
        }
        ais_animation_stop(self)?;
        if self.trace.scenario == 2 {
            self.trace.replace(2);
        }
        Ok(())
    }
    fn update_own(&self, progress: &AisAnimationProgress) -> Result<(), Failure> {
        self.trace.record("own", Some(self));
        if self.trace.scenario == 11 {
            self.trace.replace(1);
        }
        ais_animation_camera_update(self, progress)
    }
}
impl AisViewCubeAnimationEvents for CubeTransportTrace {
    fn after_animation(&self) {
        self.record("after", None);
        if self.scenario == 6 {
            raise_native_failure::<()>(Err(self.reject("after")));
        }
        if self.scenario == 9 {
            self.replace(2);
        }
        if self.scenario == 10 {
            self.cube
                .read()
                .upgrade()
                .set_view_animation(&Handle::default());
        }
    }
    fn animation_finished(&self) {
        self.record("finish", None);
        if self.scenario == 7 {
            raise_native_failure::<()>(Err(self.reject("finish")));
        }
        if self.scenario == 8 {
            self.replace(2);
        }
    }
}
