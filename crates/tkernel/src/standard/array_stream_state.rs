// The cursor rules translate Standard_ArrayStreamBuffer.cxx at OCCT revision
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Mutable storage shared by concrete stream-buffer adapters and hook overrides.
use std::io;

/// Seek origins exposed to OCCT-compatible overrides, including its no-op invalid direction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArrayStreamSeekDirection {
    /// Offset starts at the beginning of the buffer.
    Begin,
    /// Offset starts at the current cursor.
    Current,
    /// Native offset is subtracted from the end position.
    End,
    /// Represents an invalid native seek direction, which leaves the cursor unchanged.
    Other,
}

/// The one cursor and borrowed byte range used by every hook invocation.
pub struct ArrayStreamBufferState<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> ArrayStreamBufferState<'a> {
    /// Starts a stream at the beginning of borrowed bytes; `ArrayStreamBufferState::new(b"ABC")`.
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    /// Replaces storage and resets the shared cursor; `state.init(b"replacement")`.
    pub fn init(&mut self, bytes: &'a [u8]) {
        self.bytes = bytes;
        self.position = 0;
    }

    /// Returns the current valid cursor offset; `state.position() == 0`.
    pub const fn position(&self) -> usize {
        self.position
    }

    /// Returns the immutable bytes currently backing the stream; `state.bytes() == b"ABC"`.
    pub const fn bytes(&self) -> &'a [u8] {
        self.bytes
    }

    /// Sets a valid cursor offset without changing storage; `state.set_position(2)?`.
    pub fn set_position(&mut self, position: usize) -> io::Result<()> {
        if position > self.bytes.len() {
            return Err(invalid_position(position, self.bytes.len()));
        }
        self.position = position;
        Ok(())
    }

    /// Returns the number of bytes remaining at the cursor; `state.available() == 3`.
    pub const fn available(&self) -> usize {
        self.bytes.len() - self.position
    }

    /// Returns native `int_type` or EOF without advancing; `state.underflow() == 65`.
    pub fn underflow(&mut self) -> i32 {
        self.bytes
            .get(self.position)
            .map_or(-1, |byte| i32::from(*byte))
    }

    /// Returns native `int_type` and advances once; `state.uflow() == 65`.
    pub fn uflow(&mut self) -> i32 {
        let byte = self.underflow();
        if byte == -1 {
            return -1;
        }
        self.position += 1;
        byte
    }

    /// Applies native integer putback comparison; `state.pbackfail(65) == 65`.
    pub fn pbackfail(&mut self, requested: i32) -> i32 {
        let Some(previous) = self.position.checked_sub(1) else {
            return -1;
        };
        let byte = self.bytes[previous];
        let promoted = i32::from(byte as std::ffi::c_char);
        if requested != -1 && requested != promoted {
            return -1;
        }
        self.position = previous;
        i32::from(byte)
    }

    /// Seeks with OCCT's offset convention; `state.seekoff(2, ArrayStreamSeekDirection::Begin)?`.
    pub fn seekoff(&mut self, offset: i64, direction: ArrayStreamSeekDirection) -> io::Result<u64> {
        let length = self.bytes.len() as i128;
        let current = self.position as i128;
        let offset = i128::from(offset);
        let target = seek_target(current, length, offset, direction)?;
        self.position = target as usize;
        Ok(self.position as u64)
    }

    /// Seeks to an absolute native position; `state.seekpos(2)?`.
    pub fn seekpos(&mut self, position: u64) -> io::Result<u64> {
        let offset = i64::try_from(position).map_err(|_| invalid_seek_position(position))?;
        self.seekoff(offset, ArrayStreamSeekDirection::Begin)
    }

    /// Copies and consumes up to the destination length; `state.xsgetn(&mut output)`.
    pub fn xsgetn(&mut self, destination: &mut [u8]) -> usize {
        let count = destination.len().min(self.available());
        let end = self.position + count;
        destination[..count].copy_from_slice(&self.bytes[self.position..end]);
        self.position = end;
        count
    }

    /// Returns unread bytes for Rust's borrowed `BufRead` adapter; `state.remaining() == b"BC"`.
    pub fn remaining(&self) -> &'a [u8] {
        &self.bytes[self.position..]
    }
}

// Native out-of-range pointer arithmetic is undefined, so only valid targets reach this state.
fn seek_target(
    current: i128,
    length: i128,
    offset: i128,
    direction: ArrayStreamSeekDirection,
) -> io::Result<i128> {
    match direction {
        ArrayStreamSeekDirection::Begin => clamp_high(offset, length),
        ArrayStreamSeekDirection::Current => clamp_high(current + offset, length),
        ArrayStreamSeekDirection::End => clamp_low(length - offset, length),
        ArrayStreamSeekDirection::Other => Ok(current),
    }
}

fn clamp_high(target: i128, length: i128) -> io::Result<i128> {
    if target < 0 {
        return Err(invalid_signed_position(target, length));
    }
    Ok(target.min(length))
}

fn clamp_low(target: i128, length: i128) -> io::Result<i128> {
    if target > length {
        return Err(invalid_signed_position(target, length));
    }
    Ok(target.max(0))
}

fn invalid_position(position: usize, length: usize) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("position {position}: expected 0..={length}"),
    )
}

fn invalid_seek_position(position: u64) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("position {position}: expected a signed native position"),
    )
}

fn invalid_signed_position(position: i128, length: i128) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("position {position}: expected 0..={length}"),
    )
}
