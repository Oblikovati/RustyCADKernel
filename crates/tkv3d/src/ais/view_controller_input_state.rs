//! Native controller-owned bookkeeping, kept separate from gesture buffers.
use super::AisMouseGesture;
use rustycad_tkernel::osd::Timer;
#[derive(Default)]
pub(super) struct ViewControllerNavigationState {
    pub(super) last_events_time: f64,
    pub(super) has_thrust: bool,
    pub(super) to_ask_next_frame: bool,
}
pub(super) struct ViewControllerClickState {
    pub(super) timer: Timer,
    pub(super) counter: i32,
    pub(super) single_button: i32,
    pub(super) active_gesture: AisMouseGesture,
}
impl ViewControllerClickState {
    pub(super) fn with_timer(mut timer: Timer) -> Self {
        timer.reset();
        Self {
            timer,
            counter: 0,
            single_button: -1,
            active_gesture: AisMouseGesture::None,
        }
    }
}
