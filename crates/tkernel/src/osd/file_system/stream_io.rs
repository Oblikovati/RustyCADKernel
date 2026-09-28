// Copyright (c) 2021 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! `std::istream` / `std::ostream` over a shared `std::streambuf`: the state bits and
//! the unformatted operations (`read`, `seekg`, `tellg`, `write`, `flush`, `seekp`,
//! `tellp`) with the libstdc++ sentry rules.
use super::{IosOpenMode, SeekDir, SharedStreamBuf, lock_shared};
use std::{
    any::Any,
    sync::{Arc, Mutex},
};

/// `std::ios_base::iostate`: the eof, fail and bad bits; `StreamState::GOOD`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StreamState(u8);
impl StreamState {
    /// `goodbit`.
    pub const GOOD: Self = Self(0);
    /// `badbit`.
    pub const BAD: Self = Self(1);
    /// `eofbit`.
    pub const EOF: Self = Self(2);
    /// `failbit`.
    pub const FAIL: Self = Self(4);
    /// Returns the libstdc++ bits (bad 1, eof 2, fail 4); `state.bits()`.
    pub const fn bits(self) -> u8 {
        self.0
    }
    const fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    const fn has(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

/// The state and buffer of one stream; `StreamIo::new(buffer)`.
pub struct StreamIo {
    buffer: SharedStreamBuf,
    state: StreamState,
    gcount: usize,
}
impl StreamIo {
    /// A good stream over `buffer`; `StreamIo::new(share_stream_buf(FileBuffer::new()))`.
    pub fn new(buffer: SharedStreamBuf) -> Self {
        Self {
            buffer,
            state: StreamState::GOOD,
            gcount: 0,
        }
    }
    /// `setstate(failbit)`, used by the file streams' `open` and `close`.
    pub(crate) fn set_fail(&mut self) {
        self.setstate(StreamState::FAIL);
    }
    fn setstate(&mut self, bits: StreamState) {
        self.state = self.state.with(bits);
    }
    /// The istream sentry (noskipws): a stream that is not good gets failbit and the
    /// operation is skipped.
    fn istream_sentry(&mut self) -> bool {
        if self.state == StreamState::GOOD {
            return true;
        }
        self.setstate(StreamState::FAIL);
        false
    }
    /// The ostream sentry: only a good stream proceeds; a bad one also gets failbit.
    fn ostream_sentry(&mut self) -> bool {
        if self.state == StreamState::GOOD {
            return true;
        }
        if self.state.has(StreamState::BAD) {
            self.setstate(StreamState::FAIL);
        }
        false
    }
    fn failed(&self) -> bool {
        self.state.has(StreamState::FAIL.with(StreamState::BAD))
    }
    fn seek(&mut self, offset: i64, direction: SeekDir, which: IosOpenMode) {
        if self.failed() {
            return;
        }
        if lock_shared(&self.buffer).pubseekoff(offset, direction, which) == -1 {
            self.setstate(StreamState::FAIL);
        }
    }
    fn tell(&self, which: IosOpenMode) -> i64 {
        if self.failed() {
            return -1;
        }
        lock_shared(&self.buffer).pubseekoff(0, SeekDir::Cur, which)
    }
}

/// The `std::basic_ios` members shared by input and output streams; implement the two
/// accessors, `stream.good()`.
pub trait StreamAccess: Send + Any {
    /// Borrows the stream state; `stream.stream_io()`.
    fn stream_io(&self) -> &StreamIo;
    /// Borrows the stream state mutably; `stream.stream_io_mut()`.
    fn stream_io_mut(&mut self) -> &mut StreamIo;
    /// `rdstate`; `stream.rdstate() == StreamState::GOOD`.
    fn rdstate(&self) -> StreamState {
        self.stream_io().state
    }
    /// `good`; `stream.good()`.
    fn good(&self) -> bool {
        self.rdstate() == StreamState::GOOD
    }
    /// `eof`; `stream.eof()`.
    fn eof(&self) -> bool {
        self.rdstate().has(StreamState::EOF)
    }
    /// `fail`: failbit or badbit; `stream.fail()`.
    fn fail(&self) -> bool {
        self.rdstate().has(StreamState::FAIL.with(StreamState::BAD))
    }
    /// `bad`; `stream.bad()`.
    fn bad(&self) -> bool {
        self.rdstate().has(StreamState::BAD)
    }
    /// `clear()`: resets every state bit; `stream.clear()`.
    fn clear(&mut self) {
        self.stream_io_mut().state = StreamState::GOOD;
    }
    /// `rdbuf`: the shared buffer; `stream.rdbuf()`.
    fn rdbuf(&self) -> SharedStreamBuf {
        Arc::clone(&self.stream_io().buffer)
    }
}

/// `std::istream`; `stream.read(&mut bytes)`.
pub trait IStream: StreamAccess {
    /// `read`: fills `out`; a short read sets eof and fail. Returns `gcount()`;
    /// a stream that is not good reads nothing and sets fail, `stream.read(&mut bytes)`.
    fn read(&mut self, out: &mut [u8]) -> usize {
        let io: &mut StreamIo = self.stream_io_mut();
        io.gcount = 0;
        if !io.istream_sentry() {
            return 0;
        }
        io.gcount = lock_shared(&io.buffer).sgetn(out);
        if io.gcount < out.len() {
            io.setstate(StreamState::EOF.with(StreamState::FAIL));
        }
        io.gcount
    }
    /// `gcount`: bytes the last `read` extracted; `stream.gcount()`.
    fn gcount(&self) -> usize {
        self.stream_io().gcount
    }
    /// `seekg(off, dir)`: clears eofbit first (N3168), then seeks unless the stream
    /// failed; a refused seek sets failbit, `stream.seekg(0, SeekDir::Beg)`.
    fn seekg(&mut self, offset: i64, direction: SeekDir) {
        let io: &mut StreamIo = self.stream_io_mut();
        io.state = StreamState(io.state.0 & !StreamState::EOF.0);
        if io.istream_sentry() {
            io.seek(offset, direction, IosOpenMode::IN);
        }
    }
    /// `tellg`: -1, and failbit, on a stream that is not good; `stream.tellg()`.
    fn tellg(&mut self) -> i64 {
        let io: &mut StreamIo = self.stream_io_mut();
        if !io.istream_sentry() {
            return -1;
        }
        io.tell(IosOpenMode::IN)
    }
}

/// `std::ostream`; `stream.write(b"bytes")`.
pub trait OStream: StreamAccess {
    /// `write`: a short write sets badbit; a stream that is not good writes nothing (a
    /// bad one also gets failbit, the ostream sentry rule), `stream.write(b"bytes")`.
    fn write(&mut self, bytes: &[u8]) {
        let io: &mut StreamIo = self.stream_io_mut();
        if !io.ostream_sentry() {
            return;
        }
        if lock_shared(&io.buffer).sputn(bytes) != bytes.len() {
            io.setstate(StreamState::BAD);
        }
    }
    /// `flush`: a failing `pubsync` sets badbit; `stream.flush()`.
    fn flush(&mut self) {
        let io: &mut StreamIo = self.stream_io_mut();
        if io.ostream_sentry() && lock_shared(&io.buffer).pubsync() == -1 {
            io.setstate(StreamState::BAD);
        }
    }
    /// `seekp(off, dir)`: libstdc++ seeks whenever the stream has not failed, even
    /// when the sentry refused an eof stream; `stream.seekp(0, SeekDir::End)`.
    fn seekp(&mut self, offset: i64, direction: SeekDir) {
        let io: &mut StreamIo = self.stream_io_mut();
        io.ostream_sentry();
        io.seek(offset, direction, IosOpenMode::OUT);
    }
    /// `tellp`: -1 on a failed stream; `stream.tellp()`.
    fn tellp(&mut self) -> i64 {
        let io: &mut StreamIo = self.stream_io_mut();
        io.ostream_sentry();
        io.tell(IosOpenMode::OUT)
    }
}

/// `std::shared_ptr<std::istream>`.
pub type SharedIStream = Arc<Mutex<dyn IStream>>;
/// `std::shared_ptr<std::ostream>`.
pub type SharedOStream = Arc<Mutex<dyn OStream>>;

/// Tells whether two shared values are the same object (C++ pointer equality);
/// `same_object(&first, &second)`.
pub(crate) fn same_object<T: ?Sized>(first: &Arc<T>, second: &Arc<T>) -> bool {
    std::ptr::addr_eq(Arc::as_ptr(first), Arc::as_ptr(second))
}
