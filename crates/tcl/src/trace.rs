//! Variable traces implemented in Rust (`Tcl_TraceVar`, `Tcl_UntraceVar`,
//! `Tcl_VarTraceInfo`).
//!
//! Every Rust trace shares one C trampoline, so a trace carries a `class` tag (the
//! identity a C caller expresses with the trace procedure pointer) and a `key` (the
//! client data a C caller stores, e.g. the address of an OCCT drawable).
//! [`InterpHandle::var_trace_key`] is `Tcl_VarTraceInfo` restricted to one class.
//!
//! Ownership: Tcl holds one `Rc` reference to the record. It is released exactly once,
//! either by [`InterpHandle::untrace_var`] or when Tcl destroys the trace
//! (`TCL_TRACE_DESTROYED`, on unset or interpreter deletion). The wrapper always adds
//! `TCL_TRACE_UNSETS` so that Tcl reports the destruction; unset events the caller did
//! not ask for are not passed to the body. The trampoline holds its own reference while
//! the body runs, so a body that removes its own trace stays alive.
use crate::text::{from_c_pointer, to_c_string};
use crate::{InterpHandle, TclError, sys};
use std::cell::Cell;
use std::ffi::{c_char, c_int};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr::NonNull;
use std::rc::Rc;

/// What a trace body is told about one event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEvent {
    /// The variable (or array) name as Tcl passes it.
    pub part1: String,
    /// The array element, if the event concerns one.
    pub part2: Option<String>,
    /// `TCL_TRACE_WRITES`, `TCL_TRACE_UNSETS`, `TCL_TRACE_DESTROYED`, ... as Tcl passes them.
    pub flags: c_int,
}

/// A trace body; returning `Some(message)` makes the traced operation fail with it.
pub trait VarTraceProc {
    /// Handles one trace event.
    fn on_event(&self, interp: &InterpHandle, event: &TraceEvent) -> Option<String>;
}

impl<F: Fn(&InterpHandle, &TraceEvent) -> Option<String>> VarTraceProc for F {
    fn on_event(&self, interp: &InterpHandle, event: &TraceEvent) -> Option<String> {
        self(interp, event)
    }
}

/// Identifies one Rust trace: its class tag and its key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TraceTag {
    /// The class tag, standing for the C trace procedure.
    pub class: &'static str,
    /// The per-trace value, standing for the C client data.
    pub key: u64,
}

struct TraceRecord {
    tag: TraceTag,
    requested: c_int,
    registered: c_int,
    attached: Cell<bool>,
    body: Box<dyn VarTraceProc>,
}

/// Flags whose presence must match for `Tcl_UntraceVar` to find a trace.
const TRACE_KIND_MASK: c_int = sys::TCL_TRACE_READS | sys::TCL_TRACE_WRITES | sys::TCL_TRACE_UNSETS;

impl InterpHandle {
    /// `Tcl_TraceVar` with a Rust body; `flags` selects the events and the scope
    /// (`TCL_GLOBAL_ONLY`). Returns false when Tcl refused (e.g. a bad array name).
    ///
    /// ```
    /// use rustycad_tcl::{Interp, TraceTag, sys};
    /// let interp = Interp::new();
    /// let tag = TraceTag { class: "doc", key: 7 };
    /// let refuse = Box::new(|_: &rustycad_tcl::InterpHandle, _: &rustycad_tcl::TraceEvent| {
    ///     Some("read only".to_string())
    /// });
    /// assert!(interp.trace_var("x", sys::TCL_TRACE_WRITES, tag, refuse).unwrap());
    /// assert_eq!(interp.var_trace_key("x", 0, "doc").unwrap(), Some(7));
    /// ```
    pub fn trace_var(
        &self,
        name: &str,
        flags: c_int,
        tag: TraceTag,
        body: Box<dyn VarTraceProc>,
    ) -> Result<bool, TclError> {
        let c_name = to_c_string(name)?;
        let registered = flags | sys::TCL_TRACE_UNSETS | sys::TCL_TRACE_RESULT_DYNAMIC;
        let record = Rc::new(TraceRecord {
            tag,
            requested: flags & TRACE_KIND_MASK,
            registered,
            attached: Cell::new(true),
            body,
        });
        let client_data = Rc::into_raw(record).cast_mut().cast();
        // SAFETY: live interpreter and name; `client_data` is one `Rc` reference that the
        // trampoline or `untrace_var` releases once (released below if Tcl refuses).
        let code = unsafe {
            sys::Tcl_TraceVar(
                self.as_ptr(),
                c_name.as_ptr(),
                registered,
                Some(trace_trampoline),
                client_data,
            )
        };
        if code != sys::TCL_OK {
            // SAFETY: Tcl did not keep the reference.
            drop(unsafe { Rc::from_raw(client_data.cast_const().cast::<TraceRecord>()) });
        }
        Ok(code == sys::TCL_OK)
    }

    /// `Tcl_VarTraceInfo` for one class: the key of the first trace of `class` on `name`.
    pub fn var_trace_key(
        &self,
        name: &str,
        flags: c_int,
        class: &str,
    ) -> Result<Option<u64>, TclError> {
        let found = self.find_trace(name, flags, |tag| tag.class == class)?;
        // SAFETY: `find_trace` returns live records.
        Ok(found.map(|record| unsafe { &*record.as_ptr() }.tag.key))
    }

