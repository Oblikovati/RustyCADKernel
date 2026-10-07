//! Project-owned thin binding to the Tcl/Tk 8.6 C API, for the DRAW test harness port.
//!
//! - [`sys`]: the raw, hand-written declarations of exactly the functions OCCT
//!   `src/Draw` calls (no bindgen), with tables the header test checks against the
//!   installed `tcl.h`/`tk.h`.
//! - The rest is a safe wrapper: [`Interp`]/[`InterpHandle`] (evaluation, results,
//!   variables, Rust commands and variable traces), [`DString`], [`Channel`], event
//!   handlers, process calls and the Tk calls of DRAW's window layer.
//!
//! Every `unsafe` block names the contract it relies on. Tcl objects belong to the
//! thread that created them, so no wrapper type is `Send` or `Sync`.
//!
//! ```
//! use rustycad_tcl::{Completion, Interp};
//! let interp = Interp::new();
//! assert_eq!(interp.eval("set greeting hello").unwrap(), Completion::Ok);
//! assert_eq!(interp.get_var("greeting", 0).unwrap().as_deref(), Some("hello"));
//! ```
pub mod sys;

mod channel;
mod command;
mod dstring;
mod error;
mod event;
mod interp;
mod process;
mod text;
mod tk;
mod trace;
mod variable;

pub use channel::{Channel, NewChannel, StdChannel};
pub use command::CommandProc;
pub use dstring::{DString, external_to_utf, utf_to_unichars};
pub use error::{Completion, TclError};
#[cfg(unix)]
pub use event::FileHandler;
pub use event::{ChannelHandler, EventBody, do_one_event};
pub use interp::{Interp, InterpHandle, command_complete, initialize_library};
pub use process::{ExitBody, StaticPackage, create_exit_handler, exit, panic_abort, put_env};
pub use tk::{TkApp, TkPackage, TkWindow, tk_init};
pub use trace::{TraceEvent, TraceTag, VarTraceProc};
