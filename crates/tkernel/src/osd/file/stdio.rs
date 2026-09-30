//! The `FILE*` the Unix OSD_File keeps beside its descriptor (`fdopen`), for `ReadLine`
//! (`fgets`) and `Rewind` (`rewind`), as the glibc 2.28+ stdio implements them.
//!
//! The descriptor and the stream share one kernel offset but the stream buffers ahead, so
//! mixing `Read`/`Seek` with `ReadLine` observes the buffer: this model keeps the glibc
//! buffer size (`st_blksize`, at most `BUFSIZ`), its cached offset (unknown after
//! `fdopen` and after an end-of-file or error read), sticky end-of-file and the
//! in-buffer `rewind` shortcut.
use super::calls::{CallResult, Errno, PosixFileCalls, SeekOrigin};

/// glibc `BUFSIZ`: the stream buffer size when `st_blksize` is 0 or larger.
const BUFSIZ: usize = 8192;

/// The state of one glibc `FILE` over a descriptor.
#[derive(Clone, Debug)]
pub(super) struct StdioStream {
    fd: i32,
    reads_allowed: bool,
    buffer: Option<Vec<u8>>,
    read_ptr: usize,
    read_end: usize,
    /// `_offset`: the descriptor offset the stream believes; `None` is `_IO_pos_BAD`.
    offset: Option<i64>,
    eof_seen: bool,
    error_seen: bool,
    last_errno: i32,
}

/// The outcome of `fgets`: the line, or `None` for a NULL return.
pub(super) type LineRead = Option<Vec<u8>>;

impl StdioStream {
    /// `fdopen(fd, mode)`: `reads_allowed` is false for "w" and "a". glibc does not ask
    /// the descriptor for its offset (the cache starts unknown), so the first `rewind`
    /// always seeks.
    pub(super) const fn attach(fd: i32, reads_allowed: bool) -> Self {
        Self {
            fd,
            reads_allowed,
            buffer: None,
            read_ptr: 0,
            read_end: 0,
            offset: None,
            eof_seen: false,
            error_seen: false,
            last_errno: 0,
        }
    }
    /// `feof`.
    pub(super) const fn eof(&self) -> bool {
        self.eof_seen
    }
    /// The errno the last failing stream call left.
    pub(super) const fn last_errno(&self) -> i32 {
        self.last_errno
    }
    /// `fgets(buffer, size, stream)`: at most `size - 1` bytes, through the first newline.
    pub(super) fn fgets(&mut self, calls: &dyn PosixFileCalls, size: i32) -> LineRead {
        if size <= 0 {
            return None;
        }
        if size == 1 {
            return Some(Vec::new());
        }
        let old_error: bool = std::mem::replace(&mut self.error_seen, false);
        let line: Vec<u8> = self.get_line(calls, size as usize - 1);
        let failed: bool = self.error_seen && self.last_errno != eagain();
        self.error_seen |= old_error;
        (!line.is_empty() && !failed).then_some(line)
    }
    /// `_IO_getline(stream, buffer, n, '\n', extract)`.
    fn get_line(&mut self, calls: &dyn PosixFileCalls, limit: usize) -> Vec<u8> {
        let mut line: Vec<u8> = Vec::new();
        while line.len() < limit {
            if self.read_ptr == self.read_end && !self.underflow(calls) {
                break;
            }
            let byte: u8 = self.buffer.as_ref().expect("filled buffer")[self.read_ptr];
            self.read_ptr += 1;
            line.push(byte);
            if byte == b'\n' {
                break;
            }
        }
        line
    }
    /// `_IO_new_file_underflow`: refills the buffer with one `read`; false at EOF/error.
    fn underflow(&mut self, calls: &dyn PosixFileCalls) -> bool {
        if self.eof_seen {
            return false;
        }
        if !self.reads_allowed {
            self.error_seen = true;
            self.last_errno = ebadf();
            return false;
        }
        let capacity: usize = self.capacity(calls);
        let buffer: &mut Vec<u8> = self.buffer.get_or_insert_with(|| vec![0; capacity]);
        let count: usize = match calls.read(self.fd, buffer) {
            Ok(count) => count,
            Err(Errno(code)) => {
                self.error_seen = true;
                self.last_errno = code;
                0
            }
        };
        (self.read_ptr, self.read_end) = (0, count);
        if count == 0 {
            self.eof_seen |= !self.error_seen;
            // "Our offset cache would no longer be valid, so unset it."
            self.offset = None;
            return false;
        }
        self.offset = self.offset.map(|offset| offset + count as i64);
        true
    }
    /// `_IO_file_doallocate`: `st_blksize` when it is below `BUFSIZ`, else `BUFSIZ`.
    fn capacity(&self, calls: &dyn PosixFileCalls) -> usize {
        if let Some(buffer) = &self.buffer {
            return buffer.len();
        }
        match calls.fstat(self.fd) {
            Ok(status) if status.block_size > 0 && (status.block_size as usize) < BUFSIZ => {
                status.block_size as usize
            }
            _ => BUFSIZ,
        }
    }
    /// `rewind`: `_IO_new_file_seekoff(0, SEEK_SET)` then `clearerr`.
    pub(super) fn rewind(&mut self, calls: &dyn PosixFileCalls) {
        let _ = self.seek_start(calls);
        self.eof_seen = false;
        self.error_seen = false;
    }
    fn seek_start(&mut self, calls: &dyn PosixFileCalls) -> CallResult<()> {
        if self.buffer.is_none() {
            let capacity: usize = self.capacity(calls);
            self.buffer = Some(vec![0; capacity]);
            (self.read_ptr, self.read_end) = (0, 0);
        }
        if let Some(offset) = self.offset {
            // The destination 0 lies in the buffer: move the read pointer, then resync the
            // descriptor to the stream's cached offset.
            let start: i64 = offset - self.read_end as i64;
            if 0 >= start && 0 < offset {
                self.read_ptr = (-start) as usize;
                self.eof_seen = false;
                calls.lseek(self.fd, offset, SeekOrigin::Start)?;
                return Ok(());
            }
        }
        let position: i64 = calls.lseek(self.fd, 0, SeekOrigin::Start)?;
        (self.read_ptr, self.read_end) = (0, 0);
        self.offset = Some(position);
        self.eof_seen = false;
        Ok(())
    }
}

#[cfg(unix)]
const fn ebadf() -> i32 {
    libc::EBADF
}
#[cfg(unix)]
const fn eagain() -> i32 {
    libc::EAGAIN
}
#[cfg(not(unix))]
const fn ebadf() -> i32 {
    9
}
#[cfg(not(unix))]
const fn eagain() -> i32 {
    11
}
