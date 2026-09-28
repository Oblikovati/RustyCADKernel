// Copyright (c) 2025 OPEN CASCADE SAS
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

//! Arbitrarily bounded arrays, corresponding to `NCollection_Array1`.

/// An owned array or a view borrowing an external buffer.
///
/// ```
/// use rustycad_tkernel::ncollection::Array1;
/// let array = Array1::<i32>::default();
/// assert_eq!((array.lower(), array.upper()), (1, 0));
/// ```
pub struct Array1<'a, T> {
    lower: i32,
    storage: Storage<'a, T>,
}

enum Storage<'a, T> {
    Owned(Vec<T>),
    Borrowed(&'a mut [T]),
    // A read-only view (the native `const_reference` buffer constructor) and the
    // clone routine that detaches it on first mutable access.
    Shared(&'a [T], fn(&[T]) -> Vec<T>),
}

impl<T: std::fmt::Debug> std::fmt::Debug for Array1<'_, T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Array1")
            .field("lower", &self.lower)
            .field("values", &self.as_slice())
            .finish()
    }
}

impl<T> Default for Array1<'_, T> {
    fn default() -> Self {
        Self {
            lower: 1,
            storage: Storage::Owned(Vec::new()),
        }
    }
}

impl<T> Array1<'_, T> {
    /// Returns the number of elements; `Array1::<i32>::default().len() == 0`.
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    /// Tests the empty range; `Array1::<i32>::default().is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the inclusive lower bound; the default array has `lower() == 1`.
    pub const fn lower(&self) -> i32 {
        self.lower
    }

    /// Returns the inclusive upper bound; the default array has `upper() == 0`.
    pub fn upper(&self) -> i32 {
        (i64::from(self.lower) + self.len() as i64 - 1) as i32
    }

    /// Borrows the contiguous values; `Array1::<i32>::default().as_slice().is_empty()`.
    pub fn as_slice(&self) -> &[T] {
        match &self.storage {
            Storage::Owned(values) => values,
            Storage::Borrowed(values) => values,
            Storage::Shared(values, _) => values,
        }
    }
}

impl<'a, T: Default> Array1<'a, T> {
    /// Allocates inclusive bounds; `Array1::<i32>::with_bounds(-2, 2).unwrap().len() == 5`.
    pub fn with_bounds(lower: i32, upper: i32) -> Result<Self, super::ArrayError> {
        let size = i64::from(upper) - i64::from(lower) + 1;
        if size < 0 {
            return Err(super::ArrayError::Bounds { lower, upper });
        }
        let mut array = Self::with_size(size as usize);
        array.lower = lower;
        Ok(array)
    }

    /// Allocates a zero-based array; `Array1::<i32>::with_size(3).upper() == 2`.
    pub fn with_size(size: usize) -> Self {
        Self {
            lower: 0,
            storage: Storage::Owned(std::iter::repeat_with(T::default).take(size).collect()),
        }
    }

    /// Selects borrowed storage or a fresh owned allocation; `Array1::from_buffer(&mut [1,2], true)`.
    pub fn from_buffer(buffer: &'a mut [T], use_buffer: bool) -> Self {
        if use_buffer {
            return Self::from_slice(buffer);
        }
        Self::with_size(buffer.len())
    }
}

impl<'a, T> Array1<'a, T> {
    /// Borrows existing elements with lower bound zero; `Array1::from_slice(&mut [1,2])`.
    pub fn from_slice(buffer: &'a mut [T]) -> Self {
        Self {
            lower: 0,
            storage: Storage::Borrowed(buffer),
        }
    }

