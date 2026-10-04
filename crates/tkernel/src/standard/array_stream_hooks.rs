//! Rust override points corresponding to OCCT's virtual stream-buffer members.
use std::io;

use super::array_stream_state::{ArrayStreamBufferState, ArrayStreamSeekDirection};

/// Customizes stream-buffer behavior while sharing one cursor and one byte range.
///
/// Implement only the callbacks whose behavior differs; each default matches the
/// pinned OCCT implementation on valid buffer positions. Use it with
/// `ArrayStreamBuffer::with_hooks(b"ABC", CustomHooks)`.
pub trait ArrayStreamBufferHooks {
    /// Reinitializes the shared buffer; example: `state.init(bytes)` resets the cursor.
    fn init<'a>(&mut self, state: &mut ArrayStreamBufferState<'a>, bytes: &'a [u8]) {
        state.init(bytes);
    }

    /// Implements native `underflow`; example: `state.underflow()` reads the current int result.
    fn underflow(&mut self, state: &mut ArrayStreamBufferState<'_>) -> i32 {
        state.underflow()
    }

    /// Implements native `uflow`; example: `state.uflow()` returns and consumes one int result.
    fn uflow(&mut self, state: &mut ArrayStreamBufferState<'_>) -> i32 {
        state.uflow()
    }

    /// Implements native `pbackfail`; example: `state.pbackfail(-1)` requests EOF putback.
    fn pbackfail(&mut self, state: &mut ArrayStreamBufferState<'_>, requested: i32) -> i32 {
        state.pbackfail(requested)
    }

    /// Implements native `showmanyc`; example: `state.available() as isize` returns unread bytes.
    fn showmanyc(&mut self, state: &mut ArrayStreamBufferState<'_>) -> isize {
        state.available() as isize
    }

    /// Implements native `seekoff`; example: `state.seekoff(1, ArrayStreamSeekDirection::Begin)`.
    fn seekoff(
        &mut self,
        state: &mut ArrayStreamBufferState<'_>,
        offset: i64,
        direction: ArrayStreamSeekDirection,
    ) -> io::Result<u64> {
        state.seekoff(offset, direction)
    }

    /// Implements native `seekpos`; example: `state.seekpos(1)` selects the absolute position.
    fn seekpos(
        &mut self,
        state: &mut ArrayStreamBufferState<'_>,
        position: u64,
    ) -> io::Result<u64> {
        let offset = i64::try_from(position).map_err(|_| invalid_seekpos(position))?;
        self.seekoff(state, offset, ArrayStreamSeekDirection::Begin)
    }

    /// Implements native `xsgetn`; example: `state.xsgetn(output)` consumes a bounded slice.
    fn xsgetn(&mut self, state: &mut ArrayStreamBufferState<'_>, output: &mut [u8]) -> usize {
        state.xsgetn(output)
    }
}

/// Default callbacks reproducing the concrete OCCT stream-buffer behavior.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeArrayStreamBufferHooks;

impl ArrayStreamBufferHooks for NativeArrayStreamBufferHooks {}

fn invalid_seekpos(position: u64) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("position {position}: expected a signed native position"),
    )
}
