//! `Message_Algorithm`: execution status with per-status data and message output.
use super::{ExecStatus, Message, Messenger, Msg, Status};
use crate::ncollection::PackedMap;
use crate::standard::{Handle, RuntimeType, RuntimeTyped, StandardTransient};
use crate::tcollection::HExtendedString;
use std::sync::{Arc, Mutex};
mod reporting;
mod strings;

/// One optional entry per status index `FirstStatus..=LastStatus` (index 0 unused).
type StatusSlots<T> = Vec<Option<T>>;

fn status_slots<T>() -> StatusSlots<T> {
    (0..=ExecStatus::LAST_STATUS).map(|_| None).collect()
}

/// Status flags plus integers, strings and custom messages recorded per status;
/// `Algorithm::new()`.
///
/// Derived OCCT algorithms name their messages after their type chain; give the
/// descriptor with [`Algorithm::with_runtime_type`].
pub struct Algorithm {
    runtime_type: &'static RuntimeType,
    status: ExecStatus,
    messenger: Arc<Messenger>,
    report_integers: Option<StatusSlots<PackedMap<i32>>>,
    report_strings: Option<StatusSlots<Vec<Handle<HExtendedString>>>>,
    // `Get()` rewrites a message while sending it from a const method.
    report_messages: Option<StatusSlots<Mutex<Msg>>>,
}
message_runtime_type!(Algorithm, "Message_Algorithm", StandardTransient);
impl Default for Algorithm {
    fn default() -> Self {
        Self::with_runtime_type(Self::static_type())
    }
}
impl Algorithm {
    /// An algorithm sending to the default messenger; `Algorithm::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// An algorithm whose messages are named after `runtime_type` and its ancestors;
    /// `Algorithm::with_runtime_type(Algorithm::static_type())`.
    pub fn with_runtime_type(runtime_type: &'static RuntimeType) -> Self {
        Self {
            runtime_type,
            status: ExecStatus::new(),
            messenger: Message::default_messenger().clone(),
            report_integers: None,
            report_strings: None,
            report_messages: None,
        }
    }
    /// The descriptor naming the messages (`DynamicType`); `algorithm.dynamic_type()`.
    pub fn dynamic_type(&self) -> &'static RuntimeType {
        self.runtime_type
    }
    /// Raises a status flag; `algorithm.set_status(Status::WARN1)`.
    pub fn set_status(&mut self, status: Status) {
        self.status.set(status);
    }
    /// Raises a flag and records an integer for it; `algorithm.set_status_with_integer(Status::FAIL1, 7)`.
    pub fn set_status_with_integer(&mut self, status: Status, value: i32) {
        self.set_status(status);
        let index = ExecStatus::status_index(status) as usize;
        if index == 0 {
            return;
        }
        let slots = self.report_integers.get_or_insert_with(status_slots);
        slots[index]
            .get_or_insert_with(PackedMap::default)
            .add(value);
    }
    /// Raises a flag and stores a custom message for it; `algorithm.set_status_with_msg(s, &msg)`.
    pub fn set_status_with_msg(&mut self, status: Status, message: &Msg) {
        self.set_status(status);
        let index = ExecStatus::status_index(status) as usize;
        if index == 0 {
            return;
        }
        let slots = self.report_messages.get_or_insert_with(status_slots);
        slots[index] = Some(Mutex::new(message.clone()));
    }
    /// The status flags; `algorithm.get_status().is_fail()`.
    pub fn get_status(&self) -> &ExecStatus {
        &self.status
    }
    /// The status flags for editing; `algorithm.change_status().clear()`.
    pub fn change_status(&mut self) -> &mut ExecStatus {
        &mut self.status
    }
    /// Clears the flags and every recorded datum; `algorithm.clear_status()`.
    pub fn clear_status(&mut self) {
        self.status.clear();
        self.report_integers = None;
        self.report_strings = None;
        self.report_messages = None;
    }
    /// Sets the messenger; `None` selects the default one; `algorithm.set_messenger(None)`.
    pub fn set_messenger(&mut self, messenger: Option<Arc<Messenger>>) {
        self.messenger = messenger.unwrap_or_else(|| Message::default_messenger().clone());
    }
    /// The messenger; `algorithm.get_messenger()`.
    pub fn get_messenger(&self) -> Arc<Messenger> {
        self.messenger.clone()
    }
    /// Integers recorded for a status; `algorithm.get_message_numbers(Status::FAIL1)`.
    pub fn get_message_numbers(&self, status: Status) -> Option<&PackedMap<i32>> {
        let index = ExecStatus::status_index(status) as usize;
        self.report_integers
            .as_ref()?
            .get(index)?
            .as_ref()
            .filter(|_| index != 0)
    }
}
