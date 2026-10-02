//! Behavior of BOPAlgo_Options, including native floating-point edge cases.
use rustycad_tkbo::bop_algo::{AlertUserBreak, Options};
use rustycad_tkernel::message::{
    AlertHandle, BaseAlert, Gravity, MsgFile, ProgressIndicator, ProgressIndicatorHooks,
    ProgressScope,
};
use rustycad_tkernel::ncollection::BaseAllocator;
use rustycad_tkernel::standard::RuntimeTyped;
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::Arc;

#[test]
fn allocator_identity_and_default_configuration() {
    let options = Options::default();
    assert!(options.allocator() == Some(&BaseAllocator::common()));
    assert!(!options.use_obb());
    assert_eq!(options.fuzzy_value(), 1e-7);
    assert!(!options.has_errors() && !options.has_warnings());
    let heap = BaseAllocator::global_heap();
    let explicit = Options::with_allocator(Some(heap.clone()));
    assert!(explicit.allocator() == Some(&heap));
    assert!(Options::with_allocator(None).allocator().is_none());
    assert!(MsgFile::global().has_msg(&AsciiString::new("BOPAlgo_LOAD_CHECKER")));
}

#[test]
fn global_parallel_default_is_captured_at_construction() {
    let previous = Options::get_parallel_mode();
    Options::set_parallel_mode(true);
    let mut enabled = Options::new();
    Options::set_parallel_mode(false);
    let disabled = Options::new();
    Options::set_parallel_mode(previous);
    assert!(enabled.run_parallel());
    assert!(!disabled.run_parallel());
    enabled.set_run_parallel(false);
    assert!(!enabled.run_parallel());
}

#[test]
fn fuzzy_lower_bound_preserves_nan_and_infinity() {
    let mut options = Options::new();
    for tolerance in [f64::NEG_INFINITY, -1.0, -0.0, 0.0, 1e-8, 1e-7] {
        options.set_fuzzy_value(tolerance);
        assert_eq!(options.fuzzy_value(), 1e-7);
    }
    for tolerance in [1.0, f64::INFINITY, f64::NAN] {
        options.set_fuzzy_value(tolerance);
        assert_eq!(options.fuzzy_value().to_bits(), tolerance.to_bits());
    }
}

fn options_with_alerts() -> Options {
    let options = Options::new();
    options.add_error(&(Arc::new(AlertUserBreak) as AlertHandle));
    options.add_warning(&(Arc::new(BaseAlert) as AlertHandle));
    options
}

#[test]
fn report_classifies_and_dumps_each_severity() {
    let options = options_with_alerts();
    assert!(options.has_errors() && options.has_warnings());
    assert!(options.has_error(AlertUserBreak::static_type()));
    assert!(!options.has_warning(AlertUserBreak::static_type()));
    assert!(options.has_warning(BaseAlert::static_type()));
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    options.dump_errors(&mut errors).unwrap();
    options.dump_warnings(&mut warnings).unwrap();
    assert_eq!(errors, b"BOPAlgo_AlertUserBreak\n");
    assert_eq!(warnings, b"Message_Alert\n");
}

#[test]
fn clear_preserves_options_and_shared_report_identity() {
    let mut options = options_with_alerts();
    options.set_use_obb(true);
    options.set_fuzzy_value(0.01);
    options.set_run_parallel(true);
    let copy = options.clone();
    assert!(Arc::ptr_eq(options.report(), copy.report()));
    options.clear_warnings();
    assert!(!copy.has_warnings() && copy.has_errors());
    options
        .report()
        .add_alert(Gravity::Info, &(Arc::new(BaseAlert) as AlertHandle));
    options.clear();
    assert!(!copy.has_errors());
    assert!(copy.report().get_alerts(Gravity::Info).is_empty());
    assert!(options.use_obb() && options.run_parallel());
    assert_eq!(options.fuzzy_value(), 0.01);
}

struct CancelledProgress;
impl ProgressIndicatorHooks for CancelledProgress {
    fn show(&self, _: &ProgressScope<'_>, _: f64, _: bool) {}
    fn user_break(&self) -> bool {
        true
    }
}

#[test]
fn cancellation_preserves_native_report_merge_removal() {
    let options = Options::new();
    assert!(!options.user_break(&ProgressScope::default()));
    let indicator = ProgressIndicator::new(Box::new(CancelledProgress));
    let range = indicator.start();
    let scope = ProgressScope::unnamed(&range, 1.0, false);
    assert!(options.user_break(&scope));
    assert!(options.has_error(AlertUserBreak::static_type()));
    // Pinned Message_Report removes the earlier alert on a merge with limit -1.
    assert!(options.user_break(&scope));
    assert_eq!(options.report().get_alerts(Gravity::Fail).len(), 0);
}
