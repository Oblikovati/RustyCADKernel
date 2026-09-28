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

//! The C++ iostream vocabulary the OSD file systems speak: `std::ios_base::openmode`,
//! `std::ios_base::seekdir` and the `std::streambuf` public interface, as project-owned
//! Rust types so that file systems stay independent of any stream library.
use std::{
    any::Any,
    ops::{BitAnd, BitOr},
    sync::{Arc, Mutex},
};

/// `std::ios_base::openmode`, with the libstdc++ bit values (app 1, ate 2, binary 4,
/// in 8, out 16, trunc 32) so a raw native mode converts unchanged;
/// `IosOpenMode::IN | IosOpenMode::BINARY`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct IosOpenMode(u32);
impl IosOpenMode {
    /// No flag.
    pub const NONE: Self = Self(0);
    /// `std::ios_base::app`: every write appends.
    pub const APP: Self = Self(1);
    /// `std::ios_base::ate`: seek to the end once opened.
    pub const ATE: Self = Self(2);
    /// `std::ios_base::binary`: no newline translation (meaningful on Windows only).
    pub const BINARY: Self = Self(4);
    /// `std::ios_base::in`: open for reading.
    pub const IN: Self = Self(8);
    /// `std::ios_base::out`: open for writing.
    pub const OUT: Self = Self(16);
    /// `std::ios_base::trunc`: truncate an existing file.
    pub const TRUNC: Self = Self(32);
    /// Wraps raw libstdc++ bits, dropping unknown ones; `IosOpenMode::from_bits(24)`.
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits & 63)
    }
    /// Returns the raw libstdc++ bits; `IosOpenMode::IN.bits() == 8`.
    pub const fn bits(self) -> u32 {
        self.0
    }
    /// Tells whether every flag of `other` is set; `mode.contains(IosOpenMode::IN)`.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
    /// Tells whether any flag of `other` is set; `mode.intersects(IosOpenMode::OUT)`.
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}
impl BitOr for IosOpenMode {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
impl BitAnd for IosOpenMode {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

/// `std::ios_base::seekdir`; `SeekDir::Beg`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeekDir {
    /// `std::ios_base::beg`.
    Beg,
    /// `std::ios_base::cur`.
    Cur,
    /// `std::ios_base::end`.
    End,
}

/// The public interface of `std::streambuf`: the byte source/sink a stream wraps;
/// implement it for a custom protocol, `buffer.sgetn(&mut bytes)`.
pub trait StreamBuf: Send + Any {
    /// `sgetn`: reads up to `out.len()` bytes and returns how many were read (0 at the
    /// end or when the buffer cannot read); `buffer.sgetn(&mut bytes)`.
    fn sgetn(&mut self, out: &mut [u8]) -> usize;
    /// `sputn`: writes the bytes and returns how many were accepted; `buffer.sputn(b"x")`.
    fn sputn(&mut self, bytes: &[u8]) -> usize;
    /// `pubseekoff`: moves the position selected by `which` and returns it, or -1 on
    /// failure; `buffer.pubseekoff(0, SeekDir::End, IosOpenMode::IN)`.
    fn pubseekoff(&mut self, offset: i64, direction: SeekDir, which: IosOpenMode) -> i64;
    /// `pubsync`: flushes pending output, 0 on success and -1 on failure; `buffer.pubsync()`.
    fn pubsync(&mut self) -> i32;
}

/// `std::shared_ptr<std::streambuf>`: a stream buffer shared by streams and caches.
pub type SharedStreamBuf = Arc<Mutex<dyn StreamBuf>>;

/// Wraps a concrete buffer as a shared one; `share_stream_buf(FileBuffer::new())`.
pub fn share_stream_buf(buffer: impl StreamBuf) -> SharedStreamBuf {
    Arc::new(Mutex::new(buffer))
}

/// Locks a shared value, recovering it when a panicking holder poisoned the lock (the
/// guarded state is plain data that stays consistent); `lock_shared(&buffer)`.
pub(crate) fn lock_shared<T: ?Sized>(shared: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
