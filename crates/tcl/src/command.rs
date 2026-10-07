//! String-based commands implemented in Rust (`Tcl_CreateCommand`, `Tcl_DeleteCommand`).
//!
//! Each command owns an `Rc<CommandRecord>` passed to Tcl as client data. The trampoline
//! takes its own strong reference for the duration of a call, so `Tcl_DeleteCommand`
//! issued from inside the command (Tcl runs the delete procedure at once) cannot free the
//! closure that is still executing. A Rust panic is caught at the C boundary and becomes
//! a `TCL_ERROR` whose result names the command and the panic message.
use crate::text::{decode_tcl_utf8, to_c_string};
use crate::{Completion, InterpHandle, TclError, sys};
use std::borrow::Cow;
use std::ffi::{CStr, c_char, c_int};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr::NonNull;
use std::rc::Rc;

/// A command body: receives the interpreter and the words (`args[0]` is the command name).
pub trait CommandProc {
    /// Runs the command; `TCL_OK`/`TCL_ERROR`/... become the completion code.
    fn invoke(&self, interp: &InterpHandle, args: &[&str]) -> Completion;
}

impl<F: Fn(&InterpHandle, &[&str]) -> Completion> CommandProc for F {
    fn invoke(&self, interp: &InterpHandle, args: &[&str]) -> Completion {
        self(interp, args)
    }
}

struct CommandRecord {
    name: String,
    body: Box<dyn CommandProc>,
}

impl InterpHandle {
    /// `Tcl_CreateCommand`: defines (or replaces) command `name`.
    ///
    /// ```
    /// use rustycad_tcl::{Completion, Interp, InterpHandle};
    /// let interp = Interp::new();
    /// interp.create_command("twice", Box::new(|i: &InterpHandle, a: &[&str]| {
    ///     i.set_result(&format!("{0}{0}", a[1])).unwrap();
    ///     Completion::Ok
    /// })).unwrap();
    /// interp.eval("twice ab").unwrap();
    /// assert_eq!(interp.result(), "abab");
    /// ```
    pub fn create_command(&self, name: &str, body: Box<dyn CommandProc>) -> Result<(), TclError> {
        let c_name = to_c_string(name)?;
        let record = Rc::new(CommandRecord {
            name: name.to_string(),
            body,
        });
        let client_data = Rc::into_raw(record).cast_mut().cast();
        // SAFETY: live interpreter and name; `client_data` is an `Rc<CommandRecord>`
        // released exactly once by `delete_trampoline`.
        unsafe {
            sys::Tcl_CreateCommand(
                self.as_ptr(),
                c_name.as_ptr(),
                Some(command_trampoline),
                client_data,
                Some(delete_trampoline),
            )
        };
        Ok(())
    }

    /// `Tcl_DeleteCommand`: true when the command existed.
    pub fn delete_command(&self, name: &str) -> Result<bool, TclError> {
        let name = to_c_string(name)?;
        // SAFETY: live interpreter and C string.
        Ok(unsafe { sys::Tcl_DeleteCommand(self.as_ptr(), name.as_ptr()) } == 0)
    }
}

/// `Tcl_CmdProc` for every Rust command.
///
/// # Safety
/// Called by Tcl with the client data given to `Tcl_CreateCommand` (a live
/// `Rc<CommandRecord>`), a live interpreter and `argc` C strings in `argv`.
unsafe extern "C" fn command_trampoline(
    client_data: sys::ClientData,
    interp: *mut sys::Tcl_Interp,
    argc: c_int,
    argv: *mut *const c_char,
) -> c_int {
    let pointer: *const CommandRecord = client_data.cast_const().cast();
    // SAFETY: the record is alive (Tcl holds one reference); this call takes its own.
    let record = unsafe {
        Rc::increment_strong_count(pointer);
        Rc::from_raw(pointer)
    };
    // SAFETY: Tcl passes its live interpreter for the duration of the call.
    let handle = unsafe { InterpHandle::from_raw(NonNull::new_unchecked(interp)) };
    // SAFETY: Tcl passes `argc` valid C strings.
    let words = unsafe { collect_words(argc, argv) };
    let args: Vec<&str> = words.iter().map(Cow::as_ref).collect();
    let outcome = catch_unwind(AssertUnwindSafe(|| record.body.invoke(&handle, &args)));
    outcome
        .unwrap_or_else(|panic| report_panic(&handle, &record.name, &*panic))
        .code()
}

/// Copies the command words.
///
/// # Safety
/// `argv` must hold `argc` live NUL-terminated strings.
unsafe fn collect_words<'a>(argc: c_int, argv: *mut *const c_char) -> Vec<Cow<'a, str>> {
    let count = usize::try_from(argc).unwrap_or(0);
    (0..count)
        .map(|index| {
            // SAFETY: `index < argc`, and each entry is a live C string for the call.
            let word = unsafe { CStr::from_ptr(*argv.add(index)) };
            decode_tcl_utf8(word.to_bytes())
        })
        .collect()
}

/// Turns a caught panic into a `TCL_ERROR` result naming the command and the message.
fn report_panic(handle: &InterpHandle, name: &str, panic: &dyn std::any::Any) -> Completion {
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("non-string panic payload");
    let text = format!("Rust panic in command \"{name}\": {message}").replace('\0', "\\0");
    // The text has no NUL (replaced above), so setting it cannot fail.
    let _ = handle.set_result(&text);
    Completion::Error
}

/// `Tcl_CmdDeleteProc`: releases Tcl's reference to the record.
///
/// # Safety
/// Called once by Tcl with the client data of `create_command`.
unsafe extern "C" fn delete_trampoline(client_data: sys::ClientData) {
    // SAFETY: Tcl owned exactly one reference, released here exactly once.
    drop(unsafe { Rc::from_raw(client_data.cast_const().cast::<CommandRecord>()) });
}
