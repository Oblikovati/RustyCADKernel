use super::{AliasedArray, ArrayValue};
use crate::standard::Failure;

impl<'a> AliasedArray<'a> {
    /// Allocate using OCCT's signed length overload; `AliasedArray::new_signed(4, 3)?`.
    pub fn new_signed(stride: usize, count: i32) -> Result<Self, Failure> {
        Self::new(stride, Self::checked_size(count)?)
    }

    /// Wrap the requested external prefix using signed length; `AliasedArray::from_slice_signed(&mut values, 2)?`.
    pub fn from_slice_signed<T: ArrayValue>(
        values: &'a mut [T],
        count: i32,
    ) -> Result<Self, Failure> {
        let count: usize = Self::checked_size(count)?;
        let available: usize = values.len();
        let prefix: &mut [T] = values.get_mut(..count).ok_or_else(|| {
            Failure::with_type(
                "Standard_RangeError",
                &format!("AliasedArray length {count}; expected 1..={available} external elements"),
                "",
            )
        })?;
        Self::from_slice(prefix)
    }

    /// Resize through OCCT's signed length overload; `array.resize_signed(3, true)?`.
    pub fn resize_signed(&mut self, count: i32, copy: bool) -> Result<(), Failure> {
        self.resize(Self::checked_size(count)?, copy)
    }

    /// Read an element after the native signed-index check; `array.value_signed::<i32>(0)?`.
    pub fn value_signed<T: ArrayValue>(&self, index: i32) -> Result<&T, Failure> {
        self.value(checked_signed_index(index)?)
    }

    /// Change an element after the native signed-index check; `array.change_value_signed::<i32>(0)?`.
    pub fn change_value_signed<T: ArrayValue>(&mut self, index: i32) -> Result<&mut T, Failure> {
        self.change_value(checked_signed_index(index)?)
    }

    /// Read an element prefix after the native signed-index check; `array.value2_signed::<u8>(0)?`.
    pub fn value2_signed<T: ArrayValue>(&self, index: i32) -> Result<&T, Failure> {
        self.value2(checked_signed_index(index)?)
    }

    /// Change an element prefix after the native signed-index check; `array.change_value2_signed::<u8>(0)?`.
    pub fn change_value2_signed<T: ArrayValue>(&mut self, index: i32) -> Result<&mut T, Failure> {
        self.change_value2(checked_signed_index(index)?)
    }

    /// Return native `value(int)` after rejecting negative indices; `array.byte_value_ptr_signed(0)?`.
    pub fn byte_value_ptr_signed(&self, index: i32) -> Result<*const u8, Failure> {
        self.byte_value_ptr(checked_signed_index(index)?)
    }

    /// Return native `changeValue(int)` after rejecting negative indices; `array.change_byte_value_ptr_signed(0)?`.
    pub fn change_byte_value_ptr_signed(&mut self, index: i32) -> Result<*mut u8, Failure> {
        self.change_byte_value_ptr(checked_signed_index(index)?)
    }

    /// Copy a value into the supplied destination, native `Value(index, out)`; `array.value_into(0, &mut out)?`.
    pub fn value_into<T: ArrayValue>(&self, index: usize, output: &mut T) -> Result<(), Failure> {
        *output = *self.value(index)?;
        Ok(())
    }

    /// Copy a prefix into the supplied destination, native `Value2(index, out)`; `array.value2_into(0, &mut out)?`.
    pub fn value2_into<T: ArrayValue>(&self, index: usize, output: &mut T) -> Result<(), Failure> {
        *output = *self.value2(index)?;
        Ok(())
    }

    /// Copy using native signed-index exception ordering; `array.value_into_signed(0, &mut out)?`.
    pub fn value_into_signed<T: ArrayValue>(
        &self,
        index: i32,
        output: &mut T,
    ) -> Result<(), Failure> {
        self.value_into(checked_signed_index(index)?, output)
    }

    /// Copy a prefix using native signed-index exception ordering; `array.value2_into_signed(0, &mut out)?`.
    pub fn value2_into_signed<T: ArrayValue>(
        &self,
        index: i32,
        output: &mut T,
    ) -> Result<(), Failure> {
        self.value2_into(checked_signed_index(index)?, output)
    }
}

fn checked_signed_index(index: i32) -> Result<usize, Failure> {
    usize::try_from(index).map_err(|_| {
        Failure::out_of_range(&format!(
            "AliasedArray index {index}; expected a nonnegative signed index"
        ))
    })
}
