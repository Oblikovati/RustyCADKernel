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

use super::{Failure, VectorBase, VectorItem, failure};
impl<T: VectorItem> VectorBase<'_, T> {
    /// Returns the four-accumulator native squared norm; `vector.norm2()`.
    pub fn norm2(&self) -> Result<f64, Failure> {
        self.value(self.lower())?;
        let mut sums = [0.0; 4];
        let (chunks, tail) = self.values().as_chunks::<4>();
        // Each element is widened to double before squaring, as the template does.
        for chunk in chunks {
            for lane in 0..4 {
                let value = chunk[lane].to_real();
                sums[lane] += value * value;
            }
        }
        for value in tail {
            let value = value.to_real();
            sums[0] += value * value;
        }
        Ok((sums[0] + sums[1]) + (sums[2] + sums[3]))
    }
    /// Returns the Euclidean norm in native arithmetic order; `vector.norm()`.
    pub fn norm(&self) -> Result<f64, Failure> {
        Ok(self.norm2()?.sqrt())
    }
    /// Returns the first strictly greatest element index; `vector.max_index()`.
    pub fn max_index(&self) -> i32 {
        let mut index = 0;
        let mut greatest = -f64::MAX;
        for i in self.lower()..=self.upper() {
            if self[i].to_real() > greatest {
                greatest = self[i].to_real();
                index = i;
            }
        }
        index
    }
    /// Returns the first strictly least element index; `vector.min_index()`.
    pub fn min_index(&self) -> i32 {
        let mut index = 0;
        let mut least = f64::MAX;
        for i in self.lower()..=self.upper() {
            if self[i].to_real() < least {
                least = self[i].to_real();
                index = i;
            }
        }
        index
    }
    /// Divides by the norm after the native epsilon check; `vector.normalize()?`.
    pub fn normalize(&mut self) -> Result<(), Failure> {
        let norm = self.norm()?;
        if norm <= f64::EPSILON {
            return Err(failure(
                "Standard_NullValue",
                "math_VectorBase::Normalize() - vector has zero norm",
            ));
        }
        // `Array(i) / Result` is a double quotient converted back to the item type.
        for value in self.values_mut() {
            *value = T::from_real(value.to_real() / norm);
        }
        Ok(())
    }
    /// Copies then normalizes without changing source; `vector.normalized()?`.
    pub fn normalized(&self) -> Result<Self, Failure> {
        let mut result = self.clone();
        result.normalize()?;
        Ok(result)
    }
    /// Preserves the native bound-dependent swap limit; `vector.invert()?`.
    pub fn invert(&mut self) -> Result<(), Failure> {
        for index in self.lower()..=((self.lower() + self.length()) >> 1) {
            let opposite = self.upper() + self.lower() - index;
            let first = self.index_offset(index)?;
            let last = self.index_offset(opposite)?;
            self.values_mut().swap(first, last);
        }
        Ok(())
    }
    /// Copies and applies native reversal; `vector.inverse()?`.
    pub fn inverse(&self) -> Result<Self, Failure> {
        let mut result = self.clone();
        result.invert()?;
        Ok(result)
    }
    /// Multiplies each cell in place; `vector.multiply(2.0)`.
    pub fn multiply(&mut self, scalar: T) {
        for value in self.values_mut() {
            *value = value.item_mul(scalar);
        }
    }
    /// Produces native element-times-scalar products; `vector.multiplied(2.0)`.
    pub fn multiplied(&self, scalar: T) -> Self {
        let mut result = Self::with_bounds(self.lower(), self.upper()).unwrap();
        for (target, source) in result.values_mut().iter_mut().zip(self.values()) {
            *target = source.item_mul(scalar);
        }
        result
    }
    /// Native TMultiplied, identical to `multiplied` for a vector; `vector.t_multiplied(2.0)`.
    pub fn t_multiplied(&self, scalar: T) -> Self {
        self.multiplied(scalar)
    }
    /// Divides each cell directly after checking epsilon; `vector.divide(2.0)?`.
    pub fn divide(&mut self, scalar: T) -> Result<(), Failure> {
        checked_divisor(scalar.to_real(), "Divide")?;
        for value in self.values_mut() {
            *value = value.item_div(scalar);
        }
        Ok(())
    }
    /// Multiplies by the reciprocal, matching native Divided; `vector.divided(2.0)?`.
    ///
    /// The reciprocal `1. / theRight` is converted to the item type before the
    /// product, so an integer vector divided by anything but +-1 becomes zero.
    pub fn divided(&self, scalar: T) -> Result<Self, Failure> {
        checked_divisor(scalar.to_real(), "Divided")?;
        Ok(self.multiplied(T::from_real(1.0 / scalar.to_real())))
    }
    /// Returns the sign-reversed vector; `vector.opposite()`.
    pub fn opposite(&self) -> Self {
        let mut result = Self::with_bounds(self.lower(), self.upper()).unwrap();
        for (target, source) in result.values_mut().iter_mut().zip(self.values()) {
            *target = source.item_neg();
        }
        result
    }
    /// Computes a sequential dot product with positional alignment; `left.dot(&right)?`.
    pub fn dot(&self, right: &VectorBase<'_, T>) -> Result<T, Failure> {
        self.check_length(right, "Multiplied")?;
        let mut result = T::ZERO;
        for (left, right) in self.values().iter().zip(right.values()) {
            result = result.item_add(left.item_mul(*right));
        }
        Ok(result)
    }
    /// Adds equal-length vectors by position; `left.add(&right)?`.
    pub fn add(&mut self, right: &VectorBase<'_, T>) -> Result<(), Failure> {
        self.check_length(right, "Add")?;
        for (left, right) in self.values_mut().iter_mut().zip(right.values()) {
            *left = left.item_add(*right);
        }
        Ok(())
    }
    /// Subtracts equal-length vectors by position; `left.subtract(&right)?`.
    pub fn subtract(&mut self, right: &VectorBase<'_, T>) -> Result<(), Failure> {
        self.check_length(right, "Subtract")?;
        for (left, right) in self.values_mut().iter_mut().zip(right.values()) {
            *left = left.item_sub(*right);
        }
        Ok(())
    }
    /// Returns the positional sum retaining left bounds; `left.added(&right)?`.
    pub fn added(&self, right: &VectorBase<'_, T>) -> Result<Self, Failure> {
        self.check_length(right, "Added")?;
        let mut result = Self::with_bounds(self.lower(), self.upper()).unwrap();
        result.initialized(self)?;
        result.add(right)?;
        Ok(result)
    }
    /// Returns the positional difference retaining left bounds; `left.subtracted(&right)?`.
    pub fn subtracted(&self, right: &VectorBase<'_, T>) -> Result<Self, Failure> {
        self.check_length(right, "Subtracted")?;
        let mut result = Self::with_bounds(self.lower(), self.upper()).unwrap();
        result.initialized(self)?;
        result.subtract(right)?;
        Ok(result)
    }
    /// Copies a source into a checked inclusive interval; `target.set(2,4,&source)?`.
    pub fn set(
        &mut self,
        first: i32,
        last: i32,
        source: &VectorBase<'_, T>,
    ) -> Result<(), Failure> {
        if first < self.lower()
            || last > self.upper()
            || first > last
            || last - first + 1 != source.length()
        {
            return Err(failure(
                "Standard_RangeError",
                "math_VectorBase::Set() - invalid indices",
            ));
        }
        let offset = self.index_offset(first)?;
        self.values_mut()[offset..offset + source.length].copy_from_slice(source.values());
        Ok(())
    }
    /// Copies either endpoint order into ascending bounds; `vector.slice(4,2)?`.
    pub fn slice(&self, first: i32, last: i32) -> Result<Self, Failure> {
        if first < self.lower()
            || first > self.upper()
            || last < self.lower()
            || last > self.upper()
        {
            return Err(failure(
                "Standard_RangeError",
                "math_VectorBase::Slice() - invalid indices",
            ));
        }
        let mut result = Self::with_bounds(first.min(last), first.max(last)).unwrap();
        for index in result.lower()..=result.upper() {
            result[index] = self[index];
        }
        Ok(result)
    }
    /// Stores the positional sum of two vectors; `result.set_sum(&left,&right)?`.
    pub fn set_sum(
        &mut self,
        left: &VectorBase<'_, T>,
        right: &VectorBase<'_, T>,
    ) -> Result<(), Failure> {
        self.check_pair(left, right, "Add")?;
        for index in 0..self.length {
            self.values_mut()[index] = left.values()[index].item_add(right.values()[index]);
        }
        Ok(())
    }
    /// Stores the positional difference of two vectors; `result.set_difference(&left,&right)?`.
    pub fn set_difference(
        &mut self,
        left: &VectorBase<'_, T>,
        right: &VectorBase<'_, T>,
    ) -> Result<(), Failure> {
        self.check_pair(left, right, "Subtract")?;
        for index in 0..self.length {
            self.values_mut()[index] = left.values()[index].item_sub(right.values()[index]);
        }
        Ok(())
    }
    /// Uses matching declared indices, retaining the native overload's bound behavior; `result.set_scaled(2.0,&source)?`.
    pub fn set_scaled(&mut self, scalar: T, source: &VectorBase<'_, T>) -> Result<(), Failure> {
        self.check_length(source, "Multiply")?;
        for index in self.lower()..=self.upper() {
            self[index] = scalar.item_mul(*source.value(index)?);
        }
        Ok(())
    }
    fn check_length(&self, right: &VectorBase<'_, T>, operation: &str) -> Result<(), Failure> {
        if self.length != right.length {
            return Err(failure(
                "Standard_DimensionError",
                &format!("math_VectorBase::{operation}() - input vector has wrong dimensions"),
            ));
        }
        Ok(())
    }
    fn check_pair(
        &self,
        left: &VectorBase<'_, T>,
        right: &VectorBase<'_, T>,
        operation: &str,
    ) -> Result<(), Failure> {
        if self.length != right.length || self.length != left.length {
            return Err(failure(
                "Standard_DimensionError",
                &format!("math_VectorBase::{operation}() - input vectors have wrong dimensions"),
            ));
        }
        Ok(())
    }
}
fn checked_divisor(scalar: f64, operation: &str) -> Result<(), Failure> {
    if scalar.abs() <= f64::EPSILON {
        return Err(failure(
            "Standard_DivideByZero",
            &format!("math_VectorBase::{operation}() - devisor is zero"),
        ));
    }
    Ok(())
}
