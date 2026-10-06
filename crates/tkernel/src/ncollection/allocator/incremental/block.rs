//! Stable native headers precede their payload; short header borrows never cover typed bytes.
use super::super::raw_heap::{MALLOC_ALIGNMENT, heap_allocate, heap_free};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

/// Actual `NCollection_IncAllocator::IBlock` header, with readonly safe field access.
///
/// Construction computes addresses without dereferencing them or acquiring ownership.
/// `let header = IncAllocatorBlock::new(std::ptr::null_mut(), 128);`
/// Native mutable atomic fields remain private because arbitrary writes could corrupt live owners.
#[repr(C)]
pub struct IncAllocatorBlock {
    current: AtomicPtr<u8>,
    available: AtomicUsize,
    next: Option<NonNull<Self>>,
    ordered_next: Option<NonNull<Self>>,
}

/// Copied canonical header fields; addresses are identities, never dereferenced by this record.
/// `let observation = header.observation();`
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IncBlockObservation {
    /// Address of the actual header (not its payload).
    pub address: usize,
    /// Native `CurPointer` address.
    pub current: usize,
    /// Native `AvailableSize` value.
    pub available: usize,
    /// Native `NextBlock` address, zero for null.
    pub next: usize,
    /// Native `NextOrderedBlock` address, zero for null.
    pub ordered_next: usize,
}

impl IncAllocatorBlock {
    /// Computes native initial cursor and null links; `IncAllocatorBlock::new(buffer, 128)`.
    /// The buffer is not dereferenced and must outlive any later unsafe use of its pointer.
    pub fn new(buffer: *mut u8, size: usize) -> Self {
        Self {
            current: AtomicPtr::new(buffer.wrapping_add(size_of::<Self>())),
            available: AtomicUsize::new(size),
            next: None,
            ordered_next: None,
        }
    }

    /// Copies native fields without exposing mutable atomics; `header.observation().available`.
    pub fn observation(&self) -> IncBlockObservation {
        IncBlockObservation {
            address: std::ptr::from_ref(self).addr(),
            current: self.current.load(Ordering::Relaxed).addr(),
            available: self.available.load(Ordering::Relaxed),
            next: self.next.map_or(0, |link| link.as_ptr().addr()),
            ordered_next: self.ordered_next.map_or(0, |link| link.as_ptr().addr()),
        }
    }
}

/// A valid allocated header. Only unsafe `release` ends validity of it and all copied links.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct IncBlockLink(NonNull<IncAllocatorBlock>);

impl IncBlockLink {
    pub(super) fn allocate(size: usize) -> Option<Self> {
        let bytes = size_of::<IncAllocatorBlock>().checked_add(size)?;
        let buffer = heap_allocate(bytes, MALLOC_ALIGNMENT, true)?;
        let header = buffer.cast::<IncAllocatorBlock>();
        // SAFETY: one fresh suitably aligned allocation includes the header and payload.
        unsafe { header.write(IncAllocatorBlock::new(buffer.as_ptr(), size)) };
        Some(Self(header))
    }

    pub(super) fn observation(self) -> IncBlockObservation {
        // SAFETY: links remain valid until the engine releases the ordered ownership chain.
        unsafe { self.0.as_ref() }.observation()
    }

    pub(super) fn next(self) -> Option<Self> {
        // SAFETY: the mutex excludes concurrent link writes; no header reference escapes.
        unsafe { self.0.as_ref() }.next.map(Self)
    }

    pub(super) fn ordered_next(self) -> Option<Self> {
        // SAFETY: creation-order links only point to other live headers in this engine.
        unsafe { self.0.as_ref() }.ordered_next.map(Self)
    }

    pub(super) fn set_next(self, next: Option<Self>) {
        // SAFETY: the engine lock excludes concurrent access; payload bytes are disjoint.
        unsafe { (*self.0.as_ptr()).next = next.map(|link| link.0) };
    }

    pub(super) fn set_ordered_next(self, next: Option<Self>) {
        // SAFETY: the engine lock excludes concurrent access; no whole allocation borrow occurs.
        unsafe { (*self.0.as_ptr()).ordered_next = next.map(|link| link.0) };
    }

    pub(super) fn current(self) -> NonNull<u8> {
        // SAFETY: a valid header's cursor is always within or just past its owned payload.
        let pointer = unsafe { self.0.as_ref() }.current.load(Ordering::Relaxed);
        NonNull::new(pointer).expect("incremental cursor: expected non-null allocated payload")
    }

    pub(super) fn available(self) -> usize {
        // SAFETY: this reads only the stable live header, not neighboring borrowed payloads.
        unsafe { self.0.as_ref() }.available.load(Ordering::Relaxed)
    }

    pub(super) fn bump(self, size: usize) -> NonNull<u8> {
        let available = self.available();
        assert!(
            size <= available,
            "incremental request {size}: expected at most {available}"
        );
        let current = self.current();
        // SAFETY: the checked remaining size keeps the advanced cursor in its allocation.
        let next = unsafe { current.add(size) };
        // SAFETY: only the header atomics are accessed; retained payload borrows stay disjoint.
        let header = unsafe { self.0.as_ref() };
        header.current.store(next.as_ptr(), Ordering::Relaxed);
        header.available.store(available - size, Ordering::Relaxed);
        current
    }

    pub(super) fn reset(self) {
        // SAFETY: the constructor placed this header at its original allocation's payload base.
        let start = unsafe { self.0.cast::<u8>().add(size_of::<IncAllocatorBlock>()) };
        let consumed = self.current().as_ptr().addr() - start.as_ptr().addr();
        // SAFETY: both atomics occupy only the stable header; caller excludes live typed owners.
        let header = unsafe { self.0.as_ref() };
        header.available.fetch_add(consumed, Ordering::Relaxed);
        header.current.store(start.as_ptr(), Ordering::Relaxed);
    }

    pub(super) unsafe fn release(self) {
        // SAFETY: caller relinquishes this header and every copied link; its original allocation
        // address/layout is preserved, unlike deriving a free pointer from a typed payload borrow.
        unsafe { heap_free(self.0.cast()) };
    }
}

// The actual heap Layout guarantees 16-byte alignment; the header preserves it for typed payloads.
const _: () = assert!(size_of::<IncAllocatorBlock>().is_multiple_of(MALLOC_ALIGNMENT));
const _: () = assert!(align_of::<IncAllocatorBlock>() <= MALLOC_ALIGNMENT);

#[cfg(test)]
#[path = "block_tests.rs"]
mod tests;
