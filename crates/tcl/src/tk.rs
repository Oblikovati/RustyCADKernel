//! The Tk calls DRAW's X11 window layer makes: initialization, the main window, the
//! application name, uids, the main loop.
//!
//! A [`TkWindow`] borrows the interpreter it belongs to: Tk destroys the main window
//! with its interpreter, and the borrow keeps the interpreter alive.
use crate::text::{from_c_pointer, to_c_string};
use crate::{Completion, InterpHandle, StaticPackage, TclError, sys};
use std::ffi::{CStr, c_int};
use std::marker::PhantomData;
use std::ptr::NonNull;

/// Proof that `Tk_Init` succeeded on an interpreter. The Tk library reaches Tcl through
/// the stubs table that `Tk_Init` fills; before that, calls such as `Tk_GetUid`
/// dereference a null table and crash, so they are methods of this token.
pub struct TkApp<'interp> {
    interp: &'interp InterpHandle,
}

/// `Tk_Init`: loads Tk into `interp`, opening the display (`$DISPLAY` on X11). On
/// failure the error is the interpreter result (Tk's reason).
/// On macOS, call on the main application thread: `tk_init(&interp)?`.
pub fn tk_init(interp: &InterpHandle) -> Result<TkApp<'_>, String> {
    #[cfg(target_os = "macos")]
    // CI job 114323792463: Aqua raises a foreign exception from Rust test workers.
    // SAFETY: pthread_main_np has no arguments or initialization prerequisites.
    if unsafe { pthread_main_np() } == 0 {
        return Err(
            "Tk_Init received a worker thread; expected the macOS main application thread".into(),
        );
    }
    // SAFETY: live interpreter, and Aqua is restricted to the main application thread.
    match Completion::from_code(unsafe { sys::Tk_Init(interp.as_ptr()) }) {
        Completion::Ok => Ok(TkApp { interp }),
        _ => Err(interp.result()),
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn pthread_main_np() -> c_int;
}

/// The Tk package for [`InterpHandle::static_package`], as DRAW registers it.
pub struct TkPackage;

impl StaticPackage for TkPackage {
    fn init(interp: &InterpHandle) -> Completion {
        match tk_init(interp) {
            Ok(_) => Completion::Ok,
            Err(_) => Completion::Error,
        }
    }
}

impl<'interp> TkApp<'interp> {
    /// `Tk_MainWindow`: the application's main window.
    pub fn main_window(&self) -> Option<TkWindow<'interp>> {
        TkWindow::main(self.interp)
    }

    /// `Tk_GetUid`: the unique, never freed copy of `text` (equal texts give one pointer).
    pub fn get_uid(&self, text: &str) -> Result<&'static CStr, TclError> {
        let text = to_c_string(text)?;
        // SAFETY: Tk is initialized (token); Tk keeps every uid for the process life.
        Ok(unsafe { CStr::from_ptr(sys::Tk_GetUid(text.as_ptr())) })
    }

    /// `Tk_GetNumMainWindows`: how many Tk applications are open in this thread.
    pub fn main_window_count(&self) -> c_int {
        // SAFETY: Tk is initialized (token).
        unsafe { sys::Tk_GetNumMainWindows() }
    }

    /// `Tk_MainLoop`: processes events until every main window is closed.
    pub fn main_loop(&self) {
        // SAFETY: Tk is initialized (token); runs this thread's event loop.
        unsafe { sys::Tk_MainLoop() };
    }
}

/// A Tk window of the interpreter it borrows.
pub struct TkWindow<'interp> {
    raw: NonNull<sys::Tk_Window_>,
    _interp: PhantomData<&'interp InterpHandle>,
}

impl<'interp> TkWindow<'interp> {
    /// `Tk_MainWindow`: the main window, or `None` when Tk is not loaded into `interp`
    /// (Tk checks its stubs table first, so this is safe before `Tk_Init`).
    pub fn main(interp: &'interp InterpHandle) -> Option<Self> {
        // SAFETY: live interpreter; returns NULL (and sets a result) without Tk.
        let raw = NonNull::new(unsafe { sys::Tk_MainWindow(interp.as_ptr()) })?;
        Some(Self {
            raw,
            _interp: PhantomData,
        })
    }

    /// `Tk_SetAppName`: requests `name` and returns the name actually given.
    pub fn set_app_name(&self, name: &str) -> Result<String, TclError> {
        let name = to_c_string(name)?;
        // SAFETY: live window; the returned name is copied at once.
        let given = unsafe { sys::Tk_SetAppName(self.raw.as_ptr(), name.as_ptr()) };
        // SAFETY: Tk returns a live C string.
        Ok(unsafe { from_c_pointer(given) }.unwrap_or_default())
    }

    /// `Tk_Name`: the window's name uid.
    pub fn name(&self) -> String {
        // SAFETY: live window; uids are never freed.
        unsafe { from_c_pointer(sys::Tk_Name(self.raw.as_ptr())) }.unwrap_or_default()
    }

    /// `Tk_Name(window) = uid`: renames the window record, as DRAW does after
    /// `Tk_SetAppName` (the uid must come from [`TkApp::get_uid`], which never frees it).
    pub fn set_name_uid(&self, uid: &'static CStr) {
        let head = self.raw.as_ptr().cast::<sys::Tk_FakeWinHead>();
        // SAFETY: live window; `nameUid` is a public field of Tk_FakeWin (offset checked
        // against tk.h) and the uid outlives the window.
        unsafe { (*head).nameUid = uid.as_ptr() };
    }

    /// `Tk_GeometryRequest`: asks the geometry manager for a size.
    pub fn geometry_request(&self, width: c_int, height: c_int) {
        // SAFETY: live window.
        unsafe { sys::Tk_GeometryRequest(self.raw.as_ptr(), width, height) };
    }

    /// `Tk_Display`: the X display the window lives on, for the native window layer.
    pub fn display(&self) -> *mut sys::Display {
        // SAFETY: live window.
        unsafe { sys::Tk_Display(self.raw.as_ptr()) }
    }
}
