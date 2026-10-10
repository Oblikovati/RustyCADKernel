//! The interpreter: an owning [`Interp`] and the borrowed [`InterpHandle`] that every
//! operation and every callback uses.
//!
//! Soundness rests on three facts. A Tcl interpreter belongs to the thread that created
//! it, so neither type is `Send` or `Sync`. Tcl strings returned by the C API (results,
//! variable values) are invalidated by later calls, so every getter copies them into an
//! owned `String` before returning. Tcl preserves an interpreter that is deleted while
//! it evaluates (`Tcl_Preserve`), so a handle given to a callback stays valid for the
//! whole callback.
use crate::text::{from_c_pointer, to_c_string};
use crate::{Completion, TclError, sys};
use std::ffi::c_int;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::Once;

/// Calls `Tcl_FindExecutable` once per process, which initializes the encoding system
/// and the library path search, then initializes the calling thread’s notifier.
/// Call `initialize_library()` before using interpreter-free Tcl events.
pub fn initialize_library() {
    static INITIALIZED: Once = Once::new();
    INITIALIZED.call_once(|| {
        let executable = std::env::current_exe().ok();
        let argv0 = executable.and_then(|path| to_c_string(&path.to_string_lossy()).ok());
        let pointer = argv0
            .as_ref()
            .map_or(std::ptr::null(), |text| text.as_ptr());
        // SAFETY: `pointer` is null (allowed) or a live C string; Tcl copies it.
        unsafe { sys::Tcl_FindExecutable(pointer) };
    });
    initialize_thread();
}

fn initialize_thread() {
    thread_local! {
        // Tcl 8.6 TclInitNotifier is reached by Tcl_CreateInterp on each foreign thread.
        // FindExecutable's process-wide initialization alone leaves macOS CFRunLoop unset.
        static NOTIFIER_READY: () = {
            // SAFETY: this temporary interpreter never escapes its owning thread. Its
            // deletion leaves the thread's notifier ready for interpreter-free events.
            unsafe {
                let interpreter = sys::Tcl_CreateInterp();
                sys::Tcl_DeleteInterp(interpreter);
            }
        };
    }
    NOTIFIER_READY.with(|()| ());
}

/// A non-owning reference to a live interpreter, valid for the borrow it is reached by.
///
/// ```
/// let interp = rustycad_tcl::Interp::new();
/// interp.eval("set x 5").unwrap();
/// assert_eq!(interp.result(), "5");
/// ```
pub struct InterpHandle {
    raw: NonNull<sys::Tcl_Interp>,
    _not_send: PhantomData<*mut ()>,
}

impl InterpHandle {
    /// Wraps a pointer Tcl passed to a callback.
    ///
    /// # Safety
    /// `raw` must be a live interpreter that outlives the returned value.
    pub(crate) unsafe fn from_raw(raw: NonNull<sys::Tcl_Interp>) -> Self {
        Self {
            raw,
            _not_send: PhantomData,
        }
    }

    /// The raw pointer for calls into C; `handle.as_ptr()`.
    pub fn as_ptr(&self) -> *mut sys::Tcl_Interp {
        self.raw.as_ptr()
    }

    /// `Tcl_Eval`: evaluates `script`; `interp.eval("expr 1+2")`.
    pub fn eval(&self, script: &str) -> Result<Completion, TclError> {
        let script = to_c_string(script)?;
        // SAFETY: live interpreter (type invariant) and a live C string.
        let code = unsafe { sys::Tcl_Eval(self.as_ptr(), script.as_ptr()) };
        Ok(Completion::from_code(code))
    }

    /// `Tcl_RecordAndEval`: records `script` in the history, then evaluates it.
    pub fn record_and_eval(&self, script: &str, flags: c_int) -> Result<Completion, TclError> {
        let script = to_c_string(script)?;
        // SAFETY: live interpreter and C string; Tcl copies the script.
        let code = unsafe { sys::Tcl_RecordAndEval(self.as_ptr(), script.as_ptr(), flags) };
        Ok(Completion::from_code(code))
    }

    /// `Tcl_EvalFile`: sources the file `path`.
    pub fn eval_file(&self, path: &str) -> Result<Completion, TclError> {
        let path = to_c_string(path)?;
        // SAFETY: live interpreter and C string.
        let code = unsafe { sys::Tcl_EvalFile(self.as_ptr(), path.as_ptr()) };
        Ok(Completion::from_code(code))
    }

