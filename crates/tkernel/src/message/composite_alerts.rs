//! `Message_CompositeAlerts`: alert lists per gravity, merging alerts of the same type.
use super::{Alert, AlertHandle, Gravity, json};
use crate::standard::dump::DumpStream;
use crate::standard::{RuntimeType, RuntimeTyped, StandardTransient};
use std::sync::{Arc, Mutex, MutexGuard};

/// One alert list per gravity; `CompositeAlerts::default()`.
#[derive(Default)]
pub struct CompositeAlerts {
    alerts: Mutex<[Vec<AlertHandle>; 5]>,
}
message_runtime_type!(
    CompositeAlerts,
    "Message_CompositeAlerts",
    StandardTransient
);

/// Alert identity: the same allocation.
pub(crate) fn same_alert(first: &AlertHandle, second: &AlertHandle) -> bool {
    std::ptr::addr_eq(Arc::as_ptr(first), Arc::as_ptr(second))
}

impl CompositeAlerts {
    /// Creates empty lists; `CompositeAlerts::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    fn lists(&self) -> MutexGuard<'_, [Vec<AlertHandle>; 5]> {
        self.alerts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
    /// The alerts of one gravity, in insertion order; `composite.alerts(Gravity::Warning)`.
    pub fn alerts(&self, gravity: Gravity) -> Vec<AlertHandle> {
        self.lists()[gravity.index()].clone()
    }
    /// Adds an alert, or merges it into the first earlier alert of exactly its type that
    /// accepts the merge; true only when appended; `composite.add_alert(Gravity::Info, &alert)`.
    pub fn add_alert(&self, gravity: Gravity, alert: &AlertHandle) -> bool {
        let existing = self.alerts(gravity);
        if alert.supports_merge() && !existing.is_empty() {
            let kind = alert.dynamic_type();
            // Merge callbacks run without the lock: user alerts may inspect other alerts.
            let merged = existing
                .iter()
                .any(|target| std::ptr::eq(kind, target.dynamic_type()) && alert.merge(target));
            if merged {
                return false;
            }
        }
        self.lists()[gravity.index()].push(alert.clone());
        true
    }
    /// Removes an alert of one gravity; `composite.remove_alert(Gravity::Info, &alert)`.
    pub fn remove_alert(&self, gravity: Gravity, alert: &AlertHandle) -> bool {
        let mut lists = self.lists();
        let list = &mut lists[gravity.index()];
        let Some(index) = list.iter().position(|item| same_alert(item, alert)) else {
            return false;
        };
        list.remove(index);
        true
    }
    /// Whether the alert is in any list; `composite.has_alert(&alert)`.
    pub fn has_alert(&self, alert: &AlertHandle) -> bool {
        self.lists()
            .iter()
            .flatten()
            .any(|item| same_alert(item, alert))
    }
    /// Whether an alert of exactly `kind` has this gravity; `composite.has_alert_of_type(t, g)`.
    pub fn has_alert_of_type(&self, kind: &RuntimeType, gravity: Gravity) -> bool {
        self.lists()[gravity.index()]
            .iter()
            .any(|item| is_instance(item.as_ref(), kind))
    }
    /// Empties every list; `composite.clear()`.
    pub fn clear(&self) {
        self.lists().iter_mut().for_each(Vec::clear);
    }
    /// Empties one gravity's list; `composite.clear_gravity(Gravity::Fail)`.
    pub fn clear_gravity(&self, gravity: Gravity) {
        self.lists()[gravity.index()].clear();
    }
    /// Removes the alerts of exactly `kind` from every list; `composite.clear_type(t)`.
    ///
    /// Deliberate deviation: the native loop never advances past an alert of another type
    /// (`anIt.More()` where `anIt.Next()` was meant) and hangs; Rust keeps such alerts and
    /// continues, which is the native result whenever the native call terminates.
    pub fn clear_type(&self, kind: &RuntimeType) {
        for list in self.lists().iter_mut() {
            list.retain(|item| !is_instance(item.as_ref(), kind));
        }
    }
    /// `DumpJson`: the class name then `"Alert<n>": {...}` numbered across all gravities.
    pub fn dump_json(&self, output: &mut DumpStream, depth: i32) {
        json::class_begin(output, Self::static_type().name());
        let alerts: Vec<AlertHandle> = self.lists().iter().flatten().cloned().collect();
        for (number, alert) in alerts.iter().enumerate() {
            let name = format!("Alert{}", number + 1);
            json::field_dumped(output, depth, &name, |field, depth| {
                alert.dump_json(field, depth)
            });
        }
    }
}

/// `IsInstance`: exactly this descriptor, not a derived one.
fn is_instance(alert: &dyn Alert, kind: &RuntimeType) -> bool {
    alert.dynamic_type().rust_type() == kind.rust_type()
}
