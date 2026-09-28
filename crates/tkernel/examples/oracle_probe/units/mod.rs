//! Units, UnitsAPI and UnitsMethods probe operations (twin of `tkernel_units_probe.cxx`).
//!
//! One units context, one `UnitsApi`, one `UnitsMethods` and one units system live for
//! the whole run, as the native statics do, so every request sees the state the
//! previous ones left. Console text and messages are captured per request.
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::units::{Units, UnitsSystem};
use rustycad_tkernel::units_api::UnitsApi;
use rustycad_tkernel::units_methods::UnitsMethods;
use std::cell::RefCell;
use std::rc::Rc;
use std::str::SplitWhitespace;
use std::sync::Arc;

mod api;
mod common;
mod core;
mod objects;
use common::{Capture, EmptyEnvironment, NoResourceIo};

/// The process-lifetime state mirrored from the native statics.
pub(crate) struct UnitsProbeState {
    pub capture: Capture,
    pub units: Arc<Units>,
    pub api: UnitsApi,
    pub methods: UnitsMethods,
    pub system: Handle<UnitsSystem>,
}

thread_local! {
    static STATE: RefCell<Option<UnitsProbeState>> = const { RefCell::new(None) };
}

fn state() -> UnitsProbeState {
    let capture = Capture::default();
    let units = Arc::new(Units::with_diagnostics(Box::new(capture.clone())));
    let api = UnitsApi::new(
        units.clone(),
        Rc::new(NoResourceIo),
        Rc::new(EmptyEnvironment),
    );
    UnitsProbeState {
        capture,
        units,
        api,
        methods: UnitsMethods::default(),
        system: Handle::new(UnitsSystem::new()),
    }
}

/// The output of a units operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    STATE.with(|cell| {
        let mut slot = cell.borrow_mut();
        let state = slot.get_or_insert_with(state);
        core::evaluate(op, input, state)
            .or_else(|| objects::evaluate(op, input, state))
            .or_else(|| api::evaluate(op, input, state))
    })
}

/// Runs one request body and appends the captured console and messages, like the
/// native `units_run`.
pub(crate) fn run(
    state: &mut UnitsProbeState,
    body: impl FnOnce(&mut UnitsProbeState, &mut String) -> Result<(), Failure>,
) -> String {
    state.capture.clear();
    let mut result = String::new();
    if let Err(failure) = body(state, &mut result) {
        result.push_str(&format!(
            "EXC {} {}",
            failure.exception_type(),
            common::hex(failure.message().as_bytes())
        ));
    }
    format!(
        "{result} cout={} msg={}",
        common::hex(&common::mask(&state.capture.console())),
        common::hex(&state.capture.messages())
    )
}