    /// `Tcl_UntraceVar` of the trace `tag` on `name`; true when it was found.
    pub fn untrace_var(&self, name: &str, flags: c_int, tag: TraceTag) -> Result<bool, TclError> {
        let Some(record) = self.find_trace(name, flags, |found| *found == tag)? else {
            return Ok(false);
        };
        let c_name = to_c_string(name)?;
        // SAFETY: the record is alive: Tcl still holds its reference.
        let record_ref = unsafe { &*record.as_ptr() };
        let scope = flags & (sys::TCL_GLOBAL_ONLY | sys::TCL_NAMESPACE_ONLY);
        // SAFETY: live interpreter; flags and client data identify the trace exactly.
        unsafe {
            sys::Tcl_UntraceVar(
                self.as_ptr(),
                c_name.as_ptr(),
                record_ref.registered | scope,
                Some(trace_trampoline),
                record.as_ptr().cast(),
            )
        };
        // SAFETY: the trace is gone from Tcl; its reference is released once.
        unsafe { release_tcl_reference(record.as_ptr()) };
        Ok(true)
    }

    /// Walks the Rust traces on `name` and returns the first record accepted by `matches`.
    fn find_trace(
        &self,
        name: &str,
        flags: c_int,
        matches: impl Fn(&TraceTag) -> bool,
    ) -> Result<Option<NonNull<TraceRecord>>, TclError> {
        let c_name = to_c_string(name)?;
        let mut previous: sys::ClientData = std::ptr::null_mut();
        loop {
            // SAFETY: live interpreter; `previous` is null or client data Tcl returned.
            previous = unsafe {
                sys::Tcl_VarTraceInfo(
                    self.as_ptr(),
                    c_name.as_ptr(),
                    flags,
                    Some(trace_trampoline),
                    previous,
                )
            };
            let Some(record) = NonNull::new(previous.cast::<TraceRecord>()) else {
                return Ok(None);
            };
            // SAFETY: every client data of `trace_trampoline` is a live record.
            if matches(&unsafe { record.as_ref() }.tag) {
                return Ok(Some(record));
            }
        }
    }
}

/// Releases Tcl's reference to `record` unless that already happened.
///
/// # Safety
/// `record` must be alive (the caller or Tcl holds a reference).
unsafe fn release_tcl_reference(record: *mut TraceRecord) {
    // SAFETY: alive per the contract.
    let attached = unsafe { &(*record).attached };
    if attached.replace(false) {
        // SAFETY: this is the one release of Tcl's reference.
        drop(unsafe { Rc::from_raw(record.cast_const()) });
    }
}

/// `Tcl_VarTraceProc` for every Rust trace.
///
/// # Safety
/// Called by Tcl with the client data of `trace_var` (a live record), a live
/// interpreter and C strings (`part2` may be null).
unsafe extern "C" fn trace_trampoline(
    client_data: sys::ClientData,
    interp: *mut sys::Tcl_Interp,
    part1: *const c_char,
    part2: *const c_char,
    flags: c_int,
) -> *mut c_char {
    let pointer: *const TraceRecord = client_data.cast_const().cast();
    // SAFETY: Tcl holds a reference; this call takes its own for its duration.
    let record = unsafe {
        Rc::increment_strong_count(pointer);
        Rc::from_raw(pointer)
    };
    // SAFETY: live C strings for the call; `part2` may be null.
    let event = unsafe {
        TraceEvent {
            part1: from_c_pointer(part1).unwrap_or_default(),
            part2: from_c_pointer(part2),
            flags,
        }
    };
    // SAFETY: Tcl passes its live interpreter.
    let message = unsafe { run_body(&record, interp, &event) };
    if flags & sys::TCL_TRACE_DESTROYED != 0 {
        // SAFETY: `record` keeps the allocation alive; Tcl drops the trace after return.
        unsafe { release_tcl_reference(pointer.cast_mut()) };
    }
    message.map_or(std::ptr::null_mut(), |text| tcl_allocated_copy(&text))
}

/// Runs the body when the event is one the caller asked for; a panic becomes a message.
///
/// # Safety
/// `interp` must be the live interpreter of the event.
unsafe fn run_body(
    record: &TraceRecord,
    interp: *mut sys::Tcl_Interp,
    event: &TraceEvent,
) -> Option<String> {
    if event.flags & record.requested == 0 {
        return None;
    }
    let raw = NonNull::new(interp)?;
    // SAFETY: Tcl passes a live interpreter for the call.
    let handle = unsafe { InterpHandle::from_raw(raw) };
    catch_unwind(AssertUnwindSafe(|| record.body.on_event(&handle, event)))
        .unwrap_or_else(|_| Some(format!("Rust panic in variable trace {:?}", record.tag)))
}

/// Copies `text` into memory from `Tcl_Alloc`, which Tcl frees (`TCL_TRACE_RESULT_DYNAMIC`).
fn tcl_allocated_copy(text: &str) -> *mut c_char {
    let bytes = text.replace('\0', "\\0").into_bytes();
    let size = std::ffi::c_uint::try_from(bytes.len() + 1).unwrap_or(std::ffi::c_uint::MAX);
    let length = (size as usize).saturating_sub(1);
    // SAFETY: Tcl_Alloc returns `size` writable bytes or panics the process.
    let buffer = unsafe { sys::Tcl_Alloc(size) };
    // SAFETY: `length + 1 <= size` bytes are written into the fresh buffer.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), buffer, length);
        *buffer.add(length) = 0;
    }
    buffer
}