    /// `Tcl_Init`: runs `init.tcl` (auto-loading, `unknown`, the library path).
    pub fn init(&self) -> Completion {
        // SAFETY: live interpreter.
        Completion::from_code(unsafe { sys::Tcl_Init(self.as_ptr()) })
    }

    /// `Tcl_GetStringResult`, copied.
    pub fn result(&self) -> String {
        // SAFETY: live interpreter; the result pointer is valid until the next call,
        // and `from_c_pointer` copies it at once.
        unsafe { from_c_pointer(sys::Tcl_GetStringResult(self.as_ptr())) }.unwrap_or_default()
    }

    /// `Tcl_ResetResult`.
    pub fn reset_result(&self) {
        // SAFETY: live interpreter.
        unsafe { sys::Tcl_ResetResult(self.as_ptr()) };
    }

    /// `Tcl_SetResult(..., TCL_VOLATILE)`: Tcl copies `text` at once.
    pub fn set_result(&self, text: &str) -> Result<(), TclError> {
        let text = to_c_string(text)?;
        // SAFETY: live interpreter; TCL_VOLATILE makes Tcl copy the string before return,
        // and Tcl does not write through the pointer despite its `char *` type.
        unsafe { sys::Tcl_SetResult(self.as_ptr(), text.as_ptr().cast_mut(), sys::TCL_VOLATILE) };
        Ok(())
    }

    /// `Tcl_AppendResult(interp, text, NULL)`.
    pub fn append_result(&self, text: &str) -> Result<(), TclError> {
        let text = to_c_string(text)?;
        let end: *const std::ffi::c_char = std::ptr::null();
        // SAFETY: live interpreter; the variadic list is C strings closed by NULL.
        unsafe { sys::Tcl_AppendResult(self.as_ptr(), text.as_ptr(), end) };
        Ok(())
    }

    /// `Tcl_AppendElement`: appends `element` as a properly quoted list element.
    pub fn append_element(&self, element: &str) -> Result<(), TclError> {
        let element = to_c_string(element)?;
        // SAFETY: live interpreter and C string.
        unsafe { sys::Tcl_AppendElement(self.as_ptr(), element.as_ptr()) };
        Ok(())
    }

    /// `Tcl_AddErrorInfo`: appends `message` to `errorInfo`.
    pub fn add_error_info(&self, message: &str) -> Result<(), TclError> {
        let message = to_c_string(message)?;
        // SAFETY: live interpreter and C string.
        unsafe { sys::Tcl_AddErrorInfo(self.as_ptr(), message.as_ptr()) };
        Ok(())
    }
}

/// An interpreter created by this program; dropping it calls `Tcl_DeleteInterp`.
///
/// ```
/// let interp = rustycad_tcl::Interp::new();
/// assert_eq!(interp.eval("expr {6*7}").unwrap(), rustycad_tcl::Completion::Ok);
/// assert_eq!(interp.result(), "42");
/// ```
pub struct Interp {
    handle: InterpHandle,
}

impl Interp {
    /// `Tcl_CreateInterp` (after the one-time library initialization).
    pub fn new() -> Self {
        initialize_library();
        // SAFETY: the library is initialized; Tcl_CreateInterp never returns NULL (it
        // panics the process on allocation failure).
        let raw = unsafe { sys::Tcl_CreateInterp() };
        let raw = NonNull::new(raw).expect("Tcl_CreateInterp returns a non-null interpreter");
        // SAFETY: freshly created, deleted only by Drop.
        let handle = unsafe { InterpHandle::from_raw(raw) };
        Self { handle }
    }
}

impl Default for Interp {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for Interp {
    type Target = InterpHandle;

    fn deref(&self) -> &InterpHandle {
        &self.handle
    }
}

impl Drop for Interp {
    fn drop(&mut self) {
        // SAFETY: this value owns the interpreter; Tcl defers the deletion while an
        // evaluation is still running on it.
        unsafe { sys::Tcl_DeleteInterp(self.handle.as_ptr()) };
    }
}

/// `Tcl_CommandComplete`: true when `script` has no open braces, quotes or brackets.
///
/// ```
/// assert!(rustycad_tcl::command_complete("puts {a}"));
/// assert!(!rustycad_tcl::command_complete("puts {a"));
/// ```
pub fn command_complete(script: &str) -> bool {
    let Ok(script) = to_c_string(script) else {
        // A NUL cannot reach C; the C caller would see the text up to the NUL.
        let prefix = script.split('\0').next().unwrap_or_default();
        return command_complete(prefix);
    };
    // SAFETY: a live C string.
    unsafe { sys::Tcl_CommandComplete(script.as_ptr()) != 0 }
}
