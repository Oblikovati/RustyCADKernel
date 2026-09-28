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

//! Runtime-stride arrays with checked byte views and exclusive external borrowing.
use crate::standard::Failure;
mod access;
/// Plain bitwise value accepted by runtime-stride views; all integer and float primitives qualify.
pub trait ArrayValue: bytemuck::Pod {}
impl<T: bytemuck::Pod> ArrayValue for T {}
enum ArrayStorage<'a> {
    Empty,
    Owned(Vec<u128>),
    Borrowed(&'a mut [u8]),
}
/// Default-alignment OCCT aliased storage; `AliasedArray::new(size_of::<i32>(), 3)`.
pub struct AliasedArray<'a> {
    storage: ArrayStorage<'a>,
    stride: usize,
    count: usize,
}
impl<'a> AliasedArray<'a> {
    /// Create zero-length non-owning state; `AliasedArray::empty(4)?`.
    pub fn empty(stride: usize) -> Result<Self, Failure> {
        checked_stride(stride)?;
        Ok(Self {
            storage: ArrayStorage::Empty,
            stride,
            count: 0,
        })
    }
    /// Allocate default 16-byte-aligned storage; `AliasedArray::new(4, 3)?`.
    /// Fresh bytes are initialized to zero where native bytes are indeterminate.
    pub fn new(stride: usize, count: usize) -> Result<Self, Failure> {
        let mut array: Self = Self::empty(stride)?;
        array.resize(count, false)?;
        Ok(array)
    }
    /// Borrow mutable external values without copying; `AliasedArray::from_slice(&mut values)?`.
    pub fn from_slice<T: ArrayValue>(values: &'a mut [T]) -> Result<Self, Failure> {
        checked_stride(size_of::<T>())?;
        checked_count(values.len())?;
        let count: usize = values.len();
        Ok(Self {
            storage: ArrayStorage::Borrowed(bytemuck::cast_slice_mut(values)),
            stride: size_of::<T>(),
            count,
        })
    }
    /// Return element size in bytes; `array.stride()`.
    pub fn stride(&self) -> usize {
        self.stride
    }
    /// Return element count; `array.len()`.
    pub fn len(&self) -> usize {
        self.count
    }
    /// Return OCCT's signed length; `array.length()`.
    pub fn length(&self) -> i32 {
        self.count as i32
    }
    /// Check zero length; `array.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    /// Report storage ownership; `array.is_deletable()`.
    pub fn is_deletable(&self) -> bool {
        matches!(self.storage, ArrayStorage::Owned(_))
    }
    /// Return logical byte length, excluding alignment padding; `array.size_bytes()`.
    pub fn size_bytes(&self) -> usize {
        self.stride * self.count
    }
    /// Reallocate unless the element count is unchanged; `array.resize(4, true)?`.
    pub fn resize(&mut self, count: usize, copy: bool) -> Result<(), Failure> {
        checked_count(count)?;
        if count == self.count {
            return Ok(());
        }
        let bytes: usize = self
            .stride
            .checked_mul(count)
            .ok_or_else(|| invalid_shape(self.stride, count))?;
        let mut storage: Vec<u128> = vec![0; bytes.div_ceil(size_of::<u128>())];
        if copy {
            let copied: usize = self.size_bytes().min(bytes);
            bytemuck::cast_slice_mut::<u128, u8>(&mut storage)[..copied]
                .copy_from_slice(&self.bytes()[..copied]);
        }
        self.storage = ArrayStorage::Owned(storage);
        self.count = count;
        Ok(())
    }
    /// Copy equal-shaped contents into existing storage; `array.assign(&source)?`.
    pub fn assign(&mut self, source: &Self) -> Result<(), Failure> {
        if self.stride != source.stride || self.count != source.count {
            return Err(Failure::with_type(
                "Standard_DimensionMismatch",
                &format!(
                    "AliasedArray shape ({}, {}); expected ({}, {})",
                    source.stride, source.count, self.stride, self.count
                ),
                "",
            ));
        }
        self.bytes_mut().copy_from_slice(source.bytes());
        Ok(())
    }
    fn bytes(&self) -> &[u8] {
        let bytes: &[u8] = match &self.storage {
            ArrayStorage::Empty => &[],
            ArrayStorage::Owned(words) => bytemuck::cast_slice(words),
            ArrayStorage::Borrowed(bytes) => bytes,
        };
        &bytes[..self.size_bytes()]
    }
    fn bytes_mut(&mut self) -> &mut [u8] {
        let size: usize = self.size_bytes();
        let bytes: &mut [u8] = match &mut self.storage {
            ArrayStorage::Empty => &mut [],
            ArrayStorage::Owned(words) => bytemuck::cast_slice_mut(words),
            ArrayStorage::Borrowed(bytes) => bytes,
        };
        &mut bytes[..size]
    }
    fn check_type<T: ArrayValue>(&self, partial: bool) -> Result<(), Failure> {
        if size_of::<T>() == 0
            || (if partial {
                size_of::<T>() > self.stride
            } else {
                size_of::<T>() != self.stride
            })
        {
            return Err(Failure::with_type(
                "Standard_TypeMismatch",
                &format!(
                    "AliasedArray type size {}; expected {} stride {}",
                    size_of::<T>(),
                    if partial { "at most" } else { "exactly" },
                    self.stride
                ),
                "",
            ));
        }
        Ok(())
    }
    fn offset(&self, index: usize) -> Result<usize, Failure> {
        if index >= self.count {
            return Err(Failure::out_of_range(&format!(
                "AliasedArray index {index}; expected 0..{}",
                self.count
            )));
        }
        Ok(index * self.stride)
    }
}
impl Clone for AliasedArray<'_> {
    fn clone(&self) -> Self {
        if self.is_empty() {
            return Self::empty(self.stride).unwrap();
        }
        let mut copy: Self = Self::new(self.stride, self.count).unwrap();
        copy.bytes_mut().copy_from_slice(self.bytes());
        copy
    }
}
fn checked_stride(stride: usize) -> Result<(), Failure> {
    if stride == 0 || stride > u32::MAX as usize {
        return Err(invalid_shape(stride, 0));
    }
    Ok(())
}
fn checked_count(count: usize) -> Result<(), Failure> {
    if count == 0 {
        return Err(invalid_shape(0, count));
    }
    Ok(())
}
fn invalid_shape(stride: usize, count: usize) -> Failure {
    Failure::with_type(
        "Standard_RangeError",
        &format!(
            "AliasedArray stride {stride}, length {count}; expected positive supported dimensions without byte-size overflow"
        ),
        "",
    )
}
fn cast_failure(error: bytemuck::PodCastError) -> Failure {
    Failure::with_type(
        "Standard_TypeMismatch",
        &format!(
            "AliasedArray cast {error:?}; expected aligned storage matching the requested type"
        ),
        "",
    )
}
