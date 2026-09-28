// Copyright (c) 1997-1999 Matra Datavision
// Copyright (c) 1999-2023 OPEN CASCADE SAS
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

//! Bounded numeric vectors, translated from math_VectorBase at pinned OCCT 3d097a0.
use crate::gp::{Xy, Xyz};
use rustycad_tkernel::standard::Failure;
mod algebra;
mod dump;
mod item;
mod matrix;
mod operators;
mod storage;
pub use item::VectorItem;
pub(crate) use item::stream_real;
use storage::VectorStorage;
/// Numeric vector with arbitrary bounds and a 32-value inline buffer, the port of
/// `math_VectorBase<TheItemType>`; `VectorBase::<i32>::with_bounds(1,3)?`.
#[derive(Debug)]
pub struct VectorBase<'a, T: VectorItem> {
    lower: i32,
    length: usize,
    inline: [T; 32],
    storage: VectorStorage<'a, T>,
}
/// `math_Vector`, the `double` instantiation; `Vector::with_bounds(1,3)?`.
pub type Vector<'a> = VectorBase<'a, f64>;
/// `math_IntegerVector`, the `int` instantiation; `IntegerVector::with_bounds(1,3)?`.
pub type IntegerVector<'a> = VectorBase<'a, i32>;
impl<'a, T: VectorItem> VectorBase<'a, T> {
    /// Allocates zero-based elements; `Vector::with_size(3)`.
    pub fn with_size(length: usize) -> Self {
        Self {
            lower: 0,
            length,
            inline: [T::ZERO; 32],
            storage: if length <= 32 {
                VectorStorage::Inline
            } else {
                VectorStorage::Owned(vec![T::ZERO; length])
            },
        }
    }
    /// Allocates inclusive bounds, allowing an empty interval; `Vector::with_bounds(-1,2)?`.
    pub fn with_bounds(lower: i32, upper: i32) -> Result<Self, Failure> {
        let length = i64::from(upper) - i64::from(lower) + 1;
        if length < 0 {
            return Err(failure("Standard_RangeError", "NCollection_Array1::Create"));
        }
        let mut result = Self::with_size(length as usize);
        result.lower = lower;
        Ok(result)
    }
    /// Allocates zero-based elements set to one value; `Vector::with_size_filled(3,2.0)`.
    pub fn with_size_filled(length: usize, value: T) -> Self {
        let mut result = Self::with_size(length);
        result.init(value);
        result
    }
    /// Allocates and initializes a bounded vector; `Vector::filled(1,3,2.0)?`.
    pub fn filled(lower: i32, upper: i32, value: T) -> Result<Self, Failure> {
        let mut result = Self::with_bounds(lower, upper)?;
        result.init(value);
        Ok(result)
    }
    /// Borrows external elements; `Vector::from_buffer(&mut values,1,3)?`.
    pub fn from_buffer(values: &'a mut [T], lower: i32, upper: i32) -> Result<Self, Failure> {
        let mut result = Self::with_bounds(lower, upper)?;
        if values.len() < result.length {
            return Err(failure(
                "Standard_DimensionMismatch",
                "vector buffer is shorter than its bounds",
            ));
        }
        result.storage = VectorStorage::Borrowed(&mut values[..result.length]);
        Ok(result)
    }
    /// Copies two geometric coordinates into bounds 1..2; `Vector::from_xy(Xy::new(1.0,2.0))`.
    pub fn from_xy(point: Xy) -> Self {
        let mut result = Self::with_size(2);
        result.lower = 1;
        result.inline[..2].copy_from_slice(&[T::from_real(point.x()), T::from_real(point.y())]);
        result
    }
    /// Copies three geometric coordinates into bounds 1..3; `Vector::from_xyz(Xyz::default())`.
    pub fn from_xyz(point: Xyz) -> Self {
        let mut result = Self::with_size(3);
        result.lower = 1;
        result.inline[..3].copy_from_slice(&[
            T::from_real(point.x()),
            T::from_real(point.y()),
            T::from_real(point.z()),
        ]);
        result
    }
    /// Returns the native signed length; `vector.length()`.
    pub fn length(&self) -> i32 {
        self.length as i32
    }
    /// Returns the element count as `size_t`; `vector.size()`.
    pub fn size(&self) -> usize {
        self.length
    }
    /// Returns the first declared index; `vector.lower()`.
    pub fn lower(&self) -> i32 {
        self.lower
    }
    /// Returns the final declared index; `vector.upper()`.
    pub fn upper(&self) -> i32 {
        (i64::from(self.lower) + self.length as i64 - 1) as i32
    }
    /// Reads a zero-based offset; `vector.at(0)?`.
    pub fn at(&self, index: usize) -> Result<&T, Failure> {
        self.values()
            .get(index)
            .ok_or_else(|| Failure::out_of_range("NCollection_Array1::At"))
    }
    /// Mutates a zero-based offset; `*vector.change_at(0)?=1.0`.
    pub fn change_at(&mut self, index: usize) -> Result<&mut T, Failure> {
        self.values_mut()
            .get_mut(index)
            .ok_or_else(|| Failure::out_of_range("NCollection_Array1::ChangeAt"))
    }
    /// Reads a declared index; `vector.value(vector.lower())?`.
    pub fn value(&self, index: i32) -> Result<&T, Failure> {
        self.at(self.index_offset(index)?)
    }
    /// Mutates a declared index; `*vector.change_value(1)?=2.0`.
    pub fn change_value(&mut self, index: i32) -> Result<&mut T, Failure> {
        let offset = self.index_offset(index)?;
        self.change_at(offset)
    }
    /// Initializes all elements; `vector.init(1.0)`.
    pub fn init(&mut self, value: T) {
        self.values_mut().fill(value);
    }
    /// Copies matching lengths without changing bounds; `target.initialized(&source)?`.
    pub fn initialized(&mut self, source: &VectorBase<'_, T>) -> Result<(), Failure> {
        if self.length != source.length {
            return Err(failure(
                "Standard_DimensionMismatch",
                "NCollection_Array1::CopyValues",
            ));
        }
        self.values_mut().copy_from_slice(source.values());
        Ok(())
    }
    fn index_offset(&self, index: i32) -> Result<usize, Failure> {
        if index < self.lower() || index > self.upper() {
            return Err(Failure::out_of_range("NCollection_Array1::Value"));
        }
        Ok((i64::from(index) - i64::from(self.lower)) as usize)
    }
}
fn failure(kind: &'static str, message: &str) -> Failure {
    Failure::with_type(kind, message, "")
}
