//! Explicit storage state for OCCT's trivial default-construction branch.
use super::{Array1, Storage, checked_array_extent};
use crate::ncollection::ArrayError;
use std::mem::MaybeUninit;

impl<'a, T> Array1<'a, T> {
    /// Allocates slots without constructing `T`, modeling native trivial construction.
    /// `let slots = Array1::<i32>::with_uninit_size(3); assert_eq!(slots.len(), 3);`.
    pub fn with_uninit_size(size: usize) -> Array1<'a, MaybeUninit<T>> {
        let slots = std::iter::repeat_with(MaybeUninit::uninit)
            .take(size)
            .collect();
        Array1 {
            lower: 0,
            storage: Storage::Owned(slots),
        }
    }

    /// Allocates uninitialized slots with inclusive bounds without requiring `Default`.
    /// `let slots = Array1::<i32>::with_uninit_bounds(-1, 2)?;`.
    pub fn with_uninit_bounds(
        lower: i32,
        upper: i32,
    ) -> Result<Array1<'a, MaybeUninit<T>>, ArrayError> {
        let mut array = Self::with_uninit_size(checked_array_extent(lower, upper)?);
        array.lower = lower;
        Ok(array)
    }
}

#[allow(unsafe_code)]
impl<'a, T: Copy> Array1<'a, MaybeUninit<T>> {
    /// Consumes initialized slots without copying, reallocating or changing ownership.
    /// After writing every slot, use `let values = unsafe { slots.assume_init() };`.
    ///
    /// # Safety
    /// Every element in the actual storage slice must contain a valid initialized `T`.
    /// This includes slots exposed by earlier storage replacement or resize, regardless
    /// of external matrix bounds. Retained pointers must obey the resulting borrows.
    /// Spare owned capacity is not exposed and need not be initialized.
    pub unsafe fn assume_init(self) -> Array1<'a, T> {
        Array1 {
            lower: self.lower,
            // SAFETY: the caller initializes every slot in the complete flat storage.
            storage: unsafe { initialized_storage(self.storage) },
        }
    }
}

#[allow(unsafe_code)]
unsafe fn initialized_storage<T: Copy>(storage: Storage<'_, MaybeUninit<T>>) -> Storage<'_, T> {
    match storage {
        // SAFETY: ownership and allocation layout are retained; all elements are initialized.
        Storage::Owned(slots) => Storage::Owned(unsafe { initialized_vector(slots) }),
        // SAFETY: MaybeUninit<T> has T's layout and the exclusive lifetime is unchanged.
        Storage::Borrowed(slots) => Storage::Borrowed(unsafe {
            std::slice::from_raw_parts_mut(slots.as_mut_ptr().cast::<T>(), slots.len())
        }),
        // SAFETY: the caller guarantees initialized values for the full shared lifetime.
        Storage::Shared(slots, _) => Storage::Shared(
            unsafe { std::slice::from_raw_parts(slots.as_ptr().cast::<T>(), slots.len()) },
            <[T]>::to_vec,
        ),
    }
}

#[allow(unsafe_code)]
unsafe fn initialized_vector<T>(slots: Vec<MaybeUninit<T>>) -> Vec<T> {
    let (pointer, length, capacity) = slots.into_raw_parts();
    // SAFETY: MaybeUninit<T> has identical size/alignment, and the caller initialized
    // every element. The original allocation, length and capacity transfer exactly once.
    unsafe { Vec::from_raw_parts(pointer.cast::<T>(), length, capacity) }
}

#[cfg(test)]
mod tests {
    use super::initialized_vector;
    use std::mem::MaybeUninit;

    #[test]
    #[allow(unsafe_code)]
    fn initialized_vector_retains_spare_uninitialized_capacity_without_allocating() {
        let mut slots = Vec::with_capacity(8);
        slots.extend([MaybeUninit::new(7_u32), MaybeUninit::new(9_u32)]);
        let pointer = slots.as_ptr().cast::<u32>();
        let capacity = slots.capacity();
        // SAFETY: all length elements are initialized; spare capacity remains unexposed.
        let values = unsafe { initialized_vector(slots) };
        assert_eq!(values.as_ptr(), pointer);
        assert_eq!(values.capacity(), capacity);
        assert_eq!(values, [7, 9]);
    }
}
