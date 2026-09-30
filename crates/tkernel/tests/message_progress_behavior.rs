//! Supplemental behavior of the Message progress classes (indicator, range, scope,
//! sentry, lazy scope); the bit-exact comparison with OCCT is the
//! `msgprog_*` family of the TKernel oracle probe.
use rustycad_tkernel::message::{
    LazyProgressScope, ProgressIndicator, ProgressIndicatorHooks, ProgressRange, ProgressScope,
    ProgressSentry,
};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

/// One `show` call: scope name, position and the force flag.
type ShowRecord = (Option<Vec<u8>>, f64, bool);

/// Fake indicator recording every hook call; breaks after `break_after` polls.
#[derive(Default)]
struct RecordingIndicator {
    shows: Mutex<Vec<ShowRecord>>,
    polls: AtomicUsize,
    resets: AtomicUsize,
    break_after: Option<usize>,
}
/// Hooks owning a shared handle to the record, so tests can inspect it afterwards.
struct RecordingHooks(Arc<RecordingIndicator>);
impl ProgressIndicatorHooks for RecordingHooks {
    fn show(&self, scope: &ProgressScope<'_>, position: f64, force: bool) {
        let name = scope.name().map(<[u8]>::to_vec);
        self.0.shows.lock().unwrap().push((name, position, force));
    }
    fn user_break(&self) -> bool {
        let polls = self.0.polls.fetch_add(1, Ordering::SeqCst) + 1;
        self.0.break_after.is_some_and(|limit| polls > limit)
    }
    fn reset(&self) {
        self.0.resets.fetch_add(1, Ordering::SeqCst);
    }
}
fn indicator(break_after: Option<usize>) -> (ProgressIndicator, Arc<RecordingIndicator>) {
    let record = Arc::new(RecordingIndicator {
        break_after,
        ..RecordingIndicator::default()
    });
    (
        ProgressIndicator::new(Box::new(RecordingHooks(Arc::clone(&record)))),
        record,
    )
}
fn positions(record: &RecordingIndicator) -> Vec<f64> {
    record.shows.lock().unwrap().iter().map(|s| s.1).collect()
}

