use super::Array1;
use std::mem::MaybeUninit;

#[allow(unsafe_code)]
impl<T> Array1<'_, T> {
    /// Constructs native nontrivial copies without automatically dropping a completed prefix.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::Array1;
    /// use std::mem::MaybeUninit;
    /// let source = [7];
    /// let mut target = [MaybeUninit::uninit()];
    /// unsafe {
    ///     Array1::copy_construct_nontrivial_raw(target.as_mut_ptr(), source.as_ptr(), 1, |v| *v);
    ///     assert_eq!(target[0].assume_init(), 7);
    /// }
    /// ```
    ///
    /// The callback builds a value before writing its slot. Constructors that observe
    /// their placement address or partially write a failing slot need a separate policy.
    ///
    /// # Safety
    /// For nonzero count and nonzero-sized `T`, `target` must designate writable,
    /// aligned storage for `count` `MaybeUninit<T>` slots, and `source` must
    /// designate that many initialized readable `T` values. Each range must fit
    /// within one allocation and `isize::MAX` bytes with valid pointer provenance.
    /// Source and destination ranges must be disjoint; no conflicting references
    /// may remain live. The callback must not reenter either range through aliases.
    ///
    /// The caller owns every successfully constructed destination slot, including
    /// the completed prefix if the callback panics. Track completed callbacks when
    /// recovery needs that prefix length, then move out or drop exactly those slots.
    /// Neither unwinding nor returning destroys them. The failing slot and suffix
    /// remain untouched. Existing destination payloads are not automatically dropped.
    /// Zero count permits null pointers and invokes no callbacks.
    ///
    /// # Panics
    /// Nonzero count with zero-sized `T` is rejected before pointer arithmetic.
    /// Constructor panic payloads propagate unchanged.
    pub unsafe fn copy_construct_nontrivial_raw(
        target: *mut MaybeUninit<T>,
        source: *const T,
        count: usize,
        mut construct: impl FnMut(&T) -> T,
    ) {
        assert!(
            count == 0 || size_of::<T>() > 0,
            "element size 0; expected nonzero size for {count} construction slots"
        );
        unsafe { construct_raw_range(target, 0, count, |offset| construct(&*source.add(offset))) };
    }

    /// Default-constructs a native nontrivial raw range with caller-owned prefix recovery.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::Array1;
    /// use std::mem::MaybeUninit;
    /// let mut slots = [MaybeUninit::<String>::uninit()];
    /// unsafe {
    ///     Array1::construct_nontrivial_raw(slots.as_mut_ptr(), 0, 1, String::new);
    ///     slots[0].assume_init_drop();
    /// }
    /// ```
    ///
    /// The callback creates a value before writing its slot. Native constructors
    /// observing their placement address or failing-slot bytes need a separate policy.
    ///
    /// # Safety
    /// For a nonempty range and nonzero-sized `T`, each `target.add(index)` in
    /// `from..to` must designate writable aligned `MaybeUninit<T>` storage within
    /// one allocation and `isize::MAX` bytes, with valid provenance and no conflicting
    /// references. The callback must not reenter the destination through aliases.
    /// Existing payloads are not implicitly destroyed when slots are overwritten.
    ///
    /// Caller owns every completed slot, including the prefix after a panic; track
    /// successful callbacks to recover its length, then move or destroy only those
    /// initialized slots. Neither unwinding nor returning cleans this prefix. Empty
    /// and reversed ranges permit null pointers without invoking the callback.
    ///
    /// # Panics
    /// Nonempty zero-sized `T` is rejected before pointer arithmetic; constructor
    /// panic payloads propagate unchanged without initializing remaining slots.
    pub unsafe fn construct_nontrivial_raw(
        target: *mut MaybeUninit<T>,
        from: usize,
        to: usize,
        mut construct: impl FnMut() -> T,
    ) {
        assert!(
            from >= to || size_of::<T>() > 0,
            "element size 0; expected nonzero size for construction range {from}..{to}"
        );
        unsafe { construct_raw_range(target, from, to, |_| construct()) };
    }
}

#[allow(unsafe_code)]
unsafe fn construct_raw_range<T>(
    target: *mut MaybeUninit<T>,
    from: usize,
    to: usize,
    mut construct: impl FnMut(usize) -> T,
) {
    for offset in from..to {
        let value = construct(offset);
        // MaybeUninit retains the native caller-owned prefix during unwinding.
        unsafe { target.add(offset).write(MaybeUninit::new(value)) };
    }
}
