//! The console DRAW prints to: `std::cout`/`std::cerr` in the C++ code.
//!
//! The port writes through the [`DrawConsole`] interface so that a session can be driven
//! and observed in tests ([`RecordingConsole`]); [`ProcessConsole`] is the real standard
//! output and error used by the `drawexe` binary. Messages sent through
//! `Message::send_*` still go to the default messenger's printers, as in C++.
use std::cell::RefCell;
use std::io::Write;

/// Text output of a DRAW session.
pub trait DrawConsole {
    /// Writes to the standard output stream (`std::cout << text`).
    fn write_out(&self, text: &str);
    /// Writes to the standard error stream (`std::cerr << text`).
    fn write_err(&self, text: &str);
    /// Flushes both streams (`flush_standard_streams`).
    fn flush(&self);
}

/// The process's standard output and error.
///
/// ```
/// use rustycad_tkdraw::draw::{DrawConsole, ProcessConsole};
/// ProcessConsole.flush();
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct ProcessConsole;

impl DrawConsole for ProcessConsole {
    fn write_out(&self, text: &str) {
        // `std::ostream` reports write failures through state bits DRAW never reads.
        let _ = std::io::stdout().write_all(text.as_bytes());
    }

    fn write_err(&self, text: &str) {
        let _ = std::io::stderr().write_all(text.as_bytes());
    }

    fn flush(&self) {
        let _ = std::io::stderr().flush();
        let _ = std::io::stdout().flush();
    }
}

/// A console that records what a session prints (the test double of [`ProcessConsole`]).
///
/// ```
/// use rustycad_tkdraw::draw::{DrawConsole, RecordingConsole};
/// let console = RecordingConsole::default();
/// console.write_out("a");
/// console.write_err("b");
/// assert_eq!(console.output(), "a");
/// assert_eq!(console.errors(), "b");
/// ```
#[derive(Debug, Default)]
pub struct RecordingConsole {
    output: RefCell<String>,
    errors: RefCell<String>,
}

impl RecordingConsole {
    /// Everything written to the output stream so far.
    pub fn output(&self) -> String {
        self.output.borrow().clone()
    }

    /// Everything written to the error stream so far.
    pub fn errors(&self) -> String {
        self.errors.borrow().clone()
    }

    /// Returns and clears the recorded output.
    pub fn take_output(&self) -> String {
        std::mem::take(&mut *self.output.borrow_mut())
    }
}

impl DrawConsole for RecordingConsole {
    fn write_out(&self, text: &str) {
        self.output.borrow_mut().push_str(text);
    }

    fn write_err(&self, text: &str) {
        self.errors.borrow_mut().push_str(text);
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_console_takes_output() {
        let console = RecordingConsole::default();
        console.write_out("x");
        assert_eq!(console.take_output(), "x");
        assert_eq!(console.output(), "");
    }
}
