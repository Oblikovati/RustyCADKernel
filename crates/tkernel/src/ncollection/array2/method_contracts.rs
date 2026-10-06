use super::{Array2, ArrayError};

impl<T> Array2<'_, T> {
    /// Returns row-major storage, or null before allocation; `array.data_ptr()`.
    pub fn data_ptr(&self) -> *const T {
        self.elements.data_ptr()
    }
    /// Returns writable row-major storage; shared views detach first; `array.data_mut_ptr()`.
    pub fn data_mut_ptr(&mut self) -> *mut T {
        self.elements.data_mut_ptr()
    }
    /// Returns the row-major initialized range end; `array.data_end_ptr()`.
    pub fn data_end_ptr(&self) -> *const T {
        self.elements.data_end_ptr()
    }
    /// Returns the writable row-major initialized range end; `array.data_end_mut_ptr()`.
    pub fn data_end_mut_ptr(&mut self) -> *mut T {
        self.elements.data_end_mut_ptr()
    }
    /// Preserves storage and bounds for native self assignment; `array.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
    pub(super) fn adopt_rectangle(&mut self, source: &Array2<'_, T>) {
        self.rows = source.rows;
        self.cols = source.cols;
        self.row_lower = source.row_lower;
        self.col_lower = source.col_lower;
    }
    pub(super) fn check_rectangle(&self, source: &Array2<'_, T>) -> Result<(), ArrayError> {
        self.check_dimensions(source.rows, source.cols)
    }
    pub(super) fn check_dimensions(&self, rows: usize, cols: usize) -> Result<(), ArrayError> {
        if (self.rows, self.cols) == (rows, cols) {
            return Ok(());
        }
        Err(ArrayError::ShapeMismatch {
            expected: (self.rows, self.cols),
            actual: (rows, cols),
        })
    }
}
impl<T: Clone> Array2<'_, T> {
    /// Copies equally shaped contents and returns the receiver; `array.copy_values_returning(&source)?`.
    pub fn copy_values_returning(
        &mut self,
        source: &Array2<'_, T>,
    ) -> Result<&mut Self, ArrayError> {
        self.copy_values(source)?;
        Ok(self)
    }
}
impl<T: Copy> Array2<'_, T> {
    /// Selects native trivial copying without Clone hooks; `array.assign_trivial(&source)`.
    pub fn assign_trivial(&mut self, source: &Array2<'_, T>) -> &mut Self {
        self.elements.assign_trivial(&source.elements);
        self.adopt_rectangle(source);
        self
    }
    /// Copies trivial values while retaining rectangle bounds; `array.copy_values_trivial(&source)?`.
    pub fn copy_values_trivial(&mut self, source: &Array2<'_, T>) -> Result<&mut Self, ArrayError> {
        self.check_rectangle(source)?;
        self.elements.copy_values_trivial(&source.elements)?;
        Ok(self)
    }
    /// Copies the rectangle without Clone hooks; `array.clone_trivial()`.
    pub fn clone_trivial(&self) -> Self {
        Self {
            elements: self.elements.clone_trivial(),
            row_lower: self.row_lower,
            col_lower: self.col_lower,
            rows: self.rows,
            cols: self.cols,
        }
    }
}
