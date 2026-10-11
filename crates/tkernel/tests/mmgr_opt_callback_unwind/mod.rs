//! A serialized, thread-local callback recorder makes panic-prefix tests independent.
use rustycad_tkernel::standard::{MMgrOpt, StorageAddress};
use std::{
    cell::RefCell,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Mutex, MutexGuard},
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PanicPhase {
    #[default]
    Never,
    Free,
    Allocate,
}
type CallbackEvent = (bool, StorageAddress, usize, usize);
#[derive(Default)]
struct CallbackTrace {
    phase: PanicPhase,
    events: Vec<CallbackEvent>,
    replaced: usize,
}
thread_local! { static TRACE: RefCell<CallbackTrace> = RefCell::new(CallbackTrace::default()); }
static CALLBACK_SERIAL: Mutex<()> = Mutex::new(());
pub struct CallbackScope {
    _serial: MutexGuard<'static, ()>,
}

impl CallbackScope {
    /// Installs an isolated callback policy; `CallbackScope::new(PanicPhase::Free)`.
    pub fn new(phase: PanicPhase) -> Self {
        Self::with_setup(phase, || ()).0
    }

    /// Serializes setup before installing the observer; `CallbackScope::with_setup(PanicPhase::Free, || allocator.allocate(24))`.
    pub fn with_setup<T>(phase: PanicPhase, setup: impl FnOnce() -> T) -> (Self, T) {
        let serial = CALLBACK_SERIAL
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // Setup can notify the global callback, so it belongs inside the critical
        // section too (macOS runner 38141570023; reproduced locally in six runs).
        let prepared = setup();
        TRACE.with(|trace| {
            *trace.borrow_mut() = CallbackTrace {
                phase,
                ..CallbackTrace::default()
            }
        });
        MMgrOpt::set_call_back_function(Some(observe));
        (Self { _serial: serial }, prepared)
    }
}
impl Drop for CallbackScope {
    fn drop(&mut self) {
        // Unwinding a failed assertion must never leave the process-global callback installed.
        MMgrOpt::set_call_back_function(None);
    }
}

fn observe(allocate: bool, storage: StorageAddress, round: usize, size: usize) {
    let phase = record_event((allocate, storage, round, size));
    let expected = if allocate {
        PanicPhase::Allocate
    } else {
        PanicPhase::Free
    };
    assert_ne!(
        phase, expected,
        "callback allocate={allocate}: expected configured unwind"
    );
}

fn record_event(event: CallbackEvent) -> PanicPhase {
    TRACE.with(|trace| {
        let mut trace = trace.borrow_mut();
        trace.events.push(event);
        trace.phase
    })
}

/// Changes the unwind phase before repair; `set_phase(PanicPhase::Never)`.
pub fn set_phase(phase: PanicPhase) {
    // Change policy after catch_unwind so retained allocations can be explicitly repaired.
    TRACE.with(|trace| trace.borrow_mut().phase = phase);
}

/// Returns handles captured before callback failure; `captured_allocations()[0]`.
pub fn captured_allocations() -> Vec<StorageAddress> {
    TRACE.with(|trace| {
        trace
            .borrow()
            .events
            .iter()
            .filter_map(|event| event.0.then_some(event.1))
            .collect()
    })
}

/// Formats native notification order without addresses; `event_shapes()`.
pub fn event_shapes() -> String {
    TRACE.with(|trace| {
        trace
            .borrow()
            .events
            .iter()
            .map(|event| format!("{}:{}:{}", u8::from(event.0), event.2, event.3))
            .collect::<Vec<String>>()
            .join(",")
    })
}

/// Catches the configured unwind for state inspection; `catches(|| allocator.free(None).unwrap())`.
pub fn catches(action: impl FnOnce()) -> bool {
    // The tested native callback exception has a direct Rust unwind counterpart.
    catch_unwind(AssertUnwindSafe(action)).is_err()
}

/// Compares a complete measured native row; `assert_native_row("silent-paths 1 0")`.
pub fn assert_native_row(actual: &str) {
    let fixture =
        include_str!("../../../../tools/oracle/tkernel_mmgr_opt_callback_unwind.expected");
    assert!(
        fixture.lines().any(|row| row == actual),
        "missing native row: {actual}"
    );
}

/// Installs a callback that replaces itself; `install_replacing_callback()`.
pub fn install_replacing_callback() {
    // The next callback writes the global callback slot from inside notification itself.
    MMgrOpt::set_call_back_function(Some(replace_callback));
}

fn replace_callback(allocate: bool, storage: StorageAddress, round: usize, size: usize) {
    observe(allocate, storage, round, size);
    MMgrOpt::set_call_back_function(Some(replacement_callback));
}

fn replacement_callback(allocate: bool, storage: StorageAddress, round: usize, size: usize) {
    observe(allocate, storage, round, size);
    TRACE.with(|trace| trace.borrow_mut().replaced += 1);
}

/// Counts calls reaching the replacement callback; `replaced_count() == 1`.
pub fn replaced_count() -> usize {
    // Separate identity from ordinary event arguments so callback replacement is observable.
    TRACE.with(|trace| trace.borrow().replaced)
}
