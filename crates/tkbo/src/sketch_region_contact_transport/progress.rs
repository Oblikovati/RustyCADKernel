// SPDX-License-Identifier: GPL-2.0-only
//! A cancellation remains latched while terminal typed evidence is retained.
use super::ContactTransportPhase;

pub(super) struct ContactProgress<'a> {
    callback: &'a mut dyn FnMut(ContactTransportPhase) -> bool,
    stopped: Option<ContactTransportPhase>,
}
impl<'a> ContactProgress<'a> {
    pub(super) fn new(callback: &'a mut dyn FnMut(ContactTransportPhase) -> bool) -> Self {
        Self {
            callback,
            stopped: None,
        }
    }
    pub(super) fn poll(&mut self, phase: ContactTransportPhase) -> bool {
        if self.stopped.is_some() {
            return true;
        }
        if (self.callback)(phase) {
            self.stopped = Some(phase);
        }
        self.stopped.is_some()
    }
    pub(super) fn stopped(&self) -> Option<ContactTransportPhase> {
        // WHY: inspecting a latched stop does not call an externally mutable progress body again.
        self.stopped
    }
}
