//! C types of `tcl.h` and `tk.h` that the declarations name. Opaque C structs are
//! zero-sized `#[repr(C)]` types that only appear behind pointers.
#![allow(non_camel_case_types, non_snake_case)]

use std::ffi::{c_char, c_int, c_ulong, c_ushort, c_void};
use std::marker::{PhantomData, PhantomPinned};

/// Declares opaque C structs that Rust only handles by pointer.
macro_rules! opaque_types {
    ($( $(#[$attr:meta])* $name:ident ; )*) => {
        $( $(#[$attr])*
        #[repr(C)]
        pub struct $name {
            _private: [u8; 0],
            // Not Send/Sync/Unpin: the object belongs to the C library and its thread.
            _marker: PhantomData<(*mut u8, PhantomPinned)>,
        } )*
    };
}

opaque_types! {
    /// `Tcl_Interp`: an interpreter (its public fields are deprecated in 8.6).
    Tcl_Interp;
    /// `struct Tcl_Channel_`: the target of the `Tcl_Channel` handle.
    Tcl_Channel_;
    /// `struct Tcl_Command_`: the target of the `Tcl_Command` token.
    Tcl_Command_;
    /// `struct Tcl_Encoding_`: the target of the `Tcl_Encoding` handle.
    Tcl_Encoding_;
    /// `struct Tk_Window_`: the target of the `Tk_Window` handle.
    Tk_Window_;
    /// Xlib `Display`.
    Display;
    /// Xlib `Visual`.
    Visual;
    /// Xlib `XEvent` (a union in C; only passed by pointer here).
    XEvent;
    /// The function type `Tcl_FreeProc`; `TCL_VOLATILE` and friends are integer values
    /// cast to `Tcl_FreeProc *`, which a Rust function pointer cannot hold, so the
    /// parameter is a data pointer to this marker (same size and passing convention).
    Tcl_FreeProcTarget;
}

/// `ClientData`: `void *`.
pub type ClientData = *mut c_void;
/// `Tcl_Channel`: `struct Tcl_Channel_ *`.
pub type Tcl_Channel = *mut Tcl_Channel_;
/// `Tcl_Command`: `struct Tcl_Command_ *`.
pub type Tcl_Command = *mut Tcl_Command_;
/// `Tcl_Encoding`: `struct Tcl_Encoding_ *`.
pub type Tcl_Encoding = *mut Tcl_Encoding_;
/// `Tk_Window`: `struct Tk_Window_ *`.
pub type Tk_Window = *mut Tk_Window_;
/// `Tk_Uid`: `const char *` with identity semantics.
pub type Tk_Uid = *const c_char;
/// `Tcl_UniChar`: UTF-16 code unit in a default 8.6 build.
pub type Tcl_UniChar = c_ushort;
/// Xlib `Window`: an XID.
pub type Window = c_ulong;
/// `Tcl_FreeProc *` as passed to `Tcl_SetResult`.
pub type Tcl_FreeProcPointer = *const Tcl_FreeProcTarget;

/// `TCL_DSTRING_STATIC_SIZE`: the inline buffer of a `Tcl_DString`.
pub const DSTRING_STATIC_SIZE: usize = 200;

/// `Tcl_DString`: a dynamic string. `string` may point into `staticSpace`, so a value
/// must not move between `Tcl_DStringInit` and `Tcl_DStringFree`.
#[repr(C)]
pub struct Tcl_DString {
    /// Current text, NUL-terminated (`Tcl_DStringValue`).
    pub string: *mut c_char,
    /// Length in bytes without the terminating NUL.
    pub length: c_int,
    /// Bytes available in `string`.
    pub spaceAvl: c_int,
    /// Inline storage used for short strings.
    pub staticSpace: [c_char; DSTRING_STATIC_SIZE],
}

/// The leading fields of `Tk_FakeWin`, the public mirror of a Tk window record; the
/// macros `Tk_Display` and `Tk_Name` read `display` and `nameUid` through it. Only the
/// prefix is declared: the record is always owned by Tk and read through a pointer.
#[repr(C)]
pub struct Tk_FakeWinHead {
    /// `Display *display`.
    pub display: *mut Display,
    /// `char *dummy1` (dispPtr).
    pub dummy1: *mut c_char,
    /// `int screenNum`.
    pub screenNum: c_int,
    /// `Visual *visual`.
    pub visual: *mut Visual,
    /// `int depth`.
    pub depth: c_int,
    /// `Window window`.
    pub window: Window,
    /// `char *dummy2` (childList).
    pub dummy2: *mut c_char,
    /// `char *dummy3` (lastChildPtr).
    pub dummy3: *mut c_char,
    /// `Tk_Window parentPtr`.
    pub parentPtr: Tk_Window,
    /// `char *dummy4` (nextPtr).
    pub dummy4: *mut c_char,
    /// `char *dummy5` (mainPtr).
    pub dummy5: *mut c_char,
    /// `char *pathName`.
    pub pathName: *mut c_char,
    /// `Tk_Uid nameUid`.
    pub nameUid: Tk_Uid,
}
