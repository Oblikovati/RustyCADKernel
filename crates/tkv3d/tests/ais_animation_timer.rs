//! Playback lifecycle on an injected wall clock; no sleeps or scheduling dependence.
use rustycad_tkernel::{
    osd::{Chronometer, Timer, WallClock},
    standard::Handle,
};
use rustycad_tkservice::media::MediaTimer;
use rustycad_tkv3d::ais::*;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
struct AnimationWallClock(AtomicU64);
impl AnimationWallClock {
    fn set(&self, seconds: f64) {
        self.0.store(seconds.to_bits(), Ordering::SeqCst);
    }
}
impl WallClock for AnimationWallClock {
    fn now(&self) -> f64 {
        f64::from_bits(self.0.load(Ordering::SeqCst))
    }
}
fn timed_animation() -> (AisAnimation, Arc<AnimationWallClock>) {
    let clock = Arc::new(AnimationWallClock(AtomicU64::new(0)));
    let timer = Timer::with_clocks(Chronometer::default(), clock.clone());
    let animation = AisAnimation::new("timed");
    animation.set_own_duration(10.);
    animation.set_timer(&Handle::new(MediaTimer::with_timer(timer)));
    (animation, clock)
}
#[test]
fn injected_clock_tracks_speed_pause_resume_and_clamped_stop() {
    let (animation, clock) = timed_animation();
    animation.start_timer(1., 2., false, false).unwrap();
    clock.set(2.);
    assert_eq!(animation.update_timer().unwrap(), 5.);
    animation.pause().unwrap();
    clock.set(4.);
    assert_eq!(animation.elapsed_time(), 5.);
    assert!(!animation.timer().is_started());
    animation.start(false).unwrap();
    clock.set(7.);
    assert_eq!(animation.update_timer().unwrap(), 11.);
    assert_eq!(animation.elapsed_time(), 10.);
    assert!(animation.is_stopped() && !animation.timer().is_started());
}
#[test]
fn stopped_timer_seek_retains_requested_position_even_beyond_animation_end() {
    let (animation, clock) = timed_animation();
    animation.start_timer(12., -2., true, true).unwrap();
    clock.set(3.);
    assert_eq!(animation.elapsed_time(), 12.);
    assert_eq!(animation.timer().playback_speed(), -2.);
    assert!(animation.is_stopped() && !animation.timer().is_started());
    animation.start_timer(-3., 0., false, true).unwrap();
    assert!(!animation.is_stopped());
    animation.stop().unwrap();
    assert_eq!(animation.elapsed_time(), -3.);
}
#[test]
fn missing_timer_reports_native_program_error_and_start_timer_creates_one() {
    let animation = AisAnimation::new("timerless");
    assert_eq!(
        animation.update_timer().unwrap_err().exception_type(),
        "Standard_ProgramError"
    );
    animation.set_own_duration(2.);
    animation.start_timer(0.5, 1., false, true).unwrap();
    assert!(!animation.timer().is_null());
    assert_eq!(animation.update_timer().unwrap(), 0.5);
    animation.set_timer(&Handle::default());
    assert_eq!(animation.elapsed_time(), 0.);
}
