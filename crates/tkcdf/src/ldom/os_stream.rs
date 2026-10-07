// Rust port of ApplicationFramework/TKCDF/LDOM/LDOM_OSStream.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LDOM_SBuffer` and `LDOM_OSStream`: an output stream into a list of chunks
//! (ocaf-design.md section 9). Every write copies with C `strncpy`, so the bytes after a
//! NUL of one write are zeros, and `str` reads each chunk up to its first NUL.
use rustycad_tkernel::standard::{Failure, exceptions};
use std::io;

/// `LDOM_OSStream::BOMType`: the byte order mark the reader found.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LdomBomType {
    /// `BOM_UNDEFINED`.
    #[default]
    BomUndefined,
    /// `BOM_UTF8`.
    BomUtf8,
    /// `BOM_UTF16BE`.
    BomUtf16be,
    /// `BOM_UTF16LE`.
    BomUtf16le,
    /// `BOM_UTF32BE`.
    BomUtf32be,
    /// `BOM_UTF32LE`.
    BomUtf32le,
    /// `BOM_UTF7`.
    BomUtf7,
    /// `BOM_UTF1`.
    BomUtf1,
    /// `BOM_UTFEBCDIC`.
    BomUtfebcdic,
    /// `BOM_SCSU`.
    BomScsu,
    /// `BOM_BOCU1`.
    BomBocu1,
    /// `BOM_GB18030`.
    BomGb18030,
}

/// `LDOM_SBuffer`; `LdomSBuffer::new(16)`.
///
/// ```
/// use rustycad_tkcdf::ldom::LdomSBuffer;
/// let mut buffer = LdomSBuffer::new(4);
/// buffer.xsputn(b"abcdef").unwrap();
/// assert_eq!(buffer.str(), b"abcdef");
/// assert_eq!(buffer.length(), 6);
/// ```
#[derive(Clone, Debug)]
pub struct LdomSBuffer {
    /// `myMaxBuf`: the default chunk size.
    max_buf: i32,
    /// `myLength`: the bytes written since the last clear.
    length: i32,
    /// `myFirstString` and its successors: the written bytes of each chunk (`len`).
    chunks: Vec<Vec<u8>>,
}

impl LdomSBuffer {
    /// `LDOM_SBuffer(theMaxBuf)`: an empty buffer with one chunk.
    pub fn new(max_buf: i32) -> Self {
        Self {
            max_buf,
            length: 0,
            chunks: vec![Vec::new()],
        }
    }
    /// `str`: the bytes written, each chunk read up to its first NUL (C `strncpy`) and
    /// padded with zeros to `length` bytes.
    pub fn str(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.length as usize);
        for chunk in &self.chunks {
            result.extend_from_slice(&strncpy(chunk, chunk.len()));
        }
        result.resize(self.length as usize, 0);
        result
    }
    /// `Length`.
    pub fn length(&self) -> i32 {
        self.length
    }
    /// `Clear`: no bytes, one new chunk.
    pub fn clear(&mut self) {
        *self = Self::new(self.max_buf);
    }
    /// `overflow(c)`: writes one byte; returns `c`.
    pub fn overflow(&mut self, byte: i32) -> i32 {
        let _ = self.xsputn(&[byte as u8]);
        byte
    }
    /// `underflow`: the buffer cannot be read; `EOF` (-1).
    pub fn underflow(&mut self) -> i32 {
        -1
    }
    /// `xsputn(s, n)`: appends the bytes to the current chunk, starting a new chunk when
    /// `myMaxBuf - 1` bytes are used; `Standard_ProgramError` beyond 2 GB.
    pub fn xsputn(&mut self, bytes: &[u8]) -> Result<i64, Failure> {
        let count = i32::try_from(bytes.len())
            .ok()
            .filter(|count| *count < i32::MAX);
        let Some(count) = count else {
            return Err(exceptions::program_error(&format!(
                "LDOM_SBuffer cannot work with strings greater than 2 Gb (got {} bytes)",
                bytes.len()
            )));
        };
        let current = self.chunks.last().map_or(0, Vec::len) as i32;
        let free = self.max_buf - current - 1;
        if free >= count {
            self.append(bytes);
        } else if free <= 0 {
            self.chunks.push(Vec::new());
            self.append(bytes);
        } else {
            self.append(&bytes[..free as usize]);
            self.chunks.push(Vec::new());
            self.append(&bytes[free as usize..]);
        }
        self.length += count;
        Ok(i64::from(count))
    }
    /// Appends `strncpy` of the bytes to the current chunk.
    fn append(&mut self, bytes: &[u8]) {
        if let Some(chunk) = self.chunks.last_mut() {
            chunk.extend_from_slice(&strncpy(bytes, bytes.len()));
        }
    }
}

/// C `strncpy` of `count` bytes: the bytes up to the first NUL, then zeros.
fn strncpy(bytes: &[u8], count: usize) -> Vec<u8> {
    let mut copy: Vec<u8> = bytes
        .iter()
        .take(count)
        .take_while(|byte| **byte != 0)
        .copied()
        .collect();
    copy.resize(count, 0);
    copy
}

/// `LDOM_OSStream`: an `std::ostream` over an [`LdomSBuffer`]; `LdomOSStream::new(16)`.
///
/// ```
/// use rustycad_tkcdf::ldom::LdomOSStream;
/// use std::io::Write;
/// let mut stream = LdomOSStream::new(16);
/// write!(stream, "{}", 42).unwrap();
/// assert_eq!(stream.str(), b"42");
/// stream.clear();
/// assert_eq!(stream.length(), 0);
/// ```
#[derive(Clone, Debug)]
pub struct LdomOSStream {
    /// `myBuffer`.
    buffer: LdomSBuffer,
}

impl LdomOSStream {
    /// `LDOM_OSStream(theMaxBuf)`.
    pub fn new(max_buf: i32) -> Self {
        Self {
            buffer: LdomSBuffer::new(max_buf),
        }
    }
    /// `str`.
    pub fn str(&self) -> Vec<u8> {
        self.buffer.str()
    }
    /// `Length`.
    pub fn length(&self) -> i32 {
        self.buffer.length()
    }
    /// `Clear`.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
    /// `rdbuf`: the buffer.
    pub fn rdbuf(&mut self) -> &mut LdomSBuffer {
        &mut self.buffer
    }
}

impl io::Write for LdomOSStream {
    /// The `std::ostream` writes go to `xsputn`.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.buffer
            .xsputn(bytes)
            .map_err(|failure| io::Error::other(failure.to_string()))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
