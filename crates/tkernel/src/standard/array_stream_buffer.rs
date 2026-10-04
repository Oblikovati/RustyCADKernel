// Rust translation of FoundationClasses/TKernel/Standard/Standard_ArrayStreamBuffer.cxx at
// OCCT revision 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Borrowed stream storage with overridable OCCT-style virtual hooks.
use std::io::{self, BufRead, Read, Seek, SeekFrom};

use super::array_stream_hooks::{ArrayStreamBufferHooks, NativeArrayStreamBufferHooks};
use super::array_stream_state::{ArrayStreamBufferState, ArrayStreamSeekDirection};

/// A borrowed stream buffer whose hook implementation can be specialized by a Rust type.
pub struct ArrayStreamBuffer<'a, H = NativeArrayStreamBufferHooks> {
    state: ArrayStreamBufferState<'a>,
    hooks: H,
}

impl<'a> ArrayStreamBuffer<'a, NativeArrayStreamBufferHooks> {
    /// Borrows memory without copying; `ArrayStreamBuffer::new(b"ABC")`.
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self::with_hooks(bytes, NativeArrayStreamBufferHooks)
    }

    /// Maps a null, zero-length native buffer to an empty slice; `ArrayStreamBuffer::from_optional(None)`.
    pub fn from_optional(bytes: Option<&'a [u8]>) -> Self {
        Self::new(bytes.unwrap_or_default())
    }
}

impl<'a, H: ArrayStreamBufferHooks> ArrayStreamBuffer<'a, H> {
    /// Creates a stream adapter with custom virtual callbacks; `ArrayStreamBuffer::with_hooks(b"ABC", Hooks)`.
    pub const fn with_hooks(bytes: &'a [u8], hooks: H) -> Self {
        Self {
            state: ArrayStreamBufferState::new(bytes),
            hooks,
        }
    }

    /// Exposes the hook implementation for stateful derived-style adapters; `buffer.hooks_mut()`.
    pub fn hooks_mut(&mut self) -> &mut H {
        &mut self.hooks
    }

    /// Calls virtual `Init` and resets state by default; `buffer.init(b"replacement")`.
    pub fn init(&mut self, bytes: &'a [u8]) {
        self.hooks.init(&mut self.state, bytes);
    }

    /// Calls virtual `underflow`; example: `buffer.underflow() == 65`.
    pub fn underflow(&mut self) -> i32 {
        self.hooks.underflow(&mut self.state)
    }

    /// Calls virtual `uflow` and returns its native integer result; `buffer.uflow_int() == 65`.
    pub fn uflow_int(&mut self) -> i32 {
        self.hooks.uflow(&mut self.state)
    }

    /// Returns a byte-or-EOF adapter for virtual `uflow`; use `uflow_int` for custom integer results.
    pub fn uflow(&mut self) -> Option<u8> {
        byte_result(self.uflow_int())
    }

    /// Calls virtual `pbackfail`; example: `buffer.pbackfail(-1)` requests EOF.
    pub fn pbackfail(&mut self, requested: i32) -> i32 {
        self.hooks.pbackfail(&mut self.state, requested)
    }

    /// Calls virtual `showmanyc`; `buffer.showmanyc()` returns unread byte count.
    pub fn showmanyc(&mut self) -> isize {
        self.hooks.showmanyc(&mut self.state)
    }

    /// Calls virtual `seekoff`; `buffer.seekoff(2, ArrayStreamSeekDirection::Begin)?`.
    pub fn seekoff(&mut self, offset: i64, direction: ArrayStreamSeekDirection) -> io::Result<u64> {
        self.hooks.seekoff(&mut self.state, offset, direction)
    }

    /// Calls virtual `seekpos`; `buffer.seekpos(2)?`.
    pub fn seekpos(&mut self, position: u64) -> io::Result<u64> {
        self.hooks.seekpos(&mut self.state, position)
    }

    /// Returns unread bytes as a state inspection, without invoking virtual `showmanyc`.
    pub const fn available(&self) -> usize {
        self.state.available()
    }

    /// Peeks at stored bytes without invoking virtual `underflow`.
    pub fn peek(&self) -> Option<u8> {
        self.state.bytes().get(self.state.position()).copied()
    }

    /// Calls virtual `pbackfail` using Rust's unsigned-byte convenience request.
    /// A high byte may not match on signed-`char` targets; pass its promoted value through
    /// `put_back_integer` or call `pbackfail` for the native integer hook.
    pub fn put_back(&mut self, byte: Option<u8>) -> Option<u8> {
        byte_result(self.pbackfail(byte.map_or(-1, i32::from)))
    }

