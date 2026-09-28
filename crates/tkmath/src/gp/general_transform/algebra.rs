// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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

//! gp_GTrsf inversion, composition, powers and the templated Mat4 conversions.
use super::super::{Failure, Matrix3, Transform3, TransformForm, Xyz};
use super::GeneralTransform3;
use rustycad_tkernel::ncollection::{ElementCast, Mat4};
use std::ops;
impl GeneralTransform3 {
    fn is_general(&self) -> bool {
        self.form == TransformForm::Other
    }
    /// Returns the inverse; `transform.inverted()?`.
    pub fn inverted(mut self) -> Result<Self, Failure> {
        self.invert()?;
        Ok(self)
    }
    /// Inverts the matrix of a general form (no zero check in a release build), otherwise
    /// inverts the equivalent gp_Trsf; `transform.invert()?`.
    pub fn invert(&mut self) -> Result<(), Failure> {
        if self.is_general() {
            self.matrix.invert()?;
            self.offset = self.matrix.apply(self.offset).reversed();
            return Ok(());
        }
        let mut similarity: Transform3 = self.trsf()?;
        similarity.invert()?;
        self.set_trsf(similarity);
        Ok(())
    }
    /// Returns `self * other`; `left.multiplied(right)`.
    pub fn multiplied(mut self, other: Self) -> Self {
        self.multiply(other);
        self
    }
    /// Right-composes; a general operand keeps this scale and forces form Other. Similarity
    /// pairs compose as gp_Trsf, whose conversion only raises for form Other, so this cannot
    /// fail; `left.multiply(right)`.
    pub fn multiply(&mut self, other: Self) {
        if self.is_general() || other.is_general() {
            self.form = TransformForm::Other;
            self.offset += self.matrix.apply(other.offset);
            self.matrix *= other.matrix;
            return;
        }
        let mut similarity: Transform3 = self.as_trsf();
        similarity.multiply(other.as_trsf());
        self.set_trsf(similarity);
    }
    /// Left-composes, `self = other * self`; `right.pre_multiply(left)`.
    pub fn pre_multiply(&mut self, other: Self) {
        if self.is_general() || other.is_general() {
            self.form = TransformForm::Other;
            self.offset = other.matrix.apply(self.offset) + other.offset;
            self.matrix.pre_multiply(other.matrix);
            return;
        }
        let mut similarity: Transform3 = self.as_trsf();
        similarity.pre_multiply(other.as_trsf());
        self.set_trsf(similarity);
    }
    /// Returns the integer power; see `power`; `transform.powered(3)?`.
    pub fn powered(mut self, exponent: i32) -> Result<Self, Failure> {
        self.power(exponent)?;
        Ok(self)
    }
    /// gp_GTrsf::Power. Unlike gp_Trsf and gp_GTrsf2d, an exponent below -1 does not invert
    /// first: a general form returns the power of `|n|`, and a similarity form passes `n` to
    /// gp_Trsf::Power, which inverts; `transform.power(3)?`.
    pub fn power(&mut self, exponent: i32) -> Result<(), Failure> {
        match exponent {
            0 => *self = Self::default(),
            1 => {}
            -1 => self.invert()?,
            _ if self.is_general() => self.power_general(exponent.wrapping_abs().wrapping_sub(1)),
            _ => {
                let mut similarity = self.trsf()?;
                similarity.power(exponent)?;
                self.set_trsf(similarity);
            }
        }
        Ok(())
    }
    fn power_general(&mut self, mut remaining: i32) {
        let (mut factor_offset, mut factor): (Xyz, Matrix3) = (self.offset, self.matrix);
        loop {
            if remaining % 2 == 1 {
                self.offset += self.matrix.apply(factor_offset);
                self.matrix *= factor;
            }
            if remaining == 1 {
                return;
            }
            factor_offset += factor.apply(factor_offset);
            factor *= factor;
            remaining /= 2;
        }
    }
    /// Writes the homogeneous 4x4 matrix with `static_cast` elements; an identity form writes
    /// the identity; `transform.get_mat4(&mut matrix)`.
    pub fn get_mat4<T: Copy + From<u8>>(&self, output: &mut Mat4<T>)
    where
        f64: ElementCast<T>,
    {
        if self.form == TransformForm::Identity {
            output.init_identity();
            return;
        }
        for row in 0..3 {
            for column in 0..4 {
                output.set_value(row, column, self.coefficient(row + 1, column + 1).cast());
            }
        }
        for column in 0..3 {
            output.set_value(3, column, T::from(0));
        }
        output.set_value(3, 3, T::from(1));
    }
    /// Reads the upper 3x4 block as a general transformation, as `SetMat4`; the bottom row
    /// is ignored; `transform.set_mat4(&matrix)`.
    pub fn set_mat4<T: Copy + Into<f64>>(&mut self, input: &Mat4<T>) {
        self.set_general();
        let value = |row: usize, column: usize| -> f64 { input.get_value(row, column).into() };
        self.matrix = Matrix3::new(std::array::from_fn(|r| {
            std::array::from_fn(|c| value(r, c))
        }));
        self.offset = Xyz::new(value(0, 3), value(1, 3), value(2, 3));
    }
}
impl ops::Mul for GeneralTransform3 {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        self.multiplied(other)
    }
}
impl ops::MulAssign for GeneralTransform3 {
    fn mul_assign(&mut self, other: Self) {
        self.multiply(other);
    }
}
