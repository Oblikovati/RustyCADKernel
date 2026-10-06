//! Test-only repair; kernel behavior must never call this unpublished-object cleanup.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::LinearVector;

/// Releases precisely known completed spare objects after native state was asserted.
/// # Safety
/// Every slot in [len, len + completed) must be live, unpublished and not overwritten
/// since construction. This function ends those lifetimes; never repair them twice.
pub unsafe fn destroy_completed_tail<T>(vector: &mut LinearVector<T>, completed: usize) {
    assert!(completed <= vector.capacity() - vector.len());
    let beginning = vector.len();
    let storage = vector.data_mut_ptr();
    for index in beginning..beginning + completed {
        // SAFETY: the caller proves this allocated spare slot still holds one live T.
        unsafe { storage.add(index).drop_in_place() };
    }
}
