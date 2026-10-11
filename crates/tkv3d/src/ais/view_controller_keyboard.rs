//! Keyboard forwarding and event-time bookkeeping at OCCT 3d097a0.
use super::AisViewController;
use rustycad_tkernel::standard::Failure;
use rustycad_tkservice::aspect::{AspectVKey, AspectVKeySet, AspectWindowInputListenerBase};
impl AisViewController {
    /// Borrow listener state for application hooks; `controller.input_base().keys.modifiers()`.
    pub fn input_base(&self) -> &AspectWindowInputListenerBase {
        &self.input_base
    }
    /// Mutate listener state for application hooks; `controller.input_base_mut().event_timer.reset_to(2.0)`.
    pub fn input_base_mut(&mut self) -> &mut AspectWindowInputListenerBase {
        &mut self.input_base
    }
    /// Read keyboard state; `controller.keys().is_key_down(key)?`.
    pub fn keys(&self) -> &AspectVKeySet {
        &self.input_base.keys
    }
    /// Change keyboard state without replacing the cache; `controller.change_keys().reset()`.
    pub fn change_keys(&mut self) -> &mut AspectVKeySet {
        &mut self.input_base.keys
    }
    /// Queue a press through the native key cache; `controller.key_down(key, 0.5, 1.0)?`.
    pub fn key_down(&mut self, key: AspectVKey, time: f64, pressure: f64) -> Result<(), Failure> {
        self.input_base.key_down(key, time, pressure)
    }
    /// Queue a release for one final duration read; `controller.key_up(key, 1.0)?`.
    pub fn key_up(&mut self, key: AspectVKey, time: f64) -> Result<(), Failure> {
        self.input_base.key_up(key, time)
    }
    /// Map signed axis pressure to an opposing key pair; `controller.key_from_axis(left, right, 0.5, -0.25)?`.
    pub fn key_from_axis(
        &mut self,
        negative: AspectVKey,
        positive: AspectVKey,
        time: f64,
        pressure: f64,
    ) -> Result<(), Failure> {
        self.input_base
            .key_from_axis(negative, positive, time, pressure)
    }
    /// Read the injected event timer; `controller.event_time()`.
    pub fn event_time(&self) -> f64 {
        self.input_base.event_timer.elapsed_time()
    }
    /// Advance the native frame interval, returning previous/current seconds; `controller.update_events_time()`.
    pub fn update_events_time(&mut self) -> (f64, f64) {
        let previous = self.navigation_state.last_events_time;
        self.navigation_state.last_events_time = self.event_time();
        (previous, self.navigation_state.last_events_time)
    }
    /// Read whether input has requested another frame; `controller.to_ask_next_frame()`.
    pub fn to_ask_next_frame(&self) -> bool {
        self.navigation_state.to_ask_next_frame
    }
    /// Set the next-frame request explicitly; `controller.set_ask_next_frame(false)`.
    pub fn set_ask_next_frame(&mut self, request: bool) {
        self.navigation_state.to_ask_next_frame = request;
    }
}
