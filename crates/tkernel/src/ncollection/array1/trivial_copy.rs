use super::{Array1, Storage};
use crate::ncollection::ArrayError;

// Clippy's to_vec alternative invokes Clone; native trivial copying must bypass that hook.
#[allow(clippy::iter_cloned_collect)]
impl<T: Copy> Array1<'_, T> {
    /// Selects OCCT's trivially-copyable assignment policy explicitly; `array.assign_trivial(&source)`.
    pub fn assign_trivial(&mut self, source: &Array1<'_, T>) -> &mut Self {
        if self.len() == source.len() {
            self.as_mut_slice().copy_from_slice(source.as_slice());
        } else {
            // Copied bypasses custom Clone hooks, unlike Vec::extend_from_slice.
            self.storage = Storage::Owned(source.as_slice().iter().copied().collect());
        }
        self.lower = source.lower;
        self
    }
    /// Copies trivial values without rebasing the destination; `array.copy_values_trivial(&source)?`.
    pub fn copy_values_trivial(&mut self, source: &Array1<'_, T>) -> Result<&mut Self, ArrayError> {
        if self.len() != source.len() {
            return Err(ArrayError::DimensionMismatch {
                expected: self.len(),
                actual: source.len(),
            });
        }
        self.as_mut_slice().copy_from_slice(source.as_slice());
        Ok(self)
    }
    /// Copies bounds and initialized values without Clone hooks; `array.clone_trivial()`.
    pub fn clone_trivial(&self) -> Self {
        Self {
            lower: self.lower,
            storage: Storage::Owned(self.as_slice().iter().copied().collect()),
        }
    }
}
