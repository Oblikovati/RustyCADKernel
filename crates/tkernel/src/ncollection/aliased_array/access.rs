// Copyright (c) 2021 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

use super::{AliasedArray, ArrayValue, cast_failure};
use crate::standard::Failure;
impl AliasedArray<'_> {
    /// Borrow contiguous typed elements after stride/alignment checks; `array.typed_data::<i32>()?`.
    pub fn typed_data<T: ArrayValue>(&self) -> Result<&[T], Failure> {
        self.check_type::<T>(false)?;
        if self.is_empty() {
            return Ok(&[]);
        }
        bytemuck::try_cast_slice(self.bytes()).map_err(cast_failure)
    }
    /// Mutably borrow contiguous typed elements; `array.change_data::<i32>()?[0] = 7`.
    pub fn change_data<T: ArrayValue>(&mut self) -> Result<&mut [T], Failure> {
        self.check_type::<T>(false)?;
        if self.is_empty() {
            return Ok(&mut []);
        }
        bytemuck::try_cast_slice_mut(self.bytes_mut()).map_err(cast_failure)
    }
    /// Fill typed elements without touching padding outside the logical array; `array.init(7_i32)?`.
    pub fn init<T: ArrayValue>(&mut self, value: T) -> Result<(), Failure> {
        self.change_data::<T>()?.fill(value);
        Ok(())
    }
    /// Borrow one stride-sized value; `array.value::<i32>(0)?`.
    pub fn value<T: ArrayValue>(&self, index: usize) -> Result<&T, Failure> {
        self.check_type::<T>(false)?;
        self.value2(index)
    }
    /// Mutably borrow one stride-sized value; `*array.change_value::<i32>(0)? = 5`.
    pub fn change_value<T: ArrayValue>(&mut self, index: usize) -> Result<&mut T, Failure> {
        self.check_type::<T>(false)?;
        self.change_value2(index)
    }
    /// Borrow a prefix type no larger than the stride; `array.value2::<u16>(0)?`.
    pub fn value2<T: ArrayValue>(&self, index: usize) -> Result<&T, Failure> {
        self.check_type::<T>(true)?;
        let offset: usize = self.offset(index)?;
        bytemuck::try_from_bytes(&self.bytes()[offset..offset + size_of::<T>()])
            .map_err(cast_failure)
    }
    /// Mutably borrow a prefix of one element; `*array.change_value2::<u16>(0)? = 1`.
    pub fn change_value2<T: ArrayValue>(&mut self, index: usize) -> Result<&mut T, Failure> {
        self.check_type::<T>(true)?;
        let offset: usize = self.offset(index)?;
        bytemuck::try_from_bytes_mut(&mut self.bytes_mut()[offset..offset + size_of::<T>()])
            .map_err(cast_failure)
    }
    /// Mutably borrow the final element; `*array.change_last::<i32>()? = 5`.
    pub fn change_last<T: ArrayValue>(&mut self) -> Result<&mut T, Failure> {
        self.change_value(self.count.wrapping_sub(1))
    }
    /// Borrow the final element; `array.last::<i32>()?`.
    pub fn last<T: ArrayValue>(&self) -> Result<&T, Failure> {
        self.value(self.count.wrapping_sub(1))
    }
    /// Borrow the first element; `array.first::<i32>()?`.
    pub fn first<T: ArrayValue>(&self) -> Result<&T, Failure> {
        self.value(0)
    }
    /// Mutably borrow the first element; `*array.change_first::<i32>()? = 5`.
    pub fn change_first<T: ArrayValue>(&mut self) -> Result<&mut T, Failure> {
        self.change_value(0)
    }
    /// The raw bytes of one element, as the native untyped `value()` pointer;
    /// `array.byte_value(0)?.len() == array.stride()`.
    pub fn byte_value(&self, index: usize) -> Result<&[u8], Failure> {
        let offset: usize = self.offset(index)?;
        Ok(&self.bytes()[offset..offset + self.stride])
    }
    /// Mutable raw bytes of one element, as the native `changeValue()`;
    /// `array.change_byte_value(0)?.fill(0)`.
    pub fn change_byte_value(&mut self, index: usize) -> Result<&mut [u8], Failure> {
        let offset: usize = self.offset(index)?;
        let stride: usize = self.stride;
        Ok(&mut self.bytes_mut()[offset..offset + stride])
    }
}
