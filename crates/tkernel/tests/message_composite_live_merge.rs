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
}

struct LiveMergeAlert {
    label: &'static str,
    composite: Weak<CompositeAlerts>,
    can_merge: bool,
    append_moment: Option<AppendMoment>,
    succeeds_on: &'static str,
    events: Arc<Mutex<Vec<&'static str>>>,
    support_calls: Arc<AtomicUsize>,
}
rustycad_tkernel::message_runtime_type!(LiveMergeAlert, "LiveMergeAlert", BaseAlert);

impl LiveMergeAlert {
    fn new(
        label: &'static str,
        composite: &Arc<CompositeAlerts>,
        can_merge: bool,
        append_moment: Option<AppendMoment>,
        succeeds_on: &'static str,
        events: Arc<Mutex<Vec<&'static str>>>,
        support_calls: Arc<AtomicUsize>,
    ) -> Arc<Self> {
        Arc::new(Self {
            label,
            composite: Arc::downgrade(composite),
            can_merge,
            append_moment,
            succeeds_on,
            events,
            support_calls,
        })
    }

    fn append_target(&self, label: &'static str) {
        let Some(composite) = self.composite.upgrade() else {
            return;
        };
        let target = Self::new(
            label,
            &composite,
            false,
            None,
            "",
            self.events.clone(),
            self.support_calls.clone(),
        );
        composite.add_alert(Gravity::Info, &(target as AlertHandle));
    }
}

impl Alert for LiveMergeAlert {
    fn dynamic_type(&self) -> &'static RuntimeType {
        Self::static_type()
    }

    fn supports_merge(&self) -> bool {
        self.support_calls.fetch_add(1, Ordering::SeqCst);
        if self.append_moment == Some(AppendMoment::SupportsMerge) {
            self.append_target("support-target");
        }
        self.can_merge
    }

    fn merge(&self, target: &AlertHandle) -> bool {
        let target = downcast_alert::<Self>(target).expect("matching runtime type");
        self.events.lock().unwrap().push(target.label);
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
    let events = Arc::new(Mutex::new(Vec::new()));
    let support_calls = Arc::new(AtomicUsize::new(0));
    let incoming = LiveMergeAlert::new(
        "incoming",
        &composite,
        true,
        Some(AppendMoment::SupportsMerge),
        "support-target",
        events.clone(),
        support_calls.clone(),
    );

    assert!(!composite.add_alert(Gravity::Info, &(incoming as AlertHandle)));
    assert_eq!(target_labels(&composite), ["support-target"]);
    assert_eq!(*events.lock().unwrap(), ["support-target"]);
    assert_eq!(support_calls.load(Ordering::SeqCst), 2);
}

#[test]
fn merge_callback_appends_a_target_visited_by_the_same_cursor() {
    let composite = Arc::new(CompositeAlerts::new());
    let events = Arc::new(Mutex::new(Vec::new()));
    let support_calls = Arc::new(AtomicUsize::new(0));
    let first = LiveMergeAlert::new(
        "first",
        &composite,
        false,
        None,
        "",
        events.clone(),
        support_calls.clone(),
    );
    composite.add_alert(Gravity::Info, &(first as AlertHandle));
    let incoming = LiveMergeAlert::new(
        "incoming",
        &composite,
        true,
        Some(AppendMoment::MergeFirst),
        "late-target",
        events.clone(),
        support_calls.clone(),
    );

    assert!(!composite.add_alert(Gravity::Info, &(incoming as AlertHandle)));
    assert_eq!(target_labels(&composite), ["first", "late-target"]);
    assert_eq!(*events.lock().unwrap(), ["first", "late-target"]);
    assert_eq!(support_calls.load(Ordering::SeqCst), 3);
}
