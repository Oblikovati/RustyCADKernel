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
        let _ = self.as_mut_slice();
        if matches!(&self.storage, Storage::Owned(values) if values.capacity() == 0) {
            return std::ptr::null_mut();
        }
        self.as_mut_slice().as_mut_ptr()
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
