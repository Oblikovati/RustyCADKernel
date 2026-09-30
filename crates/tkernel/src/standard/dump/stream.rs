// Rust translation of FoundationClasses/TKernel/Standard/Standard_Dump.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use std::io::{self, Cursor, Write};
/// Output seam supporting readable and opaque streams; `add_values_separator(&mut output)`.
pub trait DumpOutput {
    /// Returns the write position, or a nonpositive failure value; `output.position()`.
    fn position(&self) -> i64;
    /// Returns the entire buffer when readable; `output.content()`.
    fn content(&self) -> Option<&[u8]>;
    /// Writes at the current position; `output.write_fragment(b", ")?`.
    fn write_fragment(&mut self, bytes: &[u8]) -> io::Result<()>;
}
/// In-memory stream retaining write position separately from content length; `DumpStream::default()`.
#[derive(Default)]
pub struct DumpStream {
    cursor: Cursor<Vec<u8>>,
}
impl DumpStream {
    /// Writes UTF-8 text with native overwrite semantics; `stream.append("{")`.
    pub fn append(&mut self, text: &str) {
        self.cursor.write_all(text.as_bytes()).unwrap();
    }
    /// Borrows UTF-8 content; use bytes for non-UTF-8 fragments; `stream.text()`.
    pub fn text(&self) -> &str {
        std::str::from_utf8(self.cursor.get_ref())
            .expect("DumpStream text: expected UTF-8; use bytes for binary content")
    }
    /// Borrows all bytes including embedded NULs; `stream.bytes()`.
    pub fn bytes(&self) -> &[u8] {
        self.cursor.get_ref()
    }
    /// Reports the write cursor; `stream.position()`.
    pub fn position(&self) -> i64 {
        self.cursor.position() as i64
    }
    /// Sets the byte write position; `stream.seek(0)`.
    pub fn seek(&mut self, position: u64) {
        self.cursor.set_position(position);
    }
}
impl DumpOutput for DumpStream {
    fn position(&self) -> i64 {
        self.position()
    }
    fn content(&self) -> Option<&[u8]> {
        Some(self.bytes())
    }
    fn write_fragment(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.cursor.write_all(bytes)
    }
}
