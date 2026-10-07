//! Integer macros of `tcl.h` used by the binding; the header test asserts each value with
//! `_Static_assert(NAME == value)` against the installed header.
use super::signature::foreign_constants;
use super::types::Tcl_FreeProcPointer;
use std::ffi::c_int;

foreign_constants! {
    CONSTANT_VALUES;
    /// Completion code: success.
    TCL_OK: c_int = 0;
    /// Completion code: error, the result holds the message.
    TCL_ERROR: c_int = 1;
    /// Completion code of `return`.
    TCL_RETURN: c_int = 2;
    /// Completion code of `break`.
    TCL_BREAK: c_int = 3;
    /// Completion code of `continue`.
    TCL_CONTINUE: c_int = 4;
    /// Variable flag: global scope only.
    TCL_GLOBAL_ONLY: c_int = 1;
    /// Variable flag: current namespace only.
    TCL_NAMESPACE_ONLY: c_int = 2;
    /// Variable flag: append to the old value.
    TCL_APPEND_VALUE: c_int = 4;
    /// Variable flag: quote the value as a list element.
    TCL_LIST_ELEMENT: c_int = 8;
    /// Trace flag: reads.
    TCL_TRACE_READS: c_int = 0x10;
    /// Trace flag: writes.
    TCL_TRACE_WRITES: c_int = 0x20;
    /// Trace flag: unsets.
    TCL_TRACE_UNSETS: c_int = 0x40;
    /// Trace callback flag: the trace is being removed.
    TCL_TRACE_DESTROYED: c_int = 0x80;
    /// Trace callback flag: the interpreter is being deleted.
    TCL_INTERP_DESTROYED: c_int = 0x100;
    /// Variable flag: leave an error message in the result on failure.
    TCL_LEAVE_ERR_MSG: c_int = 0x200;
    /// Trace flag: the returned message was allocated with `Tcl_Alloc`; Tcl frees it.
    TCL_TRACE_RESULT_DYNAMIC: c_int = 0x8000;
    /// `Tcl_GetStdChannel` selector: standard input.
    TCL_STDIN: c_int = 1 << 1;
    /// `Tcl_GetStdChannel` selector: standard output.
    TCL_STDOUT: c_int = 1 << 2;
    /// `Tcl_GetStdChannel` selector: standard error.
    TCL_STDERR: c_int = 1 << 3;
    /// Channel/file mask: readable.
    TCL_READABLE: c_int = 1 << 1;
    /// Channel/file mask: writable.
    TCL_WRITABLE: c_int = 1 << 2;
    /// Channel/file mask: exception.
    TCL_EXCEPTION: c_int = 1 << 3;
    /// `Tcl_DoOneEvent` flag: do not block.
    TCL_DONT_WAIT: c_int = 1 << 1;
    /// `Tcl_DoOneEvent` flag: window events.
    TCL_WINDOW_EVENTS: c_int = 1 << 2;
    /// `Tcl_DoOneEvent` flag: file events.
    TCL_FILE_EVENTS: c_int = 1 << 3;
    /// `Tcl_DoOneEvent` flag: timer events.
    TCL_TIMER_EVENTS: c_int = 1 << 4;
    /// `Tcl_DoOneEvent` flag: idle callbacks.
    TCL_IDLE_EVENTS: c_int = 1 << 5;
    /// `Tcl_DoOneEvent` flag: every event type.
    TCL_ALL_EVENTS: c_int = !TCL_DONT_WAIT;
    /// `Tcl_RecordAndEval` flag: record the command without evaluating it.
    TCL_NO_EVAL: c_int = 0x10000;
    /// `Tcl_RecordAndEval` flag: evaluate at global level.
    TCL_EVAL_GLOBAL: c_int = 0x20000;
}

/// `TCL_STATIC`: the result string is static, Tcl keeps the pointer.
pub const TCL_STATIC: Tcl_FreeProcPointer = std::ptr::null();
/// `TCL_VOLATILE` (`(Tcl_FreeProc *) 1`): Tcl copies the result string immediately.
pub const TCL_VOLATILE: Tcl_FreeProcPointer = std::ptr::without_provenance(1);
