// Rust translation of FoundationClasses/TKernel/Standard/Standard_ReadLineBuffer.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The scan of one read block: line ends, backslash continuations and the carry-over.
use super::{LineSpan, ReadLineBuffer};

impl ReadLineBuffer {
    /// Scans from the current position; returns the finished line, or carries the
    /// unfinished text over and returns `None`. `continued` marks a backslash that ended
    /// the previous block (it spans blocks within one call).
    pub(super) fn scan_block(&mut self, start: usize, continued: &mut bool) -> Option<LineSpan> {
        let mut start = start;
        let mut found = false;
        while self.position < self.last_read && !found {
            let position = self.position;
            let byte = self.read_buffer[position];
            if self.multiline && byte == b'\\' {
                start = self.continuation(position, start, continued);
            } else if byte == b'\n' {
                found = self.line_end(position, &mut start, continued);
            }
            self.position += 1;
        }
        if found {
            return Some(self.finish_line(start));
        }
        if start != self.position {
            self.carry(start, self.position);
        }
        None
    }

    /// A backslash in multiline mode: at the block end it marks a continuation into the
    /// next block; before `\n` or `\r\n` it joins the lines here. Returns the new start.
    fn continuation(&mut self, position: usize, start: usize, continued: &mut bool) -> usize {
        let last = self.last_read;
        if position + 1 == last || (position + 2 == last && self.read_buffer[position + 1] == b'\r')
        {
            *continued = true;
            self.gap_at_block_end(position);
            return start;
        }
        let buffer = &self.read_buffer;
        let carriage = buffer[position + 1] == b'\r';
        if !(buffer[position + 1] == b'\n' || (carriage && buffer[position + 2] == b'\n')) {
            return start;
        }
        self.position = position + if carriage { 2 } else { 1 };
        let mut end = position;
        if self.put_gap {
            self.read_buffer[end] = b' ';
            end += 1;
        }
        self.carry(start, end);
        self.position + 1
    }

    /// With the gap option, blanks the backslash (and a following `\r`) that end the block.
    fn gap_at_block_end(&mut self, position: usize) {
        if !self.put_gap {
            return;
        }
        self.read_buffer[position] = b' ';
        if position + 1 != self.last_read {
            self.read_buffer[position + 1] = b' ';
        }
    }

    /// A `\n`: ends the line unless a continuation from the previous block is pending, in
    /// which case a `\n` or `\r\n` at the block start is dropped with the continuation
    /// characters. Any other `\n` after such a continuation does not end the line (native
    /// quirk: the line then runs on to the next unescaped block boundary or the stream end).
    fn line_end(&mut self, position: usize, start: &mut usize, continued: &mut bool) -> bool {
        if !*continued {
            return true;
        }
        if position == 1 && self.read_buffer[0] == b'\r' {
            self.carried.pop();
            *start += 2;
            *continued = false;
        } else if position == 0 {
            *start += 1;
            if self.carried.last() != Some(&b'\\') {
                self.carried.pop();
            }
            self.carried.pop();
            *continued = false;
        }
        false
    }

    /// Completes the line that ends at the current position.
    fn finish_line(&mut self, start: usize) -> LineSpan {
        let end = self.position;
        if self.use_carried {
            self.carry(start, end);
            self.use_carried = false;
            return LineSpan::Carried(terminate(&mut self.carried));
        }
        self.carried.clear();
        LineSpan::Buffer(start, terminate(&mut self.read_buffer[start..end]))
    }
}

/// NUL-terminates a line in place over its `\n` (or a `\r` before it) and returns the
/// length without them; `terminate(b"ab\r\n") == 2`.
fn terminate(line: &mut [u8]) -> usize {
    let length = line.len();
    if length > 1 && line[length - 2] == b'\r' {
        line[length - 2] = 0;
        return length - 2;
    }
    line[length - 1] = 0;
    length - 1
}
