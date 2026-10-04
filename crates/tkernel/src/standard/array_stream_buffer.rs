// Rust translation of FoundationClasses/TKernel/Standard/Standard_ArrayStreamBuffer.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Borrowed memory streams preserving OCCT's bounded seek behavior.
use std::io::{self, BufRead, Read, Seek, SeekFrom};

/// A non-owning readable buffer; `ArrayStreamBuffer::new(b"Hello")`.
pub struct ArrayStreamBuffer<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> ArrayStreamBuffer<'a> {
    /// Borrows memory without copying; `ArrayStreamBuffer::new(b"ABC")`.
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }
    /// Maps a null, zero-length C++ buffer to an empty slice; `ArrayStreamBuffer::from_optional(None)`.
    pub fn from_optional(bytes: Option<&'a [u8]>) -> Self {
        Self::new(bytes.unwrap_or_default())
    }
    /// Replaces the borrowed buffer and resets its position; `buffer.init(b"replacement")`.
    pub fn init(&mut self, bytes: &'a [u8]) {
        self.bytes = bytes;
        self.position = 0;
    }
    /// Returns unread bytes; `ArrayStreamBuffer::new(b"ABC").available() == 3`.
    pub const fn available(&self) -> usize {
        self.bytes.len() - self.position
    }
    /// Peeks without consuming; `ArrayStreamBuffer::new(b"ABC").peek() == Some(b'A')`.
    pub fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }
    /// Backs up only for EOF or the matching previous byte (`pbackfail`); `buffer.put_back(Some(b'A'))`.
    ///
    /// Native code compares the nonnegative byte request with the previous `char`
    /// promoted to `int`, so where `char` is signed a byte of 0x80 or more never matches.
    /// Use `put_back_integer` to supply the negative promoted value directly.
    pub fn put_back(&mut self, byte: Option<u8>) -> Option<u8> {
        self.put_back_integer(byte.map_or(-1, i32::from))
    }
    /// Calls native `pbackfail(int_type)`, including negative promoted character values.
    /// The EOF request `-1` backs up regardless of the previous byte.
    ///
    /// ```
    /// use rustycad_tkernel::standard::ArrayStreamBuffer;
    /// let mut buffer = ArrayStreamBuffer::new(b"A");
    /// assert_eq!(buffer.uflow(), Some(b'A'));
    /// assert_eq!(buffer.put_back_integer(65), Some(b'A'));
    /// ```
    pub fn put_back_integer(&mut self, requested: i32) -> Option<u8> {
        let previous = self.position.checked_sub(1)?;
        let actual = self.bytes[previous];
        let promoted = i32::from(actual as std::ffi::c_char);
        if requested != -1 && requested != promoted {
            return None;
        }
        self.position = previous;
        Some(actual)
    }
    /// Matches OCCT seekoff, whose end offset is subtracted; `buffer.seek_occt(SeekFrom::End(3))?`.
    /// Rust's `Seek` trait uses its conventional signed end-relative offset instead.
    ///
    /// Positions past the end clamp to the end for `Start`/`Current` and to the start for
    /// `End`, as natively. Native code moves before the start (`Start`/`Current`) or past
    /// the end (a negative `End` offset) without a check, after which reads are undefined;
    /// the port refuses those seeks with `InvalidInput` and keeps the position.
    pub fn seek_occt(&mut self, offset: SeekFrom) -> io::Result<u64> {
        match offset {
            SeekFrom::End(distance) if distance < 0 => Err(beyond_end(self.bytes.len(), distance)),
            SeekFrom::End(distance) => {
                self.set_position((self.bytes.len() as i128 - i128::from(distance)).max(0))
            }
            other => self.seek(other),
        }
    }
    /// `xsgetn`: copies up to `destination.len()` bytes and advances; `buffer.xsgetn(&mut out)`.
    pub fn xsgetn(&mut self, destination: &mut [u8]) -> usize {
        let count = destination.len().min(self.available());
        destination[..count].copy_from_slice(&self.bytes[self.position..self.position + count]);
        self.position += count;
        count
    }
    /// `uflow`: returns the current byte and advances, `None` at the end; `buffer.uflow()`.
    pub fn uflow(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.position += 1;
        Some(byte)
    }
    fn set_position(&mut self, position: i128) -> io::Result<u64> {
        if position < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("position {position}: expected 0..={}", self.bytes.len()),
            ));
        }
        self.position = position.min(self.bytes.len() as i128) as usize;
        Ok(self.position as u64)
    }
}

impl Read for ArrayStreamBuffer<'_> {
    fn read(&mut self, destination: &mut [u8]) -> io::Result<usize> {
        Ok(self.xsgetn(destination))
    }
}
fn beyond_end(length: usize, distance: i64) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!(
            "end offset {distance} would place the position past the end of a {length}-byte buffer; expected 0..={length}"
        ),
    )
}
impl BufRead for ArrayStreamBuffer<'_> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(&self.bytes[self.position..])
    }
    fn consume(&mut self, amount: usize) {
        self.position += amount.min(self.available());
    }
}
impl Seek for ArrayStreamBuffer<'_> {
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        let position = match offset {
            SeekFrom::Start(position) => i128::from(position),
            SeekFrom::Current(delta) => self.position as i128 + i128::from(delta),
            SeekFrom::End(delta) => self.bytes.len() as i128 + i128::from(delta),
        };
        self.set_position(position)
    }
}
