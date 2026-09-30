//! The output seam of a units context: text the native package writes to
//! `std::cout`, and the payloads it sends to the default `Message` messenger.
use std::io::Write;

/// Receives the package's console text and messages; inject a recording fake in tests.
pub trait UnitsDiagnostics: Send + Sync {
    /// Bytes the native code streams to `std::cout`, newlines included.
    fn print(&self, text: &[u8]);
    /// A `Message::SendWarning` payload (no trailing newline).
    fn warning(&self, text: &[u8]);
    /// A `Message::SendInfo` payload (no trailing newline).
    fn info(&self, text: &[u8]);
}

/// Writes console text to stdout and each message as one stdout line. The native
/// default printer also colours the line; that terminal decoration is not reproduced.
pub struct ConsoleDiagnostics;

impl UnitsDiagnostics for ConsoleDiagnostics {
    fn print(&self, text: &[u8]) {
        // A closed stdout must not abort a conversion, as a failing std::cout does not.
        let _ = std::io::stdout().write_all(text);
    }
    fn warning(&self, text: &[u8]) {
        self.print(&[text, b"\n"].concat());
    }
    fn info(&self, text: &[u8]) {
        self.print(&[text, b"\n"].concat());
    }
}

/// An `io::Write` forwarding every write to the console channel.
pub struct ConsoleSink<'a> {
    diagnostics: &'a dyn UnitsDiagnostics,
}

impl<'a> ConsoleSink<'a> {
    pub(crate) fn new(diagnostics: &'a dyn UnitsDiagnostics) -> Self {
        Self { diagnostics }
    }
}

impl Write for ConsoleSink<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.diagnostics.print(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    struct RecordingDiagnostics {
        printed: Mutex<Vec<u8>>,
    }
    impl UnitsDiagnostics for RecordingDiagnostics {
        fn print(&self, text: &[u8]) {
            self.printed.lock().unwrap().extend_from_slice(text);
        }
        fn warning(&self, text: &[u8]) {
            self.print(text);
        }
        fn info(&self, text: &[u8]) {
            self.print(text);
        }
    }

    #[test]
    fn console_sink_forwards_every_write() {
        let recorder = RecordingDiagnostics::default();
        let mut sink = ConsoleSink::new(&recorder);
        sink.write_all(b"abc").unwrap();
        sink.flush().unwrap();
        assert_eq!(*recorder.printed.lock().unwrap(), b"abc");
    }
}
