impl<T: Default + Clone> Array2<'_, T> {
    /// Resizes using OCCT's packed-copy rule; `array.resize_bounds(1,3,1,4,true)?`.
    /// The convenience policy uses Clone::clone_from; explicit donor mutation uses
    /// [`Self::resize_bounds_with_assignment`].
    pub fn resize_bounds(
        &mut self,
        rl: i32,
        ru: i32,
        cl: i32,
        cu: i32,
        copy: bool,
    ) -> Result<(), ArrayError> {
        self.resize_clone_bounds(rl, ru, cl, cu, copy, false)
    }
    /// Resizes preserving common row/column positions; `array.resize_trim_bounds(1,3,1,4,true)?`.
    /// Uses Clone::clone_from for the element assignment policy.
    pub fn resize_trim_bounds(
        &mut self,
        rl: i32,
        ru: i32,
        cl: i32,
        cu: i32,
        copy: bool,
    ) -> Result<(), ArrayError> {
        self.resize_clone_bounds(rl, ru, cl, cu, copy, true)
    }
    /// Resizes counts while retaining origins; `array.resize(3,4,true)?`.
    /// Uses Clone::clone_from; use [`Self::resize_with_assignment`] for mutable donors.
    pub fn resize(&mut self, rows: usize, cols: usize, copy: bool) -> Result<(), ArrayError> {
        self.resize_clone_counts(rows, cols, copy, false)
    }
    /// Resizes counts preserving the common rectangle; `array.resize_with_trim(3,4,true)?`.
    /// Uses Clone::clone_from for the element assignment policy.
    pub fn resize_with_trim(&mut self, rows: usize, cols: usize, copy: bool) -> Result<(), ArrayError> {
        self.resize_clone_counts(rows, cols, copy, true)
    }
}
