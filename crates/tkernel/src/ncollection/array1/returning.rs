use super::Array1;
use crate::ncollection::ArrayError;

impl<T: Clone> Array1<'_, T> {
    /// Copies values and returns this array without changing bounds; `array.copy_values_returning(&source)?`.
    pub fn copy_values_returning(
        &mut self,
        source: &Array1<'_, T>,
    ) -> Result<&mut Self, ArrayError> {
        self.copy_values(source)?;
        Ok(self)
    }
}
impl<T> Array1<'_, T> {
    /// Preserves storage and bounds for native self assignment or self CopyValues; `array.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
}
