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
// Rust translation of FoundationClasses/TKMath/gp/gp_Trsf.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::{Failure, Transform3, TransformForm, Xyz};
impl Transform3 {
    /// Composes using native form-specific scale and offset paths; `left.multiplied(right)`.
    pub fn multiplied(mut self, other: Self) -> Self {
        use TransformForm::*;
        match (self.form, other.form) {
            (_, Identity) => return self,
            (Identity, _) => return other,
            (Rotation, Rotation) => self.multiply_rotations(other),
            (Translation, Translation) => self.offset += other.offset,
            (Scale, Scale) => {
                self.offset += other.offset * self.scale;
                self.scale *= other.scale;
            }
            (_, Translation) => self.offset += self.scaled_offset(other.offset),
            (Translation, _) => self.translation_times(other),
            (Scale, _) => self.scale_times(other),
            (_, Scale) => self.times_scale(other),
            _ => self.multiply_general(other),
        }
        self
    }
    fn multiply_rotations(&mut self, other: Self) {
        if other.offset != Xyz::default() {
            self.offset += self.matrix.apply(other.offset);
        }
        self.matrix *= other.matrix;
    }
    fn scaled_offset(&self, offset: Xyz) -> Xyz {
        if self.form == TransformForm::Scale {
            return offset * self.scale;
        }
        let mut result = self.matrix.apply(offset);
        if self.scale != 1.0 {
            result *= self.scale;
        }
        result
    }
    fn translation_times(&mut self, other: Self) {
        self.form = if other.form == TransformForm::Scale {
            TransformForm::Scale
        } else {
            TransformForm::Compound
        };
        self.scale = other.scale;
        self.offset += other.offset;
        if other.form != TransformForm::Scale {
            self.matrix = other.matrix;
        }
    }
    fn scale_times(&mut self, other: Self) {
        self.form = TransformForm::Compound;
        self.offset += other.offset * self.scale;
        self.scale *= other.scale;
        self.matrix = other.matrix;
    }
    fn times_scale(&mut self, other: Self) {
        self.form = TransformForm::Compound;
        self.offset += self.scaled_offset(other.offset);
        self.scale = if self.scale == 1.0 {
            other.scale
        } else {
            self.scale * other.scale
        };
    }
    fn multiply_general(&mut self, other: Self) {
        self.form = TransformForm::Compound;
        self.offset += self.scaled_offset(other.offset);
        self.scale = if self.scale == 1.0 {
            other.scale
        } else {
            self.scale * other.scale
        };
        self.matrix *= other.matrix;
    }
    /// Checks scale before native reciprocal and transpose ordering; `transform.inverted()`.
    pub fn inverted(mut self) -> Result<Self, Failure> {
        if !matches!(
            self.form,
            TransformForm::Identity | TransformForm::Translation
        ) && self.scale.abs() <= f64::MIN_POSITIVE
        {
            return Err(super::super::construction(
                "gp_Trsf::Invert() - transformation has zero scale",
            ));
        }
        match self.form {
            TransformForm::Identity => {}
            TransformForm::Translation => self.offset.reverse(),
            TransformForm::Scale => {
                self.scale = 1.0 / self.scale;
                self.offset *= -self.scale;
            }
            _ => {
                self.scale = 1.0 / self.scale;
                self.matrix.transpose();
                self.offset = self.matrix.apply(self.offset) * -self.scale;
            }
        }
        Ok(self)
    }
    /// Reproduces native doubling, including compound temporary-scale ordering; `transform.powered(3)`.
    pub fn powered(self, exponent: i32) -> Result<Self, Failure> {
        if self.is_identity() || exponent == 1 {
            return Ok(self);
        }
        if exponent == 0 {
            return Ok(Self::default());
        }
        let mut result = if exponent < 0 { self.inverted()? } else { self };
        let mut remaining = exponent.unsigned_abs() - 1;
        if remaining == 0 {
            return Ok(result);
        }
        let mut factor = result;
        let rotation_offset = result.offset != Xyz::default();
        loop {
            if remaining % 2 == 1 {
                result.power_multiply(factor, rotation_offset);
            }
            if remaining == 1 {
                return Ok(result);
            }
            factor.power_square(rotation_offset);
            remaining /= 2;
        }
    }
    fn power_multiply(&mut self, factor: Self, rotation_offset: bool) {
        match self.form {
            TransformForm::Rotation => {
                if rotation_offset {
                    self.offset += self.matrix.apply(factor.offset);
                }
                self.matrix *= factor.matrix;
            }
            TransformForm::Compound => {
                self.offset += self.matrix.apply(factor.offset) * self.scale;
                self.scale *= factor.scale;
                self.matrix *= factor.matrix;
            }
            _ => *self = self.multiplied(factor),
        }
    }
    fn power_square(&mut self, rotation_offset: bool) {
        if self.form == TransformForm::Compound {
            // Pinned gp_Trsf::Power squares Tempscale before using it to update Temploc.
            self.scale *= self.scale;
            self.offset += self.matrix.apply(self.offset) * self.scale;
            self.matrix *= self.matrix;
        } else {
            self.power_multiply(*self, rotation_offset);
        }
    }
}
