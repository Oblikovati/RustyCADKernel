//! `CommandCmd`: what happens around every DRAW command call — console capture into the
//! log, echo, Ctrl-Break, exception reporting and `Draw_ExitOnCatch`.
use super::registration::DrawCommand;
use super::{DrawInterpretor, InterpretorCore};
use crate::draw::stdio_capture::{
    STDERR_DESCRIPTOR, STDOUT_DESCRIPTOR, capture_end, capture_start,
};
use rustycad_tcl::{Completion, InterpHandle, sys};
use rustycad_tkernel::message::Message;
use rustycad_tkernel::osd::Osd;
use rustycad_tkernel::standard::Failure;
use std::fmt::Write;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Weak;

/// How a command call ended, before it becomes a Tcl completion code.
enum CallOutcome {
    /// The command returned this value (0 = success).
    Returned(i32),
    /// The command raised an OCCT exception (`Standard_Failure`).
    Failed(Failure),
    /// The command panicked (the C++ `std::exception` path).
    Panicked(String),
}

/// Runs `command` as `CommandCmd` does and returns the Tcl completion.
pub(super) fn run_command(
    core: &Weak<InterpretorCore>,
    command: &dyn DrawCommand,
    handle: &InterpHandle,
    args: &[&str],
) -> Completion {
    let Some(core) = core.upgrade() else {
        let _ = handle.set_result("the Draw interpretor of this command was dropped");
        return Completion::Error;
    };
    let di = DrawInterpretor::from_core(core);
    // Commands that manipulate the log and the echo are never logged nor echoed.
    let is_log_manipulation = matches!(args.first(), Some(&"dlog") | Some(&"decho"));
    let do_log = di.get_do_log() && !is_log_manipulation;
    let do_echo = di.get_do_echo() && !is_log_manipulation;
    di.console().flush();
    let mut saved = start_log_capture(&di, do_log);
    if do_echo || do_log {
        di.console().write_out(&dump_args(args));
    }
    let completion = report_outcome(&di, handle, invoke(&di, command, args));
    if do_log || do_echo {
        echo_result(&di, handle);
    }
    di.console().flush();
    if do_log {
        capture_end(STDERR_DESCRIPTOR, &mut saved[1]);
        capture_end(STDOUT_DESCRIPTOR, &mut saved[0]);
    }
    completion
}

/// `capture_start` of standard output then standard error into the log descriptor.
fn start_log_capture(di: &DrawInterpretor, do_log: bool) -> [Option<i32>; 2] {
    let Some(log) = di.log_file_descriptor().filter(|_| do_log) else {
        return [None, None];
    };
    [
        capture_start(STDOUT_DESCRIPTOR, log),
        capture_start(STDERR_DESCRIPTOR, log),
    ]
}

/// `dumpArgs`: every word followed by a space, then a newline.
fn dump_args(args: &[&str]) -> String {
    let mut text: String = args.iter().map(|word| format!("{word} ")).collect();
    text.push('\n');
    text
}

/// Prints a non-empty result on its own line (logging or echo).
fn echo_result(di: &DrawInterpretor, handle: &InterpHandle) {
    let result = handle.result();
    if !result.is_empty() {
        di.console().write_out(&format!("{result}\n"));
    }
}

/// Checks Ctrl-Break (`OSD::ControlBreak`) then runs the command, catching panics.
fn invoke(di: &DrawInterpretor, command: &dyn DrawCommand, args: &[&str]) -> CallOutcome {
    let call = catch_unwind(AssertUnwindSafe(|| {
        Osd::control_break()?;
        command.invoke(di, args)
    }));
    match call {
        Ok(Ok(code)) => CallOutcome::Returned(code),
        Ok(Err(failure)) => CallOutcome::Failed(failure),
        Err(panic) => CallOutcome::Panicked(panic_text(&*panic)),
    }
}

/// The message of a panic payload.
fn panic_text(panic: &dyn std::any::Any) -> String {
    let text = panic.downcast_ref::<String>().cloned();
    text.or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_else(|| "non-string panic payload".to_string())
}

/// Turns the outcome into a completion; an exception becomes the result text, after
/// `Draw_ExitOnCatch` had its chance to end the session.
fn report_outcome(di: &DrawInterpretor, handle: &InterpHandle, outcome: CallOutcome) -> Completion {
    let description = match outcome {
        CallOutcome::Returned(0) => return Completion::Ok,
        CallOutcome::Returned(_) => return Completion::Error,
        CallOutcome::Failed(failure) => failure.to_string(),
        // Panics take the C++ `std::exception` path: what() and the type name in brackets.
        CallOutcome::Panicked(message) => format!("{message} [rust panic]"),
    };
    exit_on_catch(di, handle, &description);
    let _ = handle.set_result(&format!("An exception was caught {description}"));
    Completion::Error
}

/// When the Tcl variable `Draw_ExitOnCatch` is a non-zero number, reports the exception
/// and evaluates `exit` (non-Windows; Windows calls `Tcl_Exit(0)`).
fn exit_on_catch(di: &DrawInterpretor, handle: &InterpHandle, description: &str) {
    let flag = handle
        .get_var("Draw_ExitOnCatch", sys::TCL_GLOBAL_ONLY)
        .ok()
        .flatten();
    if !flag.is_some_and(|text| di.parse_integer(&text) != 0) {
        return;
    }
    let _ = write!(
        Message::send_fail(),
        "An exception was caught {description}"
    );
    if cfg!(windows) {
        rustycad_tcl::exit(0);
    }
    let _ = handle.eval("exit");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments_are_dumped_with_trailing_spaces() {
        assert_eq!(dump_args(&["cmd", "a"]), "cmd a \n");
        assert_eq!(dump_args(&[]), "\n");
    }

    #[test]
    fn panic_payloads_become_text() {
        assert_eq!(panic_text(&String::from("x")), "x");
        assert_eq!(panic_text(&"y"), "y");
        assert_eq!(panic_text(&5), "non-string panic payload");
    }
}
