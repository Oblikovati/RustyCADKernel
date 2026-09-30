//! `Message_Algorithm`: execution status with per-status data and message output.
use super::{ExecStatus, Message, Messenger, Msg, Status};
use crate::ncollection::PackedMap;
use crate::standard::{
    Handle, RuntimeType, RuntimeTyped, SharedState, StandardTransient, Transient,
};
use crate::tcollection::HExtendedString;
use std::sync::{Arc, Mutex, RwLockWriteGuard};
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
    parent: StandardTransient,
    runtime_type: &'static RuntimeType,
    status: SharedState<ExecStatus>,
    messenger: SharedState<Arc<Messenger>>,
    report_integers: SharedState<Option<StatusSlots<PackedMap<i32>>>>,
    report_strings: SharedState<Option<StatusSlots<Vec<Handle<HExtendedString>>>>>,
    // `Get()` rewrites a message while sending it from a const method.
    report_messages: SharedState<Option<StatusSlots<Arc<Mutex<Msg>>>>>,
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
            parent: StandardTransient::default(),
            runtime_type,
            status: SharedState::new(ExecStatus::new()),
            messenger: SharedState::new(Message::default_messenger().clone()),
            report_integers: SharedState::new(None),
            report_strings: SharedState::new(None),
            report_messages: SharedState::new(None),
        }
    }
    /// The descriptor naming the messages (`DynamicType`); `algorithm.dynamic_type()`.
    pub fn dynamic_type(&self) -> &'static RuntimeType {
        self.runtime_type
    }
    /// Raises a status flag; `algorithm.set_status(Status::WARN1)`.
    pub fn set_status(&self, status: Status) {
        self.status.write().set(status);
    }
    /// Raises a flag and records an integer for it; `algorithm.set_status_with_integer(Status::FAIL1, 7)`.
    pub fn set_status_with_integer(&self, status: Status, value: i32) {
        self.set_status(status);
        let index = ExecStatus::status_index(status) as usize;
        if index == 0 {
            return;
        }
        let mut recorded = self.report_integers.write();
        let slots = recorded.get_or_insert_with(status_slots);
        slots[index]
            .get_or_insert_with(PackedMap::default)
            .add(value);
    }
    /// Raises a flag and stores a custom message for it; `algorithm.set_status_with_msg(s, &msg)`.
    pub fn set_status_with_msg(&self, status: Status, message: &Msg) {
        self.set_status(status);
        let index = ExecStatus::status_index(status) as usize;
        if index == 0 {
            return;
        }
        let mut recorded = self.report_messages.write();
        let slots = recorded.get_or_insert_with(status_slots);
        slots[index] = Some(Arc::new(Mutex::new(message.clone())));
    }
    /// A snapshot of the status flags; `algorithm.get_status().is_fail()`.
    pub fn get_status(&self) -> ExecStatus {
        self.status.snapshot()
    }
    /// The status flags for scoped editing; drop the guard before calling another member; `algorithm.change_status().clear()`.
    pub fn change_status(&self) -> RwLockWriteGuard<'_, ExecStatus> {
        self.status.write()
    }
    /// Clears the flags and every recorded datum; `algorithm.clear_status()`.
    pub fn clear_status(&self) {
        self.status.write().clear();
        *self.report_integers.write() = None;
        *self.report_strings.write() = None;
        *self.report_messages.write() = None;
    }
    /// Sets the messenger; `None` selects the default one; `algorithm.set_messenger(None)`.
    pub fn set_messenger(&self, messenger: Option<Arc<Messenger>>) {
        *self.messenger.write() = messenger.unwrap_or_else(|| Message::default_messenger().clone());
    }
    /// The messenger; `algorithm.get_messenger()`.
    pub fn get_messenger(&self) -> Arc<Messenger> {
        self.messenger.snapshot()
    }
    /// Snapshot of integers recorded for a status; `algorithm.get_message_numbers(Status::FAIL1)`.
    pub fn get_message_numbers(&self, status: Status) -> Option<PackedMap<i32>> {
        let index = ExecStatus::status_index(status) as usize;
        self.report_integers
            .read()
            .as_ref()?
            .get(index)?
            .as_ref()
            .filter(|_| index != 0)
            .cloned()
    }
}

// The embedded base supports actual Handle up/down casts; message-name customization
// alone never substitutes for a registered derived object or its base projection.
impl Transient for Algorithm {
    fn dynamic_type(&self) -> &'static RuntimeType {
        self.runtime_type
    }
    fn project(&self, requested: std::any::TypeId) -> Option<&dyn std::any::Any> {
        if requested == std::any::TypeId::of::<Self>() {
            return Some(self);
        }
        self.parent.project(requested)
    }
    fn transient_base(&self) -> &StandardTransient {
        &self.parent
    }
}
