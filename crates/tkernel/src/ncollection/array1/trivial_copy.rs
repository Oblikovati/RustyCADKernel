use super::{Array1, Storage};
use crate::ncollection::ArrayError;

// Clippy's to_vec alternative invokes Clone; native trivial copying must bypass that hook.
#[allow(clippy::iter_cloned_collect, unsafe_code)]
impl<T: Copy> Array1<'_, T> {
    /// Selects OCCT's trivially-copyable assignment policy explicitly; `array.assign_trivial(&source)`.
    pub fn assign_trivial(&mut self, source: &Array1<'_, T>) -> &mut Self {
        // Safe array borrows keep source storage valid through the raw compatibility engine.
        unsafe { self.assign_trivial_from_raw_parts(source.data_ptr(), source.len(), source.lower) }
    }
    /// Copies trivial values without rebasing the destination; `array.copy_values_trivial(&source)?`.
    pub fn copy_values_trivial(&mut self, source: &Array1<'_, T>) -> Result<&mut Self, ArrayError> {
        // Safe array borrows are a disjoint subset of the native overlap-capable contract.
        unsafe { self.copy_values_trivial_from_raw_parts(source.data_ptr(), source.len()) }
    }
    /// Copies bounds and initialized values without Clone hooks; `array.clone_trivial()`.
    pub fn clone_trivial(&self) -> Self {
        Self {
            lower: self.lower,
            storage: Storage::Owned(self.as_slice().iter().copied().collect()),
        }
    }
}
