//! Composite alert callbacks observe alerts appended during `AddAlert` traversal.
use rustycad_tkernel::message::{
    Alert, AlertHandle, BaseAlert, CompositeAlerts, Gravity, downcast_alert,
};
use rustycad_tkernel::standard::{RuntimeType, RuntimeTyped};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};
#[path = "message_report_support/mod.rs"]
mod support;
use support::named_alert;

#[derive(Clone, Copy, PartialEq, Eq)]
enum AppendMoment {
    SupportsMerge,
    MergeFirst,
    RemoveFirst,
}

struct LiveMergeAlert {
    label: &'static str,
    composite: Weak<CompositeAlerts>,
    can_merge: bool,
    append_moment: Option<AppendMoment>,
    succeeds_on: &'static str,
    counters: Arc<MergeCounters>,
}
rustycad_tkernel::message_runtime_type!(LiveMergeAlert, "LiveMergeAlert", BaseAlert);

#[derive(Default)]
struct MergeCounters {
    events: Mutex<Vec<&'static str>>,
    support_calls: AtomicUsize,
    dynamic_calls: AtomicUsize,
}

impl LiveMergeAlert {
    fn new(
        label: &'static str,
        composite: &Arc<CompositeAlerts>,
        can_merge: bool,
        append_moment: Option<AppendMoment>,
        succeeds_on: &'static str,
        counters: Arc<MergeCounters>,
    ) -> Arc<Self> {
        Arc::new(Self {
            label,
            composite: Arc::downgrade(composite),
            can_merge,
            append_moment,
            succeeds_on,
            counters,
        })
    }

    fn append_target(&self, label: &'static str) {
        let Some(composite) = self.composite.upgrade() else {
            return;
        };
        let target = Self::new(label, &composite, false, None, "", self.counters.clone());
        composite.add_alert(Gravity::Info, &(target as AlertHandle));
    }

    fn remove_target(&self, label: &'static str) {
        let Some(composite) = self.composite.upgrade() else {
            return;
        };
        let target = composite
            .alerts(Gravity::Info)
            .iter()
            .find_map(downcast_alert::<LiveMergeAlert>)
            .filter(|alert| alert.label == label);
        let Some(target) = target else {
            return;
        };
        let target: AlertHandle = target;
        composite.remove_alert(Gravity::Info, &target);
    }
}

impl Alert for LiveMergeAlert {
    fn dynamic_type(&self) -> &'static RuntimeType {
        self.counters.dynamic_calls.fetch_add(1, Ordering::SeqCst);
        if self.append_moment == Some(AppendMoment::RemoveFirst) {
            self.remove_target("first");
        }
        Self::static_type()
    }

    fn supports_merge(&self) -> bool {
        self.counters.support_calls.fetch_add(1, Ordering::SeqCst);
        if self.append_moment == Some(AppendMoment::SupportsMerge) {
            self.append_target("support-target");
        }
        self.can_merge
    }

    fn merge(&self, target: &AlertHandle) -> bool {
        let target = downcast_alert::<Self>(target).expect("matching runtime type");
        self.counters.events.lock().unwrap().push(target.label);
        if self.append_moment == Some(AppendMoment::MergeFirst) && target.label == "first" {
            self.append_target("late-target");
        }
        target.label == self.succeeds_on
    }
}

fn target_labels(composite: &CompositeAlerts) -> Vec<&'static str> {
    composite
        .alerts(Gravity::Info)
        .iter()
        .filter_map(downcast_alert::<LiveMergeAlert>)
        .map(|alert| alert.label)
        .collect()
}

#[test]
fn supports_merge_appends_before_the_live_cursor_is_created() {
    let composite = Arc::new(CompositeAlerts::new());
    composite.add_alert(Gravity::Info, &named_alert("other type"));
    let counters = Arc::new(MergeCounters::default());
    let incoming = LiveMergeAlert::new(
        "incoming",
        &composite,
        true,
        Some(AppendMoment::SupportsMerge),
        "support-target",
        counters.clone(),
    );

    assert!(!composite.add_alert(Gravity::Info, &(incoming as AlertHandle)));
    assert_eq!(target_labels(&composite), ["support-target"]);
    assert_eq!(*counters.events.lock().unwrap(), ["support-target"]);
    assert_eq!(counters.support_calls.load(Ordering::SeqCst), 2);
    assert_eq!(counters.dynamic_calls.load(Ordering::SeqCst), 2);
}

#[test]
fn merge_callback_appends_a_target_visited_by_the_same_cursor() {
    let composite = Arc::new(CompositeAlerts::new());
    let counters = Arc::new(MergeCounters::default());
    let first = LiveMergeAlert::new("first", &composite, false, None, "", counters.clone());
    composite.add_alert(Gravity::Info, &(first as AlertHandle));
    let incoming = LiveMergeAlert::new(
        "incoming",
        &composite,
        true,
        Some(AppendMoment::MergeFirst),
        "late-target",
        counters.clone(),
    );

    assert!(!composite.add_alert(Gravity::Info, &(incoming as AlertHandle)));
    assert_eq!(target_labels(&composite), ["first", "late-target"]);
    assert_eq!(*counters.events.lock().unwrap(), ["first", "late-target"]);
    assert_eq!(counters.support_calls.load(Ordering::SeqCst), 3);
    assert_eq!(counters.dynamic_calls.load(Ordering::SeqCst), 3);
}

#[test]
fn empty_list_skips_incoming_dynamic_type_query() {
    let composite = Arc::new(CompositeAlerts::new());
    let counters = Arc::new(MergeCounters::default());
    let incoming = LiveMergeAlert::new("incoming", &composite, true, None, "", counters.clone());

    assert!(composite.add_alert(Gravity::Info, &(incoming as AlertHandle)));
    assert_eq!(counters.dynamic_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn dynamic_type_removal_happens_before_cursor_creation() {
    let composite = Arc::new(CompositeAlerts::new());
    let counters = Arc::new(MergeCounters::default());
    let first = LiveMergeAlert::new("first", &composite, false, None, "", counters.clone());
    let second = LiveMergeAlert::new("second", &composite, false, None, "", counters.clone());
    composite.add_alert(Gravity::Info, &(first as AlertHandle));
    composite.add_alert(Gravity::Info, &(second as AlertHandle));
    let incoming = LiveMergeAlert::new(
        "incoming",
        &composite,
        true,
        Some(AppendMoment::RemoveFirst),
        "first",
        counters.clone(),
    );

    assert!(composite.add_alert(Gravity::Info, &(incoming as AlertHandle)));
    assert_eq!(target_labels(&composite), ["second", "incoming"]);
    assert_eq!(*counters.events.lock().unwrap(), ["second"]);
    assert_eq!(counters.dynamic_calls.load(Ordering::SeqCst), 2);
}