    /// Calls virtual `pbackfail` with the promoted native character request.
    /// `None` represents EOF/failure or a non-byte custom result; `pbackfail` retains exact `int_type`.
    ///
    /// ```
    /// use rustycad_tkernel::standard::ArrayStreamBuffer;
    /// let mut buffer = ArrayStreamBuffer::new(b"A");
    /// assert_eq!(buffer.uflow(), Some(b'A'));
    /// assert_eq!(buffer.put_back_integer(65), Some(b'A'));
    /// ```
    pub fn put_back_integer(&mut self, requested: i32) -> Option<u8> {
        byte_result(self.pbackfail(requested))
    }

    /// Calls virtual `seekoff` using OCCT's end-offset convention.
    /// `SeekFrom::End(distance)` means `end - distance`; standard Rust `Seek` keeps its
    /// conventional end-relative meaning.
    /// Native out-of-range pointer arithmetic is undefined; this method rejects those targets.
    pub fn seek_occt(&mut self, offset: SeekFrom) -> io::Result<u64> {
        seek_occt(self, offset)
    }

    /// Calls virtual `xsgetn`; `buffer.xsgetn(&mut destination)`.
    pub fn xsgetn(&mut self, destination: &mut [u8]) -> usize {
        self.hooks.xsgetn(&mut self.state, destination)
    }
}

fn byte_result(result: i32) -> Option<u8> {
    u8::try_from(result).ok()
}

impl<H: ArrayStreamBufferHooks> Read for ArrayStreamBuffer<'_, H> {
    fn read(&mut self, destination: &mut [u8]) -> io::Result<usize> {
        Ok(self.xsgetn(destination).min(destination.len()))
    }
}

impl<H: ArrayStreamBufferHooks> BufRead for ArrayStreamBuffer<'_, H> {
    /// Exposes stored unread bytes; this Rust adapter does not emulate virtual `underflow`.
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(self.state.remaining())
    }

    fn consume(&mut self, amount: usize) {
        let consumed = amount.min(self.state.available());
        let next_position = self.state.position() + consumed;
        self.state
            .set_position(next_position)
            .expect("clamped to buffer length");
    }
}

impl<H: ArrayStreamBufferHooks> Seek for ArrayStreamBuffer<'_, H> {
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        seek_standard(self, offset)
    }
}

fn seek_standard<H: ArrayStreamBufferHooks>(
    buffer: &mut ArrayStreamBuffer<'_, H>,
    offset: SeekFrom,
) -> io::Result<u64> {
    match offset {
        SeekFrom::Start(position) => seek_start_clamped(buffer, position),
        SeekFrom::Current(delta) => buffer.seekoff(delta, ArrayStreamSeekDirection::Current),
        SeekFrom::End(delta) if delta >= 0 => {
            seek_start_clamped(buffer, buffer.state.bytes().len() as u64)
        }
        SeekFrom::End(delta) => seek_end_relative(buffer, delta),
    }
}

fn seek_start_clamped<H: ArrayStreamBufferHooks>(
    buffer: &mut ArrayStreamBuffer<'_, H>,
    position: u64,
) -> io::Result<u64> {
    let end_position = buffer.state.bytes().len() as u64;
    buffer.seekpos(position.min(end_position))
}

fn seek_end_relative<H: ArrayStreamBufferHooks>(
    buffer: &mut ArrayStreamBuffer<'_, H>,
    delta: i64,
) -> io::Result<u64> {
    let native_offset = delta
        .checked_neg()
        .ok_or_else(|| invalid_seek_delta(delta))?;
    buffer.seekoff(native_offset, ArrayStreamSeekDirection::End)
}

fn seek_occt<H: ArrayStreamBufferHooks>(
    buffer: &mut ArrayStreamBuffer<'_, H>,
    offset: SeekFrom,
) -> io::Result<u64> {
    match offset {
        SeekFrom::Start(position) => seek_start_clamped(buffer, position),
        SeekFrom::Current(delta) => buffer.seekoff(delta, ArrayStreamSeekDirection::Current),
        SeekFrom::End(distance) => buffer.seekoff(distance, ArrayStreamSeekDirection::End),
    }
}

fn invalid_seek_delta(delta: i64) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("end offset {delta}: expected a negatable native offset"),
    )
}
