//! `fread`-style block reads over an injected Rust byte source.
use std::io::Read;

/// Injected stream with sticky EOF and error indicators like a C `FILE*`;
/// `FreadStream::new(source)`.
pub struct FreadStream<R> {
    source: R,
    eof: bool,
    error: bool,
}

impl<R: Read> FreadStream<R> {
    /// Wraps a byte source for `Standard_ReadLineBuffer::readStream(FILE*)` semantics;
    /// `FreadStream::new(std::io::Cursor::new(b"text"))`.
    pub fn new(source: R) -> Self {
        Self {
            source,
            eof: false,
            error: false,
        }
    }

    /// Reports the sticky error indicator; `stream.is_error()`.
    pub fn is_error(&self) -> bool {
        self.error
    }

    /// Reports the sticky EOF indicator; `stream.is_eof()`.
    pub fn is_eof(&self) -> bool {
        self.eof
    }

    /// Clears both indicators without changing the source; `stream.clear_status()`.
    pub fn clear_status(&mut self) {
        self.error = false;
        self.eof = false;
    }

    /// Returns the wrapped byte source; `let source = stream.into_inner()`.
    pub fn into_inner(self) -> R {
        self.source
    }

    pub(crate) fn read_block(&mut self, buffer: &mut [u8]) -> (usize, bool) {
        if self.eof {
            return (0, !self.error);
        }
        let count = self.fill_block(buffer);
        (count, !self.error)
    }

    fn fill_block(&mut self, buffer: &mut [u8]) -> usize {
        let mut count = 0;
        while count < buffer.len() && self.read_more(buffer, &mut count) {}
        count
    }

    fn read_more(&mut self, buffer: &mut [u8], count: &mut usize) -> bool {
        match self.source.read(&mut buffer[*count..]) {
            Ok(0) => self.mark_eof(),
            Ok(read) => {
                *count += read;
                true
            }
            Err(_) => self.mark_error(),
        }
    }

    fn mark_eof(&mut self) -> bool {
        self.eof = true;
        false
    }

    fn mark_error(&mut self) -> bool {
        self.error = true;
        false
    }
}