    /// Views a read-only buffer with the given lower bound, as the native
    /// `NCollection_Array1(const_reference, lower, upper)` constructor does.
    ///
    /// The view does not own its storage (`is_deletable()` is false). Native code
    /// must not write through such a view; here the first mutable access copies
    /// the values into owned storage instead, so the shared buffer never changes;
    /// `Array1::from_shared(&[1.0, 1.0], 1).upper() == 2`.
    pub fn from_shared(buffer: &'a [T], lower: i32) -> Self
    where
        T: Clone,
    {
        Self {
            lower,
            storage: Storage::Shared(buffer, <[T]>::to_vec),
        }
    }

    /// Tests ownership of nonempty storage; `Array1::<i32>::with_size(1).is_deletable()`.
    pub fn is_deletable(&self) -> bool {
        matches!(self.storage, Storage::Owned(_)) && !self.is_empty()
    }

    /// Borrows the mutable values; `Array1::<i32>::with_size(3).as_mut_slice().fill(7)`.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        if let Storage::Shared(values, detach) = self.storage {
            self.storage = Storage::Owned(detach(values));
        }
        match &mut self.storage {
            Storage::Owned(values) => values,
            Storage::Borrowed(values) => values,
            // Detached above; an empty slice keeps this arm panic-free.
            Storage::Shared(..) => &mut [],
        }
    }

    /// Iterates in storage order; `Array1::from_slice(&mut [1,2]).iter().sum::<i32>() == 3`.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    /// Mutates in storage order; `Array1::from_slice(&mut [1,2]).iter_mut().for_each(|v| *v += 1)`.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
    }

    /// Reads a zero-based offset; `*Array1::from_slice(&mut [4]).at(0).unwrap() == 4`.
    pub fn at(&self, offset: usize) -> Result<&T, super::ArrayError> {
        self.as_slice().get(offset).ok_or(super::ArrayError::Index {
            index: offset as i64,
            lower: 0,
            upper: self.len() as i64 - 1,
        })
    }

    /// Mutates a zero-based offset; `*Array1::from_slice(&mut [4]).change_at(0).unwrap() = 5`.
    pub fn change_at(&mut self, offset: usize) -> Result<&mut T, super::ArrayError> {
        let upper = self.len() as i64 - 1;
        self.as_mut_slice()
            .get_mut(offset)
            .ok_or(super::ArrayError::Index {
                index: offset as i64,
                lower: 0,
                upper,
            })
    }

    /// Reads an index in the declared bounds; `Array1::<i32>::with_bounds(1,2).unwrap().value(1)`.
    pub fn value(&self, index: i32) -> Result<&T, super::ArrayError> {
        let offset = self.offset(index)?;
        self.at(offset)
    }

    /// Mutates an index in the declared bounds; `Array1::<i32>::with_bounds(1,2).unwrap().change_value(1)`.
    pub fn change_value(&mut self, index: i32) -> Result<&mut T, super::ArrayError> {
        let offset = self.offset(index)?;
        self.change_at(offset)
    }

    fn offset(&self, index: i32) -> Result<usize, super::ArrayError> {
        let offset = i64::from(index) - i64::from(self.lower);
        if offset >= 0 && offset < self.len() as i64 {
            return Ok(offset as usize);
        }
        Err(super::ArrayError::Index {
            index: index.into(),
            lower: self.lower.into(),
            upper: self.upper().into(),
        })
    }
}

impl<T: Clone> Clone for Array1<'_, T> {
    fn clone(&self) -> Self {
        Self {
            lower: self.lower,
            storage: Storage::Owned(self.as_slice().to_vec()),
        }
    }
}

impl<T: Clone> Array1<'_, T> {
    /// Copies values and bounds, reusing same-size storage; `dest.assign(&source)`.
    pub fn assign(&mut self, source: &Array1<'_, T>) {
        if self.len() == source.len() {
            self.as_mut_slice().clone_from_slice(source.as_slice());
        } else {
            self.storage = Storage::Owned(source.as_slice().to_vec());
        }
        self.lower = source.lower;
    }

    /// Copies equally sized contents without changing bounds; `dest.copy_values(&source)?`.
    pub fn copy_values(&mut self, source: &Array1<'_, T>) -> Result<(), super::ArrayError> {
        if self.len() != source.len() {
            return Err(super::ArrayError::DimensionMismatch {
                expected: self.len(),
                actual: source.len(),
            });
        }
        self.as_mut_slice().clone_from_slice(source.as_slice());
        Ok(())
    }
}

