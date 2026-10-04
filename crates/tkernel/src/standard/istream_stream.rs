//! Sticky stream-state adapter for `std::istream`-style block reads.
use std::io::{Error, ErrorKind, Read};

/// Tracks the C++ stream flags around a Rust byte source; `IstreamStream::new(source)`.
pub struct IstreamStream<R> {
    source: R,
    eof: bool,
    fail: bool,
    bad: bool,
}

impl<R: Read> IstreamStream<R> {
    /// Wraps a source with sticky EOF/failure state; `IstreamStream::new(Cursor::new(bytes))`.
    pub fn new(source: R) -> Self {
        Self {
            source,
            eof: false,
            fail: false,
            bad: false,
        }
    }

    /// Clears stream flags while retaining source position; `stream.clear()`.
    pub fn clear(&mut self) {
        self.eof = false;
        self.fail = false;
        self.bad = false;
    }

    /// Sets initial native-style status bits; `stream.set_status(false, true, false)`.
    pub fn set_status(&mut self, eof: bool, fail: bool, bad: bool) {
        self.eof = eof;
        self.fail = fail;
        self.bad = bad;
    }

    /// Reports the native `eof()` state; `stream.is_eof()`.
    pub fn is_eof(&self) -> bool {
        self.eof
    }

    /// Reports the native `fail()` state; `stream.is_fail()`.
    pub fn is_fail(&self) -> bool {
        self.fail
    }

    /// Reports the native `bad()` state; `stream.is_bad()`.
    pub fn is_bad(&self) -> bool {
        self.bad
    }

    /// Returns the wrapped source; `let source = stream.into_inner()`.
    pub fn into_inner(self) -> R {
        self.source
    }

    fn record_read(&mut self, result: std::io::Result<usize>) -> std::io::Result<usize> {
        match result {
            Ok(0) => {
                self.eof = true;
                self.fail = true;
                Ok(0)
            }
            Ok(count) => Ok(count),
            Err(error) if error.kind() == ErrorKind::Interrupted => Err(error),
            Err(error) => {
                self.bad = true;
                self.fail = true;
                Err(error)
            }
        }
    }
}

impl<R: Read> Read for IstreamStream<R> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if self.bad {
            return Err(Error::other("istream badbit is set"));
        }
        if self.fail {
            return Ok(0);
        }
        let result = self.source.read(buffer);
        self.record_read(result)
    }
}

#[cfg(test)]
#[path = "istream_stream/tests.rs"]
mod tests;
