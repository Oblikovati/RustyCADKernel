//! Error handler cases (tkernel_standard_runtime_handler_probe.cxx).
use super::bit;
use crate::common::{guarded, next, next_bytes};
use rustycad_tkernel::standard::error_handler::SIGNAL_EXCEPTION_TYPES;
use rustycad_tkernel::standard::{ErrorHandler, ErrorHandlerCallback, Failure};
use std::{cell::RefCell, rc::Rc, str::SplitWhitespace};

/// The objects a handler script creates, alive until the script ends.
#[derive(Default)]
struct HandlerScript {
    log: Rc<RefCell<String>>,
    handlers: Vec<ErrorHandler>,
    callbacks: Vec<ErrorHandlerCallback>,
}
impl HandlerScript {
    /// A callback that logs its DestroyCallback() calls as "cb<index>", as `RuntimeProbeCallback`.
    fn add_callback(&mut self) {
        let (log, index) = (self.log.clone(), self.callbacks.len());
        self.callbacks.push(ErrorHandlerCallback::new(move || {
            log.borrow_mut().push_str(&format!("cb{index},"))
        }));
    }
    /// One operation, as the native switch.
    fn run(&mut self, op: &str) {
        let index: usize = op
            .get(1..)
            .and_then(|digits| digits.parse().ok())
            .unwrap_or(0);
        match op.as_bytes()[0] {
            b'h' => self.handlers.push(ErrorHandler::new()),
            b'D' => self.handlers[index].destroy(),
            b'c' => self.add_callback(),
            b'R' => self.callbacks[index].register_callback(),
            b'U' => self.callbacks[index].unregister_callback(),
            b't' => self.log_try_block(),
            _ => panic!("unknown handler op {op:?}"),
        }
    }
    fn log_try_block(&self) {
        let in_try = bit(ErrorHandler::is_in_try_block());
        self.log.borrow_mut().push_str(&format!("T{in_try},"));
    }
    /// Callbacks go first, then handlers newest first, as the native teardown.
    fn finish(mut self) -> String {
        self.callbacks.clear();
        while self.handlers.pop().is_some() {}
        let text = self.log.borrow().clone();
        let text = if text.is_empty() { "-".into() } else { text };
        format!("{text} {}", bit(ErrorHandler::is_in_try_block()))
    }
}
/// "runtime_handler_script <op>...".
pub fn handler_script_case(input: &mut SplitWhitespace<'_>) -> String {
    let mut script = HandlerScript::default();
    for op in input.by_ref() {
        script.run(op);
    }
    script.finish()
}
/// Class name of an `Error()` value, "none" for the empty state.
fn error_name(error: Option<Failure>) -> String {
    error.map_or("none".into(), |failure| failure.exception_type().into())
}
/// The innermost handler's landing: Error() before Raise(), then what Raise() returns.
fn land_abort(error: Failure) -> String {
    let handler = ErrorHandler::new();
    // The `?` path that stands for the longjmp: Abort stores the failure and returns it.
    let aborted = ErrorHandler::abort(error).map(|_| ());
    assert!(aborted.is_err(), "Abort must not return normally");
    let caught = error_name(handler.error());
    let raised = handler.raise().map(|_| "not raised".to_string());
    format!("{caught} {}", guarded(raised))
}
/// "runtime_abort <Class> <hex message> <depth>".
pub fn abort_case(input: &mut SplitWhitespace<'_>) -> String {
    let class = input.next().expect("class");
    let exception_type = SIGNAL_EXCEPTION_TYPES
        .into_iter()
        .find(|name| *name == class)
        .unwrap_or_else(|| panic!("{class:?} is not a signal exception"));
    let message = String::from_utf8(next_bytes(input)).expect("UTF-8 message");
    let depth: i32 = next(input);
    let outer: Vec<ErrorHandler> = (1..depth).map(|_| ErrorHandler::new()).collect();
    let landed = land_abort(Failure::with_message(exception_type, &message));
    let outer_error = outer
        .last()
        .map_or("-".into(), |handler| error_name(handler.error()));
    let in_try = bit(ErrorHandler::is_in_try_block());
    drop(outer);
    format!(
        "{landed} {outer_error} {in_try}{}",
        bit(ErrorHandler::is_in_try_block())
    )
}
