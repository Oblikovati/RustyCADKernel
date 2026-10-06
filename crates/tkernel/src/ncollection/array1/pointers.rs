use super::{Array1, Storage};

impl<T> Array1<'_, T> {
    /// Returns storage in index order, or null for an unallocated owner; `array.data_ptr()`.
    pub fn data_ptr(&self) -> *const T {
        if matches!(&self.storage, Storage::Owned(values) if values.capacity() == 0) {
            return std::ptr::null();
        }
        self.as_slice().as_ptr()
    }
    /// Returns writable storage; a shared view detaches first; `array.data_mut_ptr()`.
    pub fn data_mut_ptr(&mut self) -> *mut T {
        if let Storage::Shared(values, detach) = self.storage {
            self.storage = Storage::Owned(detach(values));
        }
        match &mut self.storage {
            Storage::Owned(values) if values.capacity() == 0 => std::ptr::null_mut(),
            Storage::Owned(values) => values.as_mut_ptr(),
            // A slice method would reborrow the whole buffer and invalidate earlier raw aliases.
            Storage::Borrowed(values) => (&raw mut **values).cast::<T>(),
            Storage::Shared(..) => unreachable!("shared storage must detach before mutable access"),
        }
    }
    /// Returns the initialized range end, including empty borrowed buffers; `array.data_end_ptr()`.
    pub fn data_end_ptr(&self) -> *const T {
        self.data_ptr().wrapping_add(self.len())
    }
    /// Returns the writable range end; `array.data_end_mut_ptr()`.
    pub fn data_end_mut_ptr(&mut self) -> *mut T {
        self.data_mut_ptr().wrapping_add(self.len())
    }
}
