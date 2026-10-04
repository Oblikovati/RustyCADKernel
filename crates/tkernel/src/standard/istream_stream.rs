//! Sticky stream-state adapter for `std::istream`-style block reads.
use super::{IstreamException, IstreamExceptionMask, IstreamReadResult};
use std::io::{Error, ErrorKind, Read};

/// Tracks the C++ stream flags around a Rust byte source; `IstreamStream::new(source)`.
pub struct IstreamStream<R> {
    source: R,
    eof: bool,
    fail: bool,
    bad: bool,
    exceptions: IstreamExceptionMask,
    last_gcount: usize,
}

impl<R: Read> IstreamStream<R> {
    /// Wraps a source with sticky EOF/failure state; `IstreamStream::new(Cursor::new(bytes))`.
    pub fn new(source: R) -> Self {
        Self {
            source,
            eof: false,
            fail: false,
            bad: false,
            exceptions: IstreamExceptionMask::default(),
            last_gcount: 0,
        }
    }

    /// Clears stream flags while retaining source position; `stream.clear()`.
    pub fn clear(&mut self) {
        self.eof = false;
        self.fail = false;
        self.bad = false;
    }

    /// Sets the native exception mask after applying it; a masked current state returns its
    /// failure after retaining the new mask, as `istream::exceptions(mask)` does.
    pub fn set_exceptions(
        &mut self,
        exceptions: IstreamExceptionMask,
    ) -> Result<(), IstreamException> {
        self.exceptions = exceptions;
        self.masked_exception(self.last_gcount, None)
            .map_or(Ok(()), Err)
    }

    /// Assigns stream flags before checking the exception mask; `stream.clear_state(false, true, false)?`.
    pub fn clear_state(
        &mut self,
        eof: bool,
        fail: bool,
        bad: bool,
    ) -> Result<(), IstreamException> {
        self.set_status(eof, fail, bad);
        self.masked_exception(self.last_gcount, None)
            .map_or(Ok(()), Err)
    }

    /// Reads one native `istream::read` block and reports masked state without panicking;
    /// `let result = stream.read_block(&mut bytes)?`.
    pub fn read_block(&mut self, buffer: &mut [u8]) -> Result<IstreamReadResult, IstreamException> {
        self.last_gcount = 0;
        if buffer.is_empty() && !self.has_error_state() {
            return Ok(self.result(0));
        }
        if self.has_error_state() {
            return self.read_block_from_failed_state();
        }
        self.read_block_from_source(buffer)
    }

    /// Returns the last native-protocol block count, retained across state changes; `stream.gcount()`.
    pub fn gcount(&self) -> usize {
        self.last_gcount
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
        self.fail || self.bad
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
                Err(error)
            }
        }
    }

    fn read_block_from_failed_state(&mut self) -> Result<IstreamReadResult, IstreamException> {
        if self.eof || self.bad {
            self.fail = true;
        }
        if let Some(exception) = self.masked_exception(0, None) {
            return Err(exception);
        }
        Ok(self.result(0))
    }

    fn read_block_from_source(
        &mut self,
        buffer: &mut [u8],
    ) -> Result<IstreamReadResult, IstreamException> {
        let mut count = 0;
        while count < buffer.len() {
            match self.source.read(&mut buffer[count..]) {
                Ok(0) => return self.finish_short_read(count),
                Ok(read) => count += read,
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => return self.finish_bad_read(error),
            }
        }
        self.last_gcount = count;
        Ok(self.result(count))
    }

    fn finish_short_read(&mut self, count: usize) -> Result<IstreamReadResult, IstreamException> {
        self.last_gcount = count;
        self.eof = true;
        self.fail = true;
        if let Some(exception) = self.masked_exception(count, None) {
            return Err(exception);
        }
        Ok(self.result(count))
    }

    fn finish_bad_read(
        &mut self,
        source_error: std::io::Error,
    ) -> Result<IstreamReadResult, IstreamException> {
        self.bad = true;
        if let Some(exception) = self.masked_exception(0, Some(source_error)) {
            return Err(exception);
        }
        Ok(self.result(0))
    }

    fn masked_exception(
        &self,
        gcount: usize,
        source_error: Option<std::io::Error>,
    ) -> Option<IstreamException> {
        let hits_eof = self.eof && self.exceptions.eof();
        let hits_fail = self.fail && self.exceptions.fail();
        let hits_bad = self.bad && self.exceptions.bad();
        (hits_eof || hits_fail || hits_bad).then_some(IstreamException {
            eof: self.eof,
            fail: self.fail,
            bad: self.bad,
            gcount,
            source_error,
        })
    }

    fn result(&self, gcount: usize) -> IstreamReadResult {
        IstreamReadResult {
            gcount,
            good: !self.bad,
        }
    }

    fn has_error_state(&self) -> bool {
        self.eof || self.fail || self.bad
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
        if self.eof {
            self.fail = true;
            return Ok(0);
        }
        if buffer.is_empty() {
            return Ok(0);
        }
        let result = self.source.read(buffer);
        self.record_read(result)
    }
}

#[cfg(test)]
#[path = "istream_stream/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "istream_stream/count_tests.rs"]
mod count_tests;
