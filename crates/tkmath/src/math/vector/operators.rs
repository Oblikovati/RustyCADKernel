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

use super::Vector;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};
impl Index<i32> for Vector<'_> {
    type Output = f64;
    fn index(&self, index: i32) -> &f64 {
        self.value(index).unwrap()
    }
}
impl IndexMut<i32> for Vector<'_> {
    fn index_mut(&mut self, index: i32) -> &mut f64 {
        self.change_value(index).unwrap()
    }
}
impl<'a> Add<&Vector<'a>> for &Vector<'a> {
    type Output = Vector<'a>;
    fn add(self, right: &Vector<'a>) -> Self::Output {
        self.added(right).unwrap()
    }
}
impl<'a> Sub<&Vector<'a>> for &Vector<'a> {
    type Output = Vector<'a>;
    fn sub(self, right: &Vector<'a>) -> Self::Output {
        self.subtracted(right).unwrap()
    }
}
impl<'a> Mul<f64> for &Vector<'a> {
    type Output = Vector<'a>;
    fn mul(self, right: f64) -> Self::Output {
        self.multiplied(right)
    }
}
impl<'a> Mul<&Vector<'a>> for f64 {
    type Output = Vector<'a>;
    fn mul(self, right: &Vector<'a>) -> Self::Output {
        right.multiplied(self)
    }
}
impl Mul<&Vector<'_>> for &Vector<'_> {
    type Output = f64;
    fn mul(self, right: &Vector<'_>) -> f64 {
        self.dot(right).unwrap()
    }
}
impl<'a> Div<f64> for &Vector<'a> {
    type Output = Vector<'a>;
    fn div(self, right: f64) -> Self::Output {
        self.divided(right).unwrap()
    }
}
impl<'a> Neg for &Vector<'a> {
    type Output = Vector<'a>;
    fn neg(self) -> Self::Output {
        self.opposite()
    }
}
impl AddAssign<&Self> for Vector<'_> {
    fn add_assign(&mut self, right: &Self) {
        self.add(right).unwrap();
    }
}
impl SubAssign<&Self> for Vector<'_> {
    fn sub_assign(&mut self, right: &Self) {
        self.subtract(right).unwrap();
    }
}
impl MulAssign<f64> for Vector<'_> {
    fn mul_assign(&mut self, right: f64) {
        self.multiply(right);
    }
}
impl DivAssign<f64> for Vector<'_> {
    fn div_assign(&mut self, right: f64) {
        self.divide(right).unwrap();
    }
}
