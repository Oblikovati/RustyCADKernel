//! The OS-owned backing used for mapped `Standard_MMgrOpt` allocations.
#![allow(unsafe_code)]

#[cfg(unix)]
use std::ptr::NonNull;

/// An anonymous mapping whose live range is unmapped exactly once.
#[cfg(unix)]
pub(super) struct MappedRegion {
    start: NonNull<u8>,
    length: usize,
    page_size: usize,
}

#[cfg(unix)]
// SAFETY: the address is an owned mapping, methods borrow its bytes, and Drop unmaps once.
unsafe impl Send for MappedRegion {}

#[cfg(unix)]
// SAFETY: shared access only exposes immutable bytes; mutation requires exclusive access.
unsafe impl Sync for MappedRegion {}

#[cfg(unix)]
impl MappedRegion {
    /// Maps zero-filled writable pages; `MappedRegion::new(4096)`.
    pub(super) fn new(length: usize) -> Option<Self> {
        let page_size = super::os_page::os_page_size();
        if length > isize::MAX as usize || page_size == 0 || !length.is_multiple_of(page_size) {
            return None;
        }
        let start = map_anonymous(length)?;
        Some(Self {
            start,
            length,
            page_size,
        })
    }

    /// Returns the first address while this mapping remains owned.
    pub(super) fn address(&self) -> usize {
        self.start.as_ptr() as usize
    }

    /// Borrows the live mapped bytes; `mapping.bytes()[0]`.
    pub(super) fn bytes(&self) -> &[u8] {
        // SAFETY: the mapping owns `length` readable bytes until Drop or a prefix trim.
        unsafe { std::slice::from_raw_parts(self.start.as_ptr(), self.length) }
    }

    /// Borrows the live mapped bytes mutably; `mapping.bytes_mut()[0] = 1`.
    pub(super) fn bytes_mut(&mut self) -> &mut [u8] {
        // SAFETY: AddressSpace gives exclusive access to this region while borrowing it.
        unsafe { std::slice::from_raw_parts_mut(self.start.as_ptr(), self.length) }
    }

    /// Releases a page-aligned prefix and keeps the suffix mapping alive.
    pub(super) fn release_prefix(&mut self, length: usize) -> bool {
        if length == 0 || length >= self.length || !length.is_multiple_of(self.page_size) {
            return false;
        }
        let suffix = unsafe { self.start.as_ptr().add(length) };
        // SAFETY: caller supplies a page-aligned prefix within this owned mapping.
        if unsafe { libc::munmap(self.start.as_ptr().cast(), length) } != 0 {
            return false;
        }
        self.start = NonNull::new(suffix).expect("a mapped suffix has a non-null address");
        self.length -= length;
        true
    }
}

#[cfg(unix)]
fn map_anonymous(length: usize) -> Option<NonNull<u8>> {
    if length == 0 {
        return None;
    }
    // SAFETY: mmap receives no caller pointers and returns an owned mapping or MAP_FAILED.
    let address = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            length,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        )
    };
    if address == libc::MAP_FAILED {
        return None;
    }
    if address.is_null() {
        // SAFETY: a successful zero-address mapping is owned here and rejected deliberately.
        let _ = unsafe { libc::munmap(address, length) };
        return None;
    }
    NonNull::new(address.cast())
}

#[cfg(unix)]
impl Drop for MappedRegion {
    fn drop(&mut self) {
        // SAFETY: this object uniquely owns the current live mapping range.
        let _ = unsafe { libc::munmap(self.start.as_ptr().cast(), self.length) };
    }
}
