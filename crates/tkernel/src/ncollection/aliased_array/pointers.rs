use super::{AliasedArray, ArrayStorage, ArrayValue};
use crate::standard::Failure;

impl AliasedArray<'_> {
    /// Return native `Data<T>()`, retaining null for empty storage; `array.data_ptr::<i32>()?`.
    /// Raw pointer access validates stride but does not promise alignment or a live reference.
    pub fn data_ptr<T: ArrayValue>(&self) -> Result<*const T, Failure> {
        self.check_type::<T>(false)?;
        if matches!(self.storage, ArrayStorage::Empty) {
            return Ok(std::ptr::null());
        }
        Ok(self.bytes().as_ptr().cast())
    }

    /// Return native `ChangeData<T>()`; `array.data_mut_ptr::<i32>()?`.
    /// Dereferencing the pointer requires the caller to preserve alignment and exclusive access.
    pub fn data_mut_ptr<T: ArrayValue>(&mut self) -> Result<*mut T, Failure> {
        self.check_type::<T>(false)?;
        if matches!(self.storage, ArrayStorage::Empty) {
            return Ok(std::ptr::null_mut());
        }
        Ok(self.bytes_mut().as_mut_ptr().cast())
    }

    /// Return an element's untyped address using native stride; `array.byte_value_ptr(1)?`.
    pub fn byte_value_ptr(&self, index: usize) -> Result<*const u8, Failure> {
        let offset: usize = self.offset(index)?;
        Ok(self.bytes().as_ptr().wrapping_add(offset))
    }

    /// Return a mutable element address using native stride; `array.change_byte_value_ptr(1)?`.
    pub fn change_byte_value_ptr(&mut self, index: usize) -> Result<*mut u8, Failure> {
        let offset: usize = self.offset(index)?;
        Ok(self.bytes_mut().as_mut_ptr().wrapping_add(offset))
    }
}
