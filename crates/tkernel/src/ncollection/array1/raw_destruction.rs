use super::Array1;

#[allow(unsafe_code)]
impl<T> Array1<'_, T> {
    /// Destroys native nontrivial slots in ascending order without freeing their backing allocation.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::Array1;
    /// use std::mem::MaybeUninit;
    /// let mut slots = [MaybeUninit::new(String::from("owned"))];
    /// unsafe { Array1::destroy_nontrivial_raw(slots.as_mut_ptr().cast::<String>(), 0, 1); }
    /// // The slot is now dead; MaybeUninit prevents a second automatic String drop.
    /// ```
    ///
    /// # Safety
    /// For a nonempty range and nonzero-sized `T`, each `pointer.add(index)` in
    /// `from..to` must identify an aligned, initialized, live `T` owned by the caller.
    /// Pointer arithmetic must stay within one allocation and `isize::MAX` bytes,
    /// with valid provenance and no conflicting references. Destructors must not
    /// invalidate any remaining slot or its allocation through retained aliases.
    ///
    /// Every visited slot becomes dead, including a slot whose destructor panics.
    /// The caller must track the visited prefix when recovering, then destroy only
    /// surviving slots. Dead slots must not later be read or dropped as live `T`.
    /// Use raw/MaybeUninit backing or otherwise remove automatic drop ownership
    /// before calling; a live Vec<T> must not subsequently drop the same slots.
    /// Empty or reversed ranges accept null pointers and perform no accesses.
    ///
    /// # Panics
    /// Nonempty zero-sized `T` is rejected before pointer arithmetic. Destructor
    /// panic payloads propagate unchanged; remaining slots are not destroyed.
    pub unsafe fn destroy_nontrivial_raw(pointer: *mut T, from: usize, to: usize) {
        assert!(
            from >= to || size_of::<T>() > 0,
            "element size 0; expected nonzero size for destruction range {from}..{to}"
        );
        for offset in from..to {
            // Native destroy ends each lifetime before advancing to the next slot.
            unsafe { std::ptr::drop_in_place(pointer.add(offset)) };
        }
    }
}
