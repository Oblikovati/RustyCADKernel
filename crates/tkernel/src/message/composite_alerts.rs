//! `Message_CompositeAlerts`: alert lists per gravity, merging alerts of the same type.
use super::{Alert, AlertHandle, Gravity, json};
use crate::standard::dump::DumpStream;
use crate::standard::{RuntimeType, RuntimeTyped, StandardTransient};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};
mod cursor;
pub(crate) use cursor::AlertCursor;

/// Stable list-node identity keeps native iterators valid when other alerts are removed.
struct AlertEntry {
    alert: AlertHandle,
    next: Mutex<Weak<AlertEntry>>,
}

impl AlertEntry {
    fn next_entry(&self) -> Option<Arc<Self>> {
        self.next
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .upgrade()
    }

    fn set_next(&self, next: Option<&Arc<Self>>) {
        *self.next.lock().unwrap_or_else(PoisonError::into_inner) =
            next.map_or_else(Weak::new, Arc::downgrade);
    }
}

type AlertLists = [Vec<Arc<AlertEntry>>; 5];

/// One alert list per gravity; `CompositeAlerts::default()`.
#[derive(Default)]
pub struct CompositeAlerts {
    alerts: Mutex<AlertLists>,
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
    fn lists(&self) -> MutexGuard<'_, AlertLists> {
        self.alerts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
    /// The alerts of one gravity, in insertion order; `composite.alerts(Gravity::Warning)`.
    pub fn alerts(&self, gravity: Gravity) -> Vec<AlertHandle> {
        self.lists()[gravity.index()]
            .iter()
            .map(|entry| entry.alert.clone())
            .collect()
    }
    /// Adds an alert, or merges it into the first earlier alert of exactly its type that
    /// accepts the merge; true only when appended; `composite.add_alert(Gravity::Info, &alert)`.
    pub fn add_alert(&self, gravity: Gravity, alert: &AlertHandle) -> bool {
        if alert.supports_merge() && self.merge_live_alerts(gravity, alert) {
            return false;
        }
        self.append_entry(gravity, alert);
        true
    }
    fn merge_live_alerts(&self, gravity: Gravity, alert: &AlertHandle) -> bool {
        let kind = alert.dynamic_type();
        let mut cursor = AlertCursor::new(self, gravity);
        while let Some(target) = cursor.current_alert() {
            if std::ptr::eq(kind, target.dynamic_type()) && alert.merge(&target) {
                return true;
            }
            cursor.advance();
        }
        false
    }
    fn append_entry(&self, gravity: Gravity, alert: &AlertHandle) {
        let mut lists = self.lists();
        let list = &mut lists[gravity.index()];
        let entry = Arc::new(AlertEntry {
            alert: alert.clone(),
            next: Mutex::default(),
        });
        if let Some(last) = list.last() {
            last.set_next(Some(&entry));
        }
        list.push(entry);
    }
    /// Removes an alert of one gravity; `composite.remove_alert(Gravity::Info, &alert)`.
    pub fn remove_alert(&self, gravity: Gravity, alert: &AlertHandle) -> bool {
        let mut lists = self.lists();
        let list = &mut lists[gravity.index()];
        let Some(index) = list.iter().position(|item| same_alert(&item.alert, alert)) else {
            return false;
        };
        if index > 0 {
            list[index - 1].set_next(list.get(index + 1));
        }
        list.remove(index);
        true
    }
    /// Whether the alert is in any list; `composite.has_alert(&alert)`.
    pub fn has_alert(&self, alert: &AlertHandle) -> bool {
        self.lists()
            .iter()
            .flatten()
            .any(|item| same_alert(&item.alert, alert))
    }
    /// Whether an alert of exactly `kind` has this gravity; `composite.has_alert_of_type(t, g)`.
    pub fn has_alert_of_type(&self, kind: &RuntimeType, gravity: Gravity) -> bool {
        self.lists()[gravity.index()]
            .iter()
            .any(|item| is_instance(item.alert.as_ref(), kind))
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
            list.retain(|item| !is_instance(item.alert.as_ref(), kind));
            relink_entries(list);
        }
    }
    /// `DumpJson`: the class name then `"Alert<n>": {...}` numbered across all gravities.
    pub fn dump_json(&self, output: &mut DumpStream, depth: i32) {
        json::class_begin(output, Self::static_type().name());
        let alerts: Vec<AlertHandle> = self
            .lists()
            .iter()
            .flatten()
            .map(|entry| entry.alert.clone())
            .collect();
        for (number, alert) in alerts.iter().enumerate() {
            let name = format!("Alert{}", number + 1);
            json::field_dumped(output, depth, &name, |field, depth| {
                alert.dump_json(field, depth)
            });
        }
    }
}

fn relink_entries(list: &[Arc<AlertEntry>]) {
    for pair in list.windows(2) {
        pair[0].set_next(Some(&pair[1]));
    }
    if let Some(last) = list.last() {
        last.set_next(None);
    }
}

/// `IsInstance`: exactly this descriptor, not a derived one.
fn is_instance(alert: &dyn Alert, kind: &RuntimeType) -> bool {
    alert.dynamic_type().rust_type() == kind.rust_type()
}