impl<T> Array1<'_, T> {
    /// Transfers storage and resets the source to the empty default; `dest.move_from(&mut source)`.
    pub fn move_from(&mut self, source: &mut Self) {
        *self = std::mem::take(source);
    }
}

impl<T> std::ops::Index<i32> for Array1<'_, T> {
    type Output = T;
    fn index(&self, index: i32) -> &Self::Output {
        self.value(index).unwrap_or_else(|error| panic!("{error}"))
    }
}

impl<T> std::ops::IndexMut<i32> for Array1<'_, T> {
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        self.change_value(index)
            .unwrap_or_else(|error| panic!("{error}"))
    }
}

impl<T> Array1<'_, T> {
    /// Replaces a bounded element; `array.set_value(array.lower(), value)?`.
    pub fn set_value(&mut self, index: i32, value: T) -> Result<(), super::ArrayError> {
        *self.change_value(index)? = value;
        Ok(())
    }

    /// Borrows the first element; `Array1::from_slice(&mut [7]).first().unwrap()`.
    pub fn first(&self) -> Result<&T, super::ArrayError> {
        self.at(0)
    }

    /// Borrows the last element; `Array1::from_slice(&mut [7]).last().unwrap()`.
    pub fn last(&self) -> Result<&T, super::ArrayError> {
        self.at(self.len().saturating_sub(1))
    }

    /// Mutates the first element; `*array.change_first()? = value`.
    pub fn change_first(&mut self) -> Result<&mut T, super::ArrayError> {
        self.change_at(0)
    }

    /// Mutates the last element; `*array.change_last()? = value`.
    pub fn change_last(&mut self) -> Result<&mut T, super::ArrayError> {
        self.change_at(self.len().saturating_sub(1))
    }
}

impl<T: Clone> Array1<'_, T> {
    /// Fills the existing range; `Array1::<i32>::with_size(3).init(42)`.
    pub fn init(&mut self, value: T) {
        self.as_mut_slice().fill(value);
    }
}

impl<T: Default + Clone> Array1<'_, T> {
    /// Resizes inclusive bounds, optionally preserving storage order; `array.resize_bounds(1, 5, true)?`.
    pub fn resize_bounds(
        &mut self,
        lower: i32,
        upper: i32,
        copy: bool,
    ) -> Result<(), super::ArrayError> {
        let length = i64::from(upper) - i64::from(lower) + 1;
        if length < 0 {
            return Err(super::ArrayError::Bounds { lower, upper });
        }
        self.resize(length as usize, copy);
        self.lower = lower;
        Ok(())
    }

    /// Resizes while keeping the lower bound; `array.resize(8, true)`.
    pub fn resize(&mut self, length: usize, copy: bool) {
        if length == self.len() {
            return;
        }
        let mut values = Vec::new();
        if copy {
            values.extend_from_slice(&self.as_slice()[..length.min(self.len())]);
        }
        values.resize_with(length, T::default);
        self.storage = Storage::Owned(values);
    }
}

impl<'b, T> IntoIterator for &'b Array1<'_, T> {
    type Item = &'b T;
    type IntoIter = std::slice::Iter<'b, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'b, T> IntoIterator for &'b mut Array1<'_, T> {
    type Item = &'b mut T;
    type IntoIter = std::slice::IterMut<'b, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> Array1<'_, T> {
    /// Constructs a replacement and returns its mutable slot; `array.emplace_value(0, || 42)?`.
    pub fn emplace_value(
        &mut self,
        index: i32,
        construct: impl FnOnce() -> T,
    ) -> Result<&mut T, super::ArrayError> {
        let slot = self.change_value(index)?;
        *slot = construct();
        Ok(slot)
    }
}

impl<T> Array1<'_, T> {
    /// Changes only the origin; `array.update_lower_bound(5)` keeps the storage address.
    pub fn update_lower_bound(&mut self, lower: i32) {
        self.lower = lower;
    }
}
