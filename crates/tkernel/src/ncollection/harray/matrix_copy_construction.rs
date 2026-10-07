use super::{Array2, HArray2};

impl<T: Clone + Send + Sync + 'static> HArray2<T> {
    /// Copies a rectangle and its bounds into a new shared array; `HArray2::from_array(&grid)`.
    pub fn from_array(values: &Array2<'_, T>) -> Self {
        let mut copy = Array2::default();
        copy.assign(values);
        Self::from_owned(copy)
    }
}
impl<T: Copy + Send + Sync + 'static> HArray2<T> {
    /// Copies native trivial values without invoking Clone; `HArray2::from_array_trivial(&grid)`.
    ///
    /// The explicit policy preserves the independent rectangle and inherited flat bounds,
    /// including a flat length changed through `Array2::change_array1`.
    pub fn from_array_trivial(values: &Array2<'_, T>) -> Self {
        let mut copy = Array2::default();
        copy.assign_trivial(values);
        Self::from_owned(copy)
    }
}
