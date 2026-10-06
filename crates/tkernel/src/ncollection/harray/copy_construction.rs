//! Wrapper copy construction chooses ordinary Clone or native trivial-copy policy explicitly.
use super::{Array1, HArray1};

impl<T: Clone + Send + Sync + 'static> HArray1<T> {
    /// Copies an array and its bounds into a new shared array; `HArray1::from_array(&values)`.
    pub fn from_array(values: &Array1<'_, T>) -> Self {
        Self::from_owned(owned_copy(values))
    }
}
impl<T: Copy + Send + Sync + 'static> HArray1<T> {
    /// Copies bounds and initialized values without invoking Clone hooks.
    /// The caller selects OCCT's trivially-copyable item policy; borrowed sources become owned copies.
    /// `let shared = HArray1::from_array_trivial(&Array1::<i32>::with_size(3))`.
    pub fn from_array_trivial(values: &Array1<'_, T>) -> Self {
        let mut copy = Array1::default();
        copy.assign_trivial(values);
        Self::from_owned(copy)
    }
}

// A copy that owns its storage even when `values` views a borrowed buffer, as the
// native copy constructor always allocates.
fn owned_copy<T: Clone>(values: &Array1<'_, T>) -> Array1<'static, T> {
    let mut copy = Array1::default();
    copy.assign(values);
    copy
}
