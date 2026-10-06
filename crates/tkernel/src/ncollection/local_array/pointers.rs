use super::LocalArray;

impl<T, const N: usize> LocalArray<T, N> {
    /// Returns the storage beginning, including retained empty allocations; `array.data_ptr()`.
    pub fn data_ptr(&self) -> *const T {
        self.elements.as_ptr()
    }

    /// Returns mutable storage without creating a slice; `array.data_mut_ptr()`.
    pub fn data_mut_ptr(&mut self) -> *mut T {
        self.elements.as_mut_ptr()
    }

    /// Returns the initialized range's end; `array.data_end_ptr()`.
    pub fn data_end_ptr(&self) -> *const T {
        self.data_ptr().wrapping_add(self.len())
    }

    /// Returns the mutable initialized range's end; `array.data_end_mut_ptr()`.
    pub fn data_end_mut_ptr(&mut self) -> *mut T {
        self.data_mut_ptr().wrapping_add(self.len())
    }
}
