// Rust translation of FoundationClasses/TKernel/Standard/Standard_ReadLineBuffer.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Standard_ReadLineBuffer`: splits a stream into lines through a fixed-size read buffer.
use super::stream_fill::fill_from;
use std::io::Read;
mod scan;

#[cfg(test)]
#[path = "read_line_buffer/stream_hook_tests.rs"]
mod stream_hook_tests;

/// Where the line a call returns lives.
#[derive(Clone, Copy)]
enum LineSpan {
    /// Bytes `start..start + length` of the read buffer.
    Buffer(usize, usize),
    /// The first `length` bytes of the carried-over partial line.
    Carried(usize),
}

/// Line reader over any stream; `ReadLineBuffer::new(4096).read_line(&mut stream)`.
///
/// A returned line excludes its `\n` (and a `\r` before it), which are overwritten with
/// NUL in the buffer as natively. A line split across reads is assembled in a carry-over
/// buffer; the final line of a stream without a newline is returned whole. In multiline
/// mode a backslash before the newline joins lines, optionally leaving a space.
pub struct ReadLineBuffer {
    read_buffer: Vec<u8>,
    carried: Vec<u8>,
    // True once the carry-over storage was ever allocated (NCollection_LinearVector keeps
    // a null Data() until its first non-empty Resize, and Clear keeps the storage).
    carried_allocated: bool,
    use_carried: bool,
    multiline: bool,
    put_gap: bool,
    position: usize,
    last_read: usize,
}

impl ReadLineBuffer {
    /// Allocates a zeroed read buffer of `max_buffer_size` bytes; `ReadLineBuffer::new(4096)`.
    pub fn new(max_buffer_size: usize) -> Self {
        Self {
            read_buffer: vec![0; max_buffer_size],
            carried: Vec::new(),
            carried_allocated: false,
            use_carried: false,
            multiline: false,
            put_gap: true,
            position: 0,
            last_read: 0,
        }
    }

    /// Forgets buffered data and resets the modes to their defaults; `buffer.clear()`.
    /// The read buffer keeps its bytes; the next call reads a fresh block.
    pub fn clear(&mut self) {
        self.carried.clear();
        self.use_carried = false;
        self.multiline = false;
        self.put_gap = true;
        self.position = 0;
        self.last_read = 0;
    }

    /// Reads the next line, or `None` at the end of the stream or on a read error;
    /// `buffer.read_line(&mut stream)`. Empty lines are returned as empty slices.
    pub fn read_line(&mut self, stream: &mut impl Read) -> Option<&[u8]> {
        let mut read_data = 0;
        self.read_line_counted(stream, &mut read_data)
    }

    /// Reads the next line and stores in `read_data` the size of the last block read
    /// during this call (0 when the line came from the buffer); `buffer.read_line_counted(&mut s, &mut n)`.
    pub fn read_line_counted(
        &mut self,
        stream: &mut impl Read,
        read_data: &mut i64,
    ) -> Option<&[u8]> {
        *read_data = 0;
        let span = self.next_line(stream, read_data)?;
        Some(match span {
            LineSpan::Buffer(start, length) => &self.read_buffer[start..start + length],
            LineSpan::Carried(length) => &self.carried[..length],
        })
    }

    /// Reads at most `len` bytes into the internal buffer like the protected native stream hook;
    /// `buffer.read_stream(&mut stream, 4, &mut count)`. `len` must not exceed the allocated buffer.
    pub fn read_stream(
        &mut self,
        stream: &mut impl Read,
        len: usize,
        read_len: &mut usize,
    ) -> bool {
        if len > self.read_buffer.len() {
            *read_len = 0;
            return false;
        }
        let filled = fill_from(stream, &mut self.read_buffer[..len]);
        *read_len = filled.count;
        !filled.failed
    }

    /// True when backslash continuation is enabled (false by default); `buffer.is_multiline_mode()`.
    pub fn is_multiline_mode(&self) -> bool {
        self.multiline
    }

    /// True when joined lines keep a space for the continuation; `buffer.to_put_gap_in_multiline()`.
    pub fn to_put_gap_in_multiline(&self) -> bool {
        self.put_gap
    }

    /// Enables or disables multiline mode and the joining gap; `buffer.set_multiline_mode(true, true)`.
    pub fn set_multiline_mode(&mut self, multiline: bool, put_gap: bool) {
        self.multiline = multiline;
        self.put_gap = put_gap;
    }

    fn next_line(&mut self, stream: &mut impl Read, read_data: &mut i64) -> Option<LineSpan> {
        let mut continued = false;
        loop {
            if self.position == 0 || self.position >= self.last_read {
                // The native code stores gcount() before it checks bad().
                let mut block_count = 0;
                let succeeded = self.read_stream(stream, self.read_buffer.len(), &mut block_count);
                self.last_read = block_count;
                if !succeeded {
                    return None;
                }
                *read_data = self.last_read as i64;
                if self.last_read == 0 {
                    return self.take_final_line();
                }
                self.position = 0;
            }
            let start = self.position;
            if let Some(span) = self.scan_block(start, &mut continued) {
                return Some(span);
            }
        }
    }

    /// At the end of the stream the carried-over text is the last line (no terminator).
    /// Native code returns the vector's `Data()`, which is NULL while the storage was
    /// never allocated, so an empty carry-over that never held a byte ends the stream.
    fn take_final_line(&mut self) -> Option<LineSpan> {
        if !self.use_carried {
            return None;
        }
        self.use_carried = false;
        self.carried_allocated
            .then_some(LineSpan::Carried(self.carried.len()))
    }

    /// Appends `start..end` of the read buffer to the carried text, or starts it.
    fn carry(&mut self, start: usize, end: usize) {
        if !self.use_carried {
            self.carried.clear();
            self.use_carried = true;
        }
        self.carried
            .extend_from_slice(&self.read_buffer[start..end]);
        self.carried_allocated |= end > start;
    }
}
