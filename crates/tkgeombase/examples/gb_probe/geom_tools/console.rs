//! The probe's `std::cout` redirection (gt_captured): every GeomTools console write goes
//! to one buffer that a case clears before and takes after the captured calls.
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::geom_tools::{self, ConsoleOutput, UndefinedTypeHandlerBase};
use std::sync::{Arc, Mutex};

static CONSOLE: Mutex<Vec<u8>> = Mutex::new(Vec::new());

/// The console the probe handlers and sets write to.
pub struct ProbeConsole;
impl ConsoleOutput for ProbeConsole {
    fn write(&self, bytes: &[u8]) {
        CONSOLE.lock().unwrap().extend_from_slice(bytes);
    }
}

pub fn probe_console() -> Arc<dyn ConsoleOutput> {
    Arc::new(ProbeConsole)
}

/// Installs the native default handler, writing to the probe console.
pub fn install_default_handler() {
    let handler = Handle::new(UndefinedTypeHandlerBase::with_console(probe_console()));
    geom_tools::set_undefined_type_handler(&handler.into_cast());
}

/// Runs `body` and returns what it wrote to the console.
pub fn captured(body: impl FnOnce()) -> Vec<u8> {
    let saved = std::mem::take(&mut *CONSOLE.lock().unwrap());
    body();
    std::mem::replace(&mut *CONSOLE.lock().unwrap(), saved)
}

/// `GB_TRY` inside a capture: the exception type goes to the redirected `std::cout`.
pub fn attempt_captured(body: impl FnOnce() -> Result<(), Failure>) {
    if let Err(error) = body() {
        ProbeConsole.write(format!("{} ", error.exception_type()).as_bytes());
    }
}
