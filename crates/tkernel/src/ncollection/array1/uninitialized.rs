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
impl<'a, T> Array1<'a, MaybeUninit<T>> {
    /// Transfers exclusively initialized storage without requiring `Copy` or `Clone`.
    /// Shared views return unchanged in `Err`, preserving their existing detach policy.
    /// ```
    /// use rustycad_tkernel::ncollection::Array1;
    /// let mut slots = Array1::<String>::with_uninit_size(1);
    /// slots.as_mut_slice()[0].write("ready".into());
    /// let values = unsafe { slots.try_assume_init_exclusive() }.unwrap();
    /// assert_eq!(values.as_slice(), ["ready"]);
    /// ```
    ///
    /// # Safety
    /// For owned or exclusively borrowed storage, every actual length slot must
    /// contain a valid initialized `T`, including slots beyond matrix bounds after
    /// backing replacement or resize. Spare capacity and unborrowed tails are excluded.
    /// Owned payloads will drop with the result; borrowed payloads remain the donor's
    /// responsibility after the result's borrow ends. Retained pointers must obey
    /// the resulting borrows. Shared storage is rejected without reading its slots,
    /// and may still be uninitialized. Ownership transfers without element operations.
    pub unsafe fn try_assume_init_exclusive(self) -> Result<Array1<'a, T>, Self> {
        let lower = self.lower;
        // SAFETY: the caller initializes every actual exclusive slot before conversion.
        unsafe { initialized_exclusive_storage(self.storage) }
            .map(|storage| Array1 { lower, storage })
            .map_err(|storage| Self { lower, storage })
    }
}

#[allow(unsafe_code)]
unsafe fn initialized_exclusive_storage<T>(
    storage: Storage<'_, MaybeUninit<T>>,
) -> Result<Storage<'_, T>, Storage<'_, MaybeUninit<T>>> {
    match storage {
        // SAFETY: layout and allocation ownership transfer with every live slot initialized.
        Storage::Owned(slots) => Ok(Storage::Owned(unsafe { initialized_vector(slots) })),
        // SAFETY: the exclusive slice retains its complete length and original lifetime.
        Storage::Borrowed(slots) => Ok(Storage::Borrowed(unsafe {
            std::slice::from_raw_parts_mut(slots.as_mut_ptr().cast::<T>(), slots.len())
        })),
        shared @ Storage::Shared(..) => Err(shared),
    }
}

#[allow(unsafe_code)]
unsafe fn initialized_storage<T: Copy>(storage: Storage<'_, MaybeUninit<T>>) -> Storage<'_, T> {
    match unsafe { initialized_exclusive_storage(storage) } {
        Ok(storage) => storage,
        // SAFETY: the caller guarantees initialized values for the full shared lifetime.
        Err(Storage::Shared(slots, _)) => Storage::Shared(
            unsafe { std::slice::from_raw_parts(slots.as_ptr().cast::<T>(), slots.len()) },
            <[T]>::to_vec,
        ),
        Err(_) => unreachable!("exclusive storage always converts without rejection"),
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
    #[test]
    #[allow(unsafe_code)]
    fn exclusive_noncopy_vector_retains_spare_capacity_and_allocation() {
        let mut slots = Vec::with_capacity(8);
        slots.push(MaybeUninit::new(String::from("owned")));
        let (pointer, capacity) = (slots.as_ptr().cast::<String>(), slots.capacity());
        let storage = super::Storage::Owned(slots);
        let values = unsafe { super::initialized_exclusive_storage(storage) };
        let Ok(super::Storage::Owned(values)) = values else {
            panic!("owned conversion")
        };
        assert_eq!((values.as_ptr(), values.capacity()), (pointer, capacity));
        assert_eq!(values, ["owned"]);
    }
}
