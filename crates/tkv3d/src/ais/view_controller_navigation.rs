//! Native walking fetch preserves check order, pressure and duration bookkeeping.
use super::view_controller_navigation_axes::{horizontal_keys, rotation_keys, vertical_keys};
use super::{AisViewController, AisWalkDelta, AisWalkTranslation};
use rustycad_tkernel::standard::Failure;
use rustycad_tkservice::aspect::{AspectVKey, AspectVKeyBasic as V};
impl AisViewController {
    /// Consume held/released navigation keys for this frame; `controller.fetch_navigation_keys(0.5, 2.0)?`.
    pub fn fetch_navigation_keys(
        &mut self,
        crouch_ratio: f64,
        run_ratio: f64,
    ) -> Result<AisWalkDelta, Failure> {
        let (previous, current) = self.update_events_time();
        let mut walk = AisWalkDelta::new();
        self.apply_navigation_thrust(&mut walk, current - previous);
        self.apply_navigation_stance(&mut walk, current, crouch_ratio, run_ratio)?;
        let ratio = navigation_stance_ratio(&walk, crouch_ratio, run_ratio);
        let interval = current - previous;
        self.apply_navigation_axes(&mut walk, current, interval, ratio)?;
        Ok(walk)
    }
    fn apply_navigation_axes(
        &self,
        walk: &mut AisWalkDelta,
        now: f64,
        interval: f64,
        ratio: f64,
    ) -> Result<(), Failure> {
        horizontal_keys(self.keys(), walk, now, interval, ratio)?;
        rotation_keys(self.keys(), walk, now, interval, self.to_invert_pitch())?;
        vertical_keys(self.keys(), walk, now, interval)
    }
    fn apply_navigation_thrust(&mut self, walk: &mut AisWalkDelta, duration: f64) {
        let speed = self.thrust_speed() as f64;
        if speed.abs() <= rustycad_tkmath::gp::resolution() || speed.is_nan() {
            self.navigation_state.has_thrust = false;
            return;
        }
        if self.navigation_state.has_thrust {
            walk[AisWalkTranslation::Forward].value = speed * duration;
        }
        self.navigation_state.has_thrust = true;
        self.set_ask_next_frame(true);
    }
    fn apply_navigation_stance(
        &self,
        walk: &mut AisWalkDelta,
        current: f64,
        crouch_ratio: f64,
        run_ratio: f64,
    ) -> Result<(), Failure> {
        walk.set_running(run_ratio > 1.0 && self.keys().is_key_down(V::Shift as AspectVKey)?);
        self.consume_navigation_jump(walk, current)?;
        self.apply_navigation_crouch(walk, current, crouch_ratio)
    }
    fn apply_navigation_crouch(
        &self,
        walk: &mut AisWalkDelta,
        current: f64,
        ratio: f64,
    ) -> Result<(), Failure> {
        let allowed = !walk.is_jumping() && ratio < 1.0;
        let key = V::NavCrouch as AspectVKey;
        if allowed && self.keys().hold_duration(key, current)?.0 {
            walk.set_defined(true);
            walk.set_running(false);
            walk.set_crouching(true);
        }
        Ok(())
    }
    fn consume_navigation_jump(
        &self,
        walk: &mut AisWalkDelta,
        current: f64,
    ) -> Result<(), Failure> {
        let key = V::NavJump as AspectVKey;
        if self.keys().hold_duration(key, current)?.0 {
            self.keys().key_up(key, current)?;
            walk.set_defined(true);
            walk.set_jumping(true);
        }
        Ok(())
    }
}

fn navigation_stance_ratio(walk: &AisWalkDelta, crouch: f64, run: f64) -> f64 {
    if walk.is_running() {
        return run;
    }
    if walk.is_crouching() {
        return crouch;
    }
    1.0
}
