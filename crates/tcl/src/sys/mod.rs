//! Raw declarations of the Tcl/Tk 8.6 C API, hand-written (no bindgen) for exactly the
//! functions OCCT `src/Draw` uses. Everything here is `unsafe` to call; the safe wrapper
//! in the rest of this crate is the only intended user. The tables
//! [`CALLBACK_SIGNATURES`], [`TCL_SIGNATURES`], [`TCL_VARIADIC_SIGNATURES`],
//! [`TK_SIGNATURES`] and [`CONSTANT_VALUES`] describe the declarations for the header
//! test, which compiles them against the installed `tcl.h`/`tk.h`.
#![allow(non_upper_case_globals)]

mod constants;
mod functions;
mod signature;
mod types;

pub use constants::*;
pub use functions::*;
pub use signature::Signature;
pub use types::*;
