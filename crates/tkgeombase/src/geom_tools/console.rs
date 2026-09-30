// Port-only seam for the `std::cout` writes of GeomTools_UndefinedTypeHandler.cxx,
// GeomTools_*Set.cxx (Read) and GeomTools_Debug.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The console GeomTools writes to where the native code uses `std::cout`: the
//! "UNKNOWN ... TYPE" lines of the compact print, the "Not a ... table" message of
//! the set readers and the debug dump. It is injected so tests can capture it.
use std::io::Write;
use std::sync::Arc;

/// Destination of the text native GeomTools streams to `std::cout`.
///
/// ```
/// use rustycad_tkgeombase::geom_tools::{ConsoleOutput, StdoutConsole};
/// StdoutConsole.write(b"");
/// ```
pub trait ConsoleOutput: Send + Sync {
    /// Writes the bytes exactly as the native `std::cout <<` would; `console.write(b"x\n")`.
    fn write(&self, bytes: &[u8]);
}

/// The process standard output, the native destination.
#[derive(Clone, Copy, Debug, Default)]
pub struct StdoutConsole;

impl ConsoleOutput for StdoutConsole {
    fn write(&self, bytes: &[u8]) {
        // A failing `std::cout` never raises natively, so a write error is ignored.
        let _ = std::io::stdout().write_all(bytes);
    }
}

/// The shared standard-output console the default constructors use.
pub(crate) fn stdout_console() -> Arc<dyn ConsoleOutput> {
    Arc::new(StdoutConsole)
}
