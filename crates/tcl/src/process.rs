//! Process-wide calls: environment, exit, exit handlers, static packages, panic.
use crate::text::to_c_string;
use crate::{Completion, InterpHandle, TclError, sys};
use std::ffi::c_int;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr::NonNull;

/// `Tcl_PutEnv("NAME=value")`: sets a variable in the process environment and Tcl's
/// `env` array; returns false when Tcl rejects the assignment.
///
/// ```
/// assert!(rustycad_tcl::put_env("RUSTYCAD_TCL_DOC=1").unwrap());
/// ```
pub fn put_env(assignment: &str) -> Result<bool, TclError> {
    crate::initialize_library();
    let assignment = to_c_string(assignment)?;
    // SAFETY: a live C string; Tcl copies it into the environment it manages.
    Ok(unsafe { sys::Tcl_PutEnv(assignment.as_ptr()) } == 0)
}

/// `Tcl_Exit(status)`: runs the exit handlers, finalizes Tcl and ends the process.
pub fn exit(status: c_int) -> ! {
    crate::initialize_library();
    // SAFETY: ends the process after Tcl's exit handlers.
    unsafe { sys::Tcl_Exit(status) }
}

/// `Tcl_Panic("%s", message)`: Tcl's fatal error path; prints and aborts.
pub fn panic_abort(message: &str) -> ! {
    let text = to_c_string(&message.replace('\0', "\\0")).unwrap_or_default();
    // SAFETY: a "%s" format with one live C string argument.
    unsafe { sys::Tcl_Panic(c"%s".as_ptr(), text.as_ptr()) }
}

/// The body of an exit handler; it runs once, on the thread that calls `Tcl_Exit`.
pub type ExitBody = Box<dyn FnOnce() + Send>;

/// `Tcl_CreateExitHandler` with a Rust body (leaked if the process never exits via Tcl).
///
/// ```
/// rustycad_tcl::create_exit_handler(Box::new(|| {}));
/// ```
pub fn create_exit_handler(body: ExitBody) {
    crate::initialize_library();
    let record: *mut ExitBody = Box::into_raw(Box::new(body));
    // SAFETY: Tcl calls the trampoline at most once with this record.
    unsafe { sys::Tcl_CreateExitHandler(Some(exit_trampoline), record.cast()) };
}

/// `Tcl_ExitProc` of every Rust exit handler.
///
/// # Safety
/// Called once by Tcl with a record from `create_exit_handler`.
unsafe extern "C" fn exit_trampoline(client_data: sys::ClientData) {
    // SAFETY: the record is used exactly once, here.
    let body = unsafe { Box::from_raw(client_data.cast::<ExitBody>()) };
    if catch_unwind(AssertUnwindSafe(body)).is_err() {
        eprintln!("Rust panic in a Tcl exit handler");
    }
}

/// A package linked into the program; `Tcl_StaticPackage` makes `load {} Name` call it.
pub trait StaticPackage {
    /// The package initializer (`Name_Init`).
    fn init(interp: &InterpHandle) -> Completion;
}

/// `Tcl_PackageInitProc` for package `P`.
///
/// # Safety
/// Called by Tcl with a live interpreter.
unsafe extern "C" fn package_trampoline<P: StaticPackage>(interp: *mut sys::Tcl_Interp) -> c_int {
    let Some(raw) = NonNull::new(interp) else {
        return sys::TCL_ERROR;
    };
    // SAFETY: live interpreter for the call.
    let handle = unsafe { InterpHandle::from_raw(raw) };
    catch_unwind(AssertUnwindSafe(|| P::init(&handle)))
        .unwrap_or(Completion::Error)
        .code()
}

impl InterpHandle {
    /// `Tcl_StaticPackage(interp, prefix, P::init, NULL)`: no safe-interpreter initializer.
    pub fn static_package<P: StaticPackage>(&self, prefix: &str) -> Result<(), TclError> {
        let prefix = to_c_string(prefix)?;
        // SAFETY: live interpreter; Tcl copies the prefix.
        unsafe {
            sys::Tcl_StaticPackage(
                self.as_ptr(),
                prefix.as_ptr(),
                Some(package_trampoline::<P>),
                None,
            )
        };
        Ok(())
    }
}
