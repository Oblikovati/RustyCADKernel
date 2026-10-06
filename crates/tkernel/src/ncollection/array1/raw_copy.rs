use super::{Array1, Storage};
use crate::ncollection::ArrayError;

#[allow(unsafe_code)]
impl<T: Copy> Array1<'_, T> {
    /// Performs native `copyAssign` directly on possibly overlapping raw buffers.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::Array1;
    /// let mut values = [1, 2, 3];
    /// let pointer = values.as_mut_ptr();
    /// unsafe { Array1::copy_assign_trivial_raw(pointer.add(1), pointer, 2); }
    /// assert_eq!(values, [1, 1, 2]);
    /// ```
    ///
    /// # Safety
    /// For nonzero `count`, `source` must be readable and `target` writable for
    /// `count` properly aligned `T` values with valid provenance; source values
    /// must be initialized. The regions may
    /// overlap. No references conflicting with either access may remain live.
    /// Zero count accepts null pointers and performs no accesses.
    pub unsafe fn copy_assign_trivial_raw(target: *mut T, source: *const T, count: usize) {
        if count == 0 {
            return;
        }
        // OCCT copyAssign selects memmove for trivially-copyable elements.
        unsafe { std::ptr::copy(source, target, count) };
    }

    /// Copies potentially overlapping native storage, preserving this array's bounds.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::Array1;
    /// let mut array = Array1::<i32>::with_size(2);
    /// let source = [3, 4];
    /// unsafe { array.copy_values_trivial_from_raw_parts(source.as_ptr(), 2).unwrap(); }
    /// assert_eq!(array.as_slice(), source);
    /// ```
    ///
    /// # Safety
    /// When lengths match and are nonzero, `source` must point to `source_len`
    /// initialized, readable `T` values in one allocation. It may overlap this
    /// array. No incompatible references may remain live during the copy.
    pub unsafe fn copy_values_trivial_from_raw_parts(
        &mut self,
        source: *const T,
        source_len: usize,
    ) -> Result<&mut Self, ArrayError> {
        if self.len() != source_len {
            return Err(ArrayError::DimensionMismatch {
                expected: self.len(),
                actual: source_len,
            });
        }
        unsafe { Self::copy_assign_trivial_raw(self.data_mut_ptr(), source, source_len) };
        Ok(self)
    }

    /// Assigns native raw storage and bounds, retaining equal-size borrowed storage.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::Array1;
    /// let mut array = Array1::<i32>::default();
    /// let source = [3, 4];
    /// unsafe { array.assign_trivial_from_raw_parts(source.as_ptr(), 2, -1); }
    /// assert_eq!(array.lower(), -1);
    /// ```
    ///
    /// # Safety
    /// For nonzero `source_len`, `source` must point to that many initialized,
    /// readable `T` values in one allocation. Overlap with this array is allowed;
    /// no incompatible references may remain live during the copy.
    #[allow(clippy::iter_cloned_collect)]
    pub unsafe fn assign_trivial_from_raw_parts(
        &mut self,
        source: *const T,
        source_len: usize,
        source_lower: i32,
    ) -> &mut Self {
        if self.len() == source_len {
            unsafe { self.copy_values_trivial_from_raw_parts(source, source_len) }.unwrap();
        } else {
            // Read before releasing old storage: the source can be a slice of that allocation.
            self.storage = Storage::Owned(unsafe { copy_trivial_raw_to_vec(source, source_len) });
        }
        self.lower = source_lower;
        self
    }
}

#[allow(clippy::iter_cloned_collect, unsafe_code)]
unsafe fn copy_trivial_raw_to_vec<T: Copy>(source: *const T, count: usize) -> Vec<T> {
    if count == 0 {
        return Vec::new();
    }
    // The caller guarantees a readable initialized range; copied bypasses Clone hooks.
    let values = unsafe { std::slice::from_raw_parts(source, count) };
    values.iter().copied().collect()
}
