//! Unique byte allocation with weak copied observers that survive ownership transfer.
#![allow(unsafe_code)]
use std::{
    cell::UnsafeCell,
    fmt,
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock, RwLockWriteGuard},
};
static SHARED_EMPTY_BYTES: Vec<u8> = Vec::new();

pub(super) struct AsciiBufferAllocation {
    pub(super) bytes: UnsafeCell<Vec<u8>>,
    pub(super) gate: RwLock<bool>,
}
// Only the unique AsciiString owner mutates bytes, under its allocation write gate.
// Weak observers only copy under a read gate; ordinary slices borrow the unique owner.
unsafe impl Sync for AsciiBufferAllocation {}
impl AsciiBufferAllocation {
    pub(super) fn observe_prefix(&self) -> Option<std::ffi::CString> {
        let live = self
            .gate
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !*live {
            return None;
        }
        // Copied observations retain this read gate until no borrowed byte can escape.
        let bytes = unsafe { &*self.bytes.get() };
        let end = bytes
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(bytes.len());
        Some(std::ffi::CString::new(&bytes[..end]).expect("bounded allocation prefix excludes NUL"))
    }
}

#[derive(Default)]
pub(super) struct OwnedAsciiBuffer {
    pub(super) allocation: Option<Arc<AsciiBufferAllocation>>,
}
impl OwnedAsciiBuffer {
    pub(super) fn from_vector(bytes: Vec<u8>) -> Self {
        // The owner and its byte allocation travel together through native Move/Swap.
        let allocation = AsciiBufferAllocation {
            bytes: UnsafeCell::new(bytes),
            gate: RwLock::new(true),
        };
        Self {
            allocation: Some(Arc::new(allocation)),
        }
    }
    pub(super) fn reserve(&mut self, required: usize) {
        if required <= self.capacity() {
            return;
        }
        let mut bytes = Vec::with_capacity(super::storage::padded_byte_capacity(required - 1));
        bytes.extend_from_slice(self);
        // A new allocation retires the old target, including allocator address reuse.
        *self = Self::from_vector(bytes);
    }
    pub(super) fn write(&mut self) -> AsciiBufferWrite<'_> {
        let Some(allocation) = &self.allocation else {
            return AsciiBufferWrite::StaticEmpty(Vec::new());
        };
        let lease = allocation
            .gate
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        AsciiBufferWrite::Allocated {
            _lease: lease,
            bytes: &allocation.bytes,
            bounds: capture_ascii_buffer_bounds(self),
        }
    }
}
impl Drop for OwnedAsciiBuffer {
    fn drop(&mut self) {
        let Some(allocation) = &self.allocation else {
            return;
        };
        let mut live = allocation
            .gate
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Temporary observation leases cannot keep a released allocation logically alive.
        *live = false;
    }
}
impl Deref for OwnedAsciiBuffer {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        let Some(allocation) = &self.allocation else {
            return &SHARED_EMPTY_BYTES;
        };
        // Ordinary slices borrow the unique owner; concurrent observers only read.
        unsafe { &*allocation.bytes.get() }
    }
}
impl fmt::Debug for OwnedAsciiBuffer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Keep logical diagnostics independent of lock/control-block implementation details.
        fmt::Debug::fmt(&**self, formatter)
    }
}
pub(super) enum AsciiBufferWrite<'a> {
    Allocated {
        _lease: RwLockWriteGuard<'a, bool>,
        bytes: &'a UnsafeCell<Vec<u8>>,
        bounds: AsciiBufferBounds,
    },
    StaticEmpty(Vec<u8>),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AsciiBufferBounds {
    address: usize,
    capacity: usize,
}
fn capture_ascii_buffer_bounds(bytes: &Vec<u8>) -> AsciiBufferBounds {
    AsciiBufferBounds {
        address: bytes.as_ptr() as usize,
        capacity: bytes.capacity(),
    }
}
impl Drop for AsciiBufferWrite<'_> {
    fn drop(&mut self) {
        match self {
            Self::Allocated { bytes, bounds, .. } => verify_ascii_buffer_bounds(bytes, *bounds),
            Self::StaticEmpty(bytes) => debug_assert!(
                bytes.is_empty() && bytes.capacity() == 0,
                "static buffer length/capacity {}/{}; expected 0/0",
                bytes.len(),
                bytes.capacity()
            ),
        }
    }
}
fn verify_ascii_buffer_bounds(bytes: &UnsafeCell<Vec<u8>>, expected: AsciiBufferBounds) {
    // Guard still owns the write gate; a Vec realloc must instead retire the allocation owner.
    let actual = capture_ascii_buffer_bounds(unsafe { &*bytes.get() });
    debug_assert_eq!(
        actual, expected,
        "received {actual:?}; expected unchanged allocation {expected:?}"
    );
}
impl Deref for AsciiBufferWrite<'_> {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        match self {
            // Retain the write gate throughout the Vec operation.
            Self::Allocated { bytes, .. } => unsafe { &*bytes.get() },
            Self::StaticEmpty(bytes) => bytes,
        }
    }
}
impl DerefMut for AsciiBufferWrite<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            // Exclusive owner borrow and write gate serialize writes with copied observers.
            Self::Allocated { bytes, .. } => unsafe { &mut *bytes.get() },
            Self::StaticEmpty(bytes) => bytes,
        }
    }
}