#[test]
fn start_resets_shows_root_and_dropped_range_completes() {
    let (progress, record) = indicator(None);
    let range = progress.start();
    assert!(range.is_active());
    assert_eq!(record.resets.load(Ordering::SeqCst), 1);
    assert_eq!(positions(&record), [0.0]);
    drop(range);
    assert_eq!(positions(&record), [0.0, 1.0]);
    assert_eq!(progress.get_position(), 1.0);
    assert!(progress.root_scope().name().is_none());
}
#[test]
fn nested_scopes_map_steps_onto_the_parent_range() {
    let (progress, record) = indicator(None);
    let range = progress.start();
    let outer = ProgressScope::with_literal(&range, "Outer", 4., false);
    assert!(!range.is_active());
    for _ in 0..2 {
        let inner = ProgressScope::with_literal(&outer.next(1.), "Inner", 2., false);
        drop(inner.next(1.));
        assert_eq!(inner.parent().unwrap().name(), Some(&b"Outer"[..]));
    }
    drop(outer);
    assert_eq!(positions(&record), [0.0, 0.125, 0.25, 0.375, 0.5, 1.0]);
}
#[test]
fn infinite_scope_advances_on_the_hyperbola() {
    let (progress, record) = indicator(None);
    let range = progress.start();
    let scope = ProgressScope::unnamed(&range, 2., true);
    drop(scope.next(1.));
    drop(scope.next(1.));
    assert_eq!(positions(&record)[1..], [1. / 3., 0.5]);
    scope.close();
    assert_eq!(scope.value(), rustycad_tkernel::precision::infinite());
}
#[test]
fn names_follow_the_three_native_constructors() {
    let range = ProgressRange::default();
    assert_eq!(
        ProgressScope::with_literal(&range, "", 1., false).name(),
        Some(&b""[..])
    );
    assert!(
        ProgressScope::new(&range, &AsciiString::new(""), 1., false)
            .name()
            .is_none()
    );
    let mut scope = ProgressScope::unnamed(&range, 1., false);
    assert!(scope.name().is_none());
    scope.set_name(&AsciiString::from_bytes(b"ab\0cd"));
    assert_eq!(scope.name(), Some(&b"ab"[..]));
    scope.set_literal_name("lit");
    assert_eq!(scope.name(), Some(&b"lit"[..]));
}
#[test]
fn clone_disarms_source_and_clone_from_drops_target_advance() {
    let (progress, record) = indicator(None);
    let range = progress.start();
    let copy = range.clone();
    assert!(!range.is_active() && copy.is_active());
    let mut target = ProgressRange::default();
    target.clone_from(&copy);
    assert!(!copy.is_active() && target.is_active());
    drop((range, copy, target));
    assert_eq!(positions(&record), [0.0, 1.0]);
}
#[test]
fn a_used_range_builds_an_inactive_scope_and_zero_max_is_clamped() {
    let (progress, _record) = indicator(None);
    let range = progress.start();
    let first = ProgressScope::unnamed(&range, 0., false);
    let second = ProgressScope::unnamed(&range, 3., false);
    assert!(first.is_active() && !second.is_active());
    assert_eq!(first.max_value(), 1.0e-6);
    assert_eq!(second.value(), 3.0);
    assert_eq!(first.get_portion(), 1.0);
}
#[test]
fn value_rounds_the_global_position_up() {
    let (progress, _record) = indicator(None);
    let range = progress.start();
    let scope = ProgressScope::unnamed(&range, 3., false);
    assert_eq!(scope.value(), 0.0);
    drop(scope.next(1.));
    assert_eq!(scope.value(), 1.0);
    drop(scope.next(0.5));
    assert_eq!(scope.value(), 2.0);
    assert!(scope.next(0.).is_active().eq(&false));
}
#[test]
fn user_break_is_polled_through_scopes_and_ranges() {
    let (progress, record) = indicator(Some(2));
    let range = progress.start();
    assert!(range.more());
    let scope = ProgressScope::unnamed(&range, 3., false);
    assert!(scope.more());
    assert!(scope.user_break());
    assert_eq!(record.polls.load(Ordering::SeqCst), 3);
    assert!(!ProgressRange::default().user_break());
}
#[test]
fn sentry_refuses_legacy_parameters_after_consuming_the_range() {
    let (progress, record) = indicator(None);
    let range = progress.start();
    let error = ProgressSentry::new(&range, "Legacy", 1., 10., 1., false, 0.)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
    assert!(error.message().contains("min 1"));
    assert!(!range.is_active());
    assert_eq!(positions(&record), [0.0, 1.0]);
    let range = progress.start();
    let sentry = ProgressSentry::new(&range, "Legacy", 0., 10., 1., false, 0.).unwrap();
    sentry.relieve();
    assert!(!sentry.is_active());
}
#[test]
fn lazy_scope_advances_and_polls_once_per_patch() {
    let (progress, record) = indicator(Some(1));
    let range = progress.start();
    let mut lazy = LazyProgressScope::new(&range, "Lazy", 10., 4., false);
    for _ in 0..3 {
        lazy.next();
    }
    assert_eq!(record.polls.load(Ordering::SeqCst), 0);
    lazy.next();
    assert!(lazy.more());
    for _ in 0..4 {
        lazy.next();
    }
    assert!(!lazy.more() && lazy.is_aborted());
    assert_eq!(positions(&record)[1..], [0.4, 0.8]);
}
#[test]
fn dummy_ranges_and_scopes_never_reach_an_indicator() {
    let range = ProgressIndicator::start_optional(None);
    assert!(!range.is_active());
    let scope = ProgressScope::default();
    assert!(!scope.is_active() && scope.parent().is_none());
    assert!(!scope.next(1.).is_active());
    assert_eq!(scope.value(), 1.0);
}
#[test]
fn ranges_advance_the_indicator_from_parallel_tasks() {
    let (progress, record) = indicator(None);
    let range = progress.start();
    let scope = ProgressScope::unnamed(&range, 8., false);
    let ranges: Vec<ProgressRange<'_>> = (0..8).map(|_| scope.next(1.)).collect();
    std::thread::scope(|threads| {
        for mut task in ranges {
            threads.spawn(move || task.close());
        }
    });
    assert_eq!(positions(&record).len(), 9);
    assert_eq!(progress.get_position(), 1.0);
}
#[test]
fn show_forces_only_armed_scopes_and_indicator_drop_is_silent() {
    let (progress, record) = indicator(None);
    let range = progress.start();
    let scope = ProgressScope::with_literal(&range, "S", 2., false);
    scope.show();
    scope.close();
    scope.show();
    assert_eq!(
        record.shows.lock().unwrap()[1],
        (Some(b"S".to_vec()), 0.0, true)
    );
    assert_eq!(record.shows.lock().unwrap().len(), 3);
    drop(scope);
    drop(range);
    drop(progress);
    assert_eq!(record.shows.lock().unwrap().len(), 3);
}
