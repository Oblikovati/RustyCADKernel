//! Common-allocator observations compare real handle deltas against the ambient process baseline.
use super::fixture::native_row;
use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy, OrderedDataMap, OrderedMap};
use std::{
    cell::RefCell,
    panic::{AssertUnwindSafe, catch_unwind},
};
struct DefaultTrace {
    arena: BaseAllocator,
    events: RefCell<String>,
    reject: bool,
    baseline: i32,
    relative: bool,
}
thread_local! {static DEFAULT_TRACE:RefCell<Option<DefaultTrace>>=const {RefCell::new(None)};}
fn with_trace<R>(action: impl FnOnce(&DefaultTrace) -> R) -> R {
    DEFAULT_TRACE.with(|trace| {
        action(
            trace
                .borrow()
                .as_ref()
                .expect("scoped default trace is active"),
        )
    })
}
impl DefaultTrace {
    fn new(common: bool, reject: bool) -> Self {
        let arena = if common {
            BaseAllocator::common()
        } else {
            BaseAllocator::accumulating(4096).unwrap()
        };
        let baseline = arena.ref_count();
        Self {
            arena,
            events: RefCell::new(String::new()),
            reject,
            baseline,
            relative: common,
        }
    }
    fn verify(&self, label: &str, outcome: &str) {
        let events = self.events.borrow();
        let text = if events.is_empty() { "-" } else { &events };
        native_row(label, format!("{label} {outcome} {} {text}", self.count()));
    }
    fn count(&self) -> i32 {
        self.arena.ref_count() - if self.relative { self.baseline } else { 0 }
    }
    fn note(&self, tag: &str) {
        self.events
            .borrow_mut()
            .push_str(&format!("{tag}r{},", self.count()));
    }
}
struct DefaultScope;
impl DefaultScope {
    fn new(common: bool, reject: bool) -> Self {
        DEFAULT_TRACE.with(|slot| *slot.borrow_mut() = Some(DefaultTrace::new(common, reject)));
        Self
    }
}
impl Drop for DefaultScope {
    fn drop(&mut self) {
        DEFAULT_TRACE.with(|slot| {
            slot.borrow_mut().take();
        });
    }
}
struct DefaultPolicy;
impl Default for DefaultPolicy {
    fn default() -> Self {
        with_trace(|trace| {
            trace.note("HF");
            assert!(
                !trace.reject,
                "default policy rejected; expected accepted policy"
            );
        });
        Self
    }
}
impl Drop for DefaultPolicy {
    fn drop(&mut self) {
        with_trace(|trace| trace.note("HD"));
    }
}
impl HashPolicy<i32> for DefaultPolicy {
    fn hash(&self, key: &i32) -> usize {
        *key as usize
    }
    fn equal(&self, a: &i32, b: &i32) -> bool {
        a == b
    }
}
#[derive(Clone, Copy)]
enum DefaultMode {
    Bare,
    Size,
    Signed,
    Negative,
    CommonInt,
}
macro_rules! default_adapter {
    ($function:ident,$map:ty) => {
        fn $function(mode: DefaultMode) -> Result<String, rustycad_tkernel::standard::Failure> {
            let map = match mode {
                DefaultMode::Bare => <$map>::default(),
                DefaultMode::CommonInt => <$map>::new_int(0)?,
                DefaultMode::Size => {
                    <$map>::with_default_allocator(13, with_trace(|trace| trace.arena.clone()))
                }
                DefaultMode::Signed => {
                    <$map>::with_default_allocator_int(0, with_trace(|trace| trace.arena.clone()))?
                }
                DefaultMode::Negative => {
                    <$map>::with_default_allocator_int(-7, with_trace(|trace| trace.arena.clone()))?
                }
            };
            Ok(format!("ok {} {}", map.len(), map.bucket_count()))
        }
    };
}
default_adapter!(set_default,OrderedMap<i32,DefaultPolicy>);
default_adapter!(data_default,OrderedDataMap<i32,i32,DefaultPolicy>);
fn default_outcome(
    constructor: impl FnOnce() -> Result<String, rustycad_tkernel::standard::Failure>,
) -> String {
    match catch_unwind(AssertUnwindSafe(constructor)) {
        Ok(Ok(text)) => text,
        Ok(Err(error)) => {
            assert!(error.to_string().contains("-7"));
            "range".to_owned()
        }
        Err(_) => "throw".to_owned(),
    }
}
fn default_case(prefix: &str, label: &str, mode: DefaultMode, reject: bool) {
    let _scope = DefaultScope::new(
        matches!(mode, DefaultMode::Bare | DefaultMode::CommonInt),
        reject,
    );
    let outcome = default_outcome(|| {
        if prefix == "set" {
            set_default(mode)
        } else {
            data_default(mode)
        }
    });
    let label = format!("{prefix}_{label}");
    with_trace(|trace| trace.verify(&label, &outcome));
}
#[test]
fn default_constructor_order_and_negative_validation_match_native_for_both_families() {
    for prefix in ["set", "data"] {
        for (label, mode, reject) in cases() {
            default_case(prefix, label, mode, reject);
        }
    }
}
fn cases() -> [(&'static str, DefaultMode, bool); 9] {
    use DefaultMode::*;
    [
        ("default_bare", Bare, false),
        ("default_size", Size, false),
        ("default_signed", Signed, false),
        ("default_negative", Negative, false),
        ("default_throw_size", Size, true),
        ("default_negative_throw", Negative, true),
        ("default_common_int", CommonInt, false),
        ("default_throw_bare", Bare, true),
        ("default_throw_signed", Signed, true),
    ]
}
