//! The `std::istream` the LDOM reader reads (ocaf-design.md section 9): `read` and
//! `getline` over a Rust byte source with the C++ eof and fail flags and `gcount`.
use rustycad_tkernel::osd::{IStream, SharedIStream};
use std::io::{ErrorKind, Read};

/// An `std::istream` over a byte source; `LdomIStream::new(&mut bytes.as_slice())`.
///
/// ```
/// use rustycad_tkcdf::ldom::LdomIStream;
/// let mut source: &[u8] = b"<a><b";
/// let mut stream = LdomIStream::new(&mut source);
/// let mut line = [0u8; 8];
/// stream.getline(&mut line, b'>');
/// assert_eq!((stream.gcount(), &line[..3]), (3, &b"<a\0"[..]));
/// ```
pub struct LdomIStream<'a> {
    source: &'a mut dyn Read,
    /// A byte `getline` looked at but did not extract.
    pending: Option<u8>,
    eof: bool,
    fail: bool,
    gcount: usize,
}

impl<'a> LdomIStream<'a> {
    /// A good stream over `source`.
    pub fn new(source: &'a mut dyn Read) -> Self {
        Self {
            source,
            pending: None,
            eof: false,
            fail: false,
            gcount: 0,
        }
    }
    /// `good`: neither eof nor fail.
    pub fn good(&self) -> bool {
        !self.eof && !self.fail
    }
    /// `gcount`: the bytes the last `read` or `getline` extracted.
    pub fn gcount(&self) -> usize {
        self.gcount
    }
    /// `read(s, n)`: fills `buffer`; a short read sets eof and fail. A stream that is not
    /// good reads nothing and sets fail.
    pub fn read(&mut self, buffer: &mut [u8]) {
        self.gcount = 0;
        if !self.sentry() {
            return;
        }
        if let Some(byte) = self.pending.take().filter(|_| !buffer.is_empty()) {
            buffer[0] = byte;
            self.gcount = 1;
        }
        while self.gcount < buffer.len() {
            match self.source.read(&mut buffer[self.gcount..]) {
                Ok(count) if count > 0 => self.gcount += count,
                Err(error) if error.kind() == ErrorKind::Interrupted => {}
                _ => {
                    self.eof = true;
                    self.fail = true;
                    return;
                }
            }
        }
    }
    /// `getline(s, n, delim)`: stores up to `n - 1` bytes before `delim` and a NUL; the
    /// delimiter is extracted (and counted) but not stored. End of input sets eof; a full
    /// buffer without delimiter, or nothing extracted, sets fail.
    pub fn getline(&mut self, buffer: &mut [u8], delimiter: u8) {
        self.gcount = 0;
        if !self.sentry() || buffer.is_empty() {
            return;
        }
        let mut next = self.next_byte();
        while let Some(byte) =
            next.filter(|byte| *byte != delimiter && self.gcount + 1 < buffer.len())
        {
            buffer[self.gcount] = byte;
            self.gcount += 1;
            next = self.next_byte();
        }
        buffer[self.gcount] = 0;
        match next {
            None => self.eof = true,
            Some(byte) if byte == delimiter => self.gcount += 1,
            Some(byte) => {
                self.pending = Some(byte);
                self.fail = true;
            }
        }
        if self.gcount == 0 {
            self.fail = true;
        }
    }
    /// The istream sentry: a stream that is not good gets fail.
    fn sentry(&mut self) -> bool {
        if !self.good() {
            self.fail = true;
        }
        self.good()
    }
    /// The next byte of the source (none at its end or on a read error).
    fn next_byte(&mut self) -> Option<u8> {
        if let Some(byte) = self.pending.take() {
            return Some(byte);
        }
        let mut byte = [0u8];
        loop {
            match self.source.read(&mut byte) {
                Ok(1) => return Some(byte[0]),
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                _ => return None,
            }
        }
    }
}

/// A [`Read`] over a shared `std::istream` of the OSD file system.
pub(crate) struct SharedIStreamReader(pub(crate) SharedIStream);

impl Read for SharedIStreamReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let mut stream = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(IStream::read(&mut *stream, buffer))
    }
}
