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

use super::{VectorBase, VectorItem};
use std::fmt;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};
impl<T: VectorItem> Index<i32> for VectorBase<'_, T> {
    type Output = T;
    fn index(&self, index: i32) -> &T {
        self.value(index).unwrap()
    }
}
impl<T: VectorItem> IndexMut<i32> for VectorBase<'_, T> {
    fn index_mut(&mut self, index: i32) -> &mut T {
        self.change_value(index).unwrap()
    }
}
impl<'a, T: VectorItem> Add<&VectorBase<'a, T>> for &VectorBase<'a, T> {
    type Output = VectorBase<'a, T>;
    fn add(self, right: &VectorBase<'a, T>) -> Self::Output {
        self.added(right).unwrap()
    }
}
impl<'a, T: VectorItem> Sub<&VectorBase<'a, T>> for &VectorBase<'a, T> {
    type Output = VectorBase<'a, T>;
    fn sub(self, right: &VectorBase<'a, T>) -> Self::Output {
        self.subtracted(right).unwrap()
    }
}
impl<'a, T: VectorItem> Mul<T> for &VectorBase<'a, T> {
    type Output = VectorBase<'a, T>;
    fn mul(self, right: T) -> Self::Output {
        self.multiplied(right)
    }
}
impl<T: VectorItem> Mul<&VectorBase<'_, T>> for &VectorBase<'_, T> {
    type Output = T;
    fn mul(self, right: &VectorBase<'_, T>) -> T {
        self.dot(right).unwrap()
    }
}
impl<'a, T: VectorItem> Div<T> for &VectorBase<'a, T> {
    type Output = VectorBase<'a, T>;
    fn div(self, right: T) -> Self::Output {
        self.divided(right).unwrap()
    }
}
impl<'a, T: VectorItem> Neg for &VectorBase<'a, T> {
    type Output = VectorBase<'a, T>;
    fn neg(self) -> Self::Output {
        self.opposite()
    }
}
impl<T: VectorItem> AddAssign<&Self> for VectorBase<'_, T> {
    fn add_assign(&mut self, right: &Self) {
        self.add(right).unwrap();
    }
}
impl<T: VectorItem> SubAssign<&Self> for VectorBase<'_, T> {
    fn sub_assign(&mut self, right: &Self) {
        self.subtract(right).unwrap();
    }
}
impl<T: VectorItem> MulAssign<T> for VectorBase<'_, T> {
    fn mul_assign(&mut self, right: T) {
        self.multiply(right);
    }
}
impl<T: VectorItem> DivAssign<T> for VectorBase<'_, T> {
    fn div_assign(&mut self, right: T) {
        self.divide(right).unwrap();
    }
}
/// The friend `operator<<`, which forwards to `Dump`; `format!("{vector}")`.
impl<T: VectorItem> fmt::Display for VectorBase<'_, T> {
    fn fmt(&self, stream: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.dump(stream)
    }
}
// The friend `operator*(TheItemType, const math_VectorBase&)`; coherence rules need one
// impl per concrete item type.
macro_rules! scalar_times_vector {
    ($item:ty) => {
        impl<'a> Mul<&VectorBase<'a, $item>> for $item {
            type Output = VectorBase<'a, $item>;
            fn mul(self, right: &VectorBase<'a, $item>) -> Self::Output {
                right.multiplied(self)
            }
        }
    };
}
scalar_times_vector!(f64);
scalar_times_vector!(i32);
