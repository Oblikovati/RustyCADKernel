//! Query the host allocator independently of Martensite's owned storage adapter.
#![allow(unsafe_code)]

/// Observe the native zero-size result; `existing_zero_reallocation_returns_null()`.
pub(super) fn existing_zero_reallocation_returns_null() -> bool {
    // Standard.cxx at 3d097a0 calls realloc directly and raises only on null.
    // SAFETY: the allocated pointer is passed once to realloc; only its result is freed.
    let moved = unsafe {
        let original = libc::malloc(8);
        assert!(
            !original.is_null(),
            "malloc(8) returned null; expected probe storage"
        );
        libc::realloc(original, 0)
    };
    let refused = moved.is_null();
    if !refused {
        // SAFETY: a non-null result is a live C allocation, including a zero-size one.
        unsafe { libc::free(moved) };
    }
    refused
}
