use super::LinearVector;

impl<T> LinearVector<T> {
    /// Returns null before allocation and after release; `vector.data_ptr()`.
    pub fn data_ptr(&self) -> *const T {
        if !self.has_data() {
            return std::ptr::null();
        }
        self.elements.as_ptr()
    }
    /// Returns mutable allocated storage or null; `vector.data_mut_ptr()`.
    pub fn data_mut_ptr(&mut self) -> *mut T {
        if !self.has_data() {
            return std::ptr::null_mut();
        }
        self.elements.as_mut_ptr()
    }
    /// Returns the initialized range's end, as const end/cend; `vector.data_end_ptr()`.
    pub fn data_end_ptr(&self) -> *const T {
        self.data_ptr().wrapping_add(self.len())
    }
    /// Returns the mutable initialized range's end; `vector.data_end_mut_ptr()`.
    pub fn data_end_mut_ptr(&mut self) -> *mut T {
        self.data_mut_ptr().wrapping_add(self.len())
    }
}
