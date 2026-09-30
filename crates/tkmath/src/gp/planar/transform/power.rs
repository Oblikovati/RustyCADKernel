//! gp_Trsf2d::Invert and gp_Trsf2d::Power with their per-form binary exponentiation.
use super::super::{Failure, Matrix2, TransformForm, Xy, construction};
use super::Transform2;
use TransformForm::*;
impl Transform2 {
    /// Returns the inverse; a zero scale raises before any change; `transform.inverted()?`.
    pub fn inverted(mut self) -> Result<Self, Failure> {
        self.invert()?;
        Ok(self)
    }
    /// Inverts in place: the matrix is a rotation or reflection, so its inverse is its
    /// transpose and the scale is inverted separately; `transform.invert()?`.
    pub fn invert(&mut self) -> Result<(), Failure> {
        match self.form {
            Identity => {}
            Translation | PointMirror => self.offset.reverse(),
            Scale => {
                self.check_invertible_scale()?;
                self.scale = 1.0 / self.scale;
                self.offset *= -self.scale;
            }
            _ => {
                self.check_invertible_scale()?;
                self.scale = 1.0 / self.scale;
                self.matrix.transpose();
                self.offset = self.offset.multiplied_matrix(self.matrix) * -self.scale;
            }
        }
        Ok(())
    }
    fn check_invertible_scale(&self) -> Result<(), Failure> {
        if self.scale.abs() <= f64::MIN_POSITIVE {
            return Err(construction(&format!(
                "gp_Trsf2d::Invert() - transformation has zero scale {}",
                self.scale
            )));
        }
        Ok(())
    }
    /// Returns the integer power; see `power`; `transform.powered(3)?`.
    pub fn powered(mut self, exponent: i32) -> Result<Self, Failure> {
        self.power(exponent)?;
        Ok(self)
    }
    /// gp_Trsf2d::Power: 0 resets to identity, negative powers invert first, and the
    /// rotation and compound forms square their matrix with gp_Mat2d's aliased self-product;
    /// `transform.power(3)?`.
    pub fn power(&mut self, exponent: i32) -> Result<(), Failure> {
        if self.form == Identity || exponent == 1 {
            return Ok(());
        }
        if exponent == 0 {
            *self = Self::default();
            return Ok(());
        }
        if exponent < 0 {
            self.invert()?;
        }
        if exponent != -1 {
            self.power_positive(exponent);
        }
        Ok(())
    }
    fn power_positive(&mut self, exponent: i32) {
        let remaining = exponent.wrapping_abs().wrapping_sub(1);
        match self.form {
            Translation => self.power_translation(remaining),
            Scale => self.power_scale(remaining),
            Rotation => self.power_rotation(remaining),
            PointMirror | AxisMirror if exponent % 2 == 0 => *self = Self::default(),
            PointMirror | AxisMirror => {}
            _ => self.power_compound(remaining),
        }
    }
    fn power_translation(&mut self, mut remaining: i32) {
        let mut factor: Xy = self.offset;
        loop {
            if remaining % 2 == 1 {
                self.offset += factor;
            }
            if remaining == 1 {
                return;
            }
            factor += factor;
            remaining /= 2;
        }
    }
    fn power_scale(&mut self, mut remaining: i32) {
        let (mut factor, mut factor_scale): (Xy, f64) = (self.offset, self.scale);
        loop {
            if remaining % 2 == 1 {
                self.offset += factor * self.scale;
                self.scale *= factor_scale;
            }
            if remaining == 1 {
                return;
            }
            factor += factor * factor_scale;
            factor_scale *= factor_scale;
            remaining /= 2;
        }
    }
}
impl Transform2 {
    fn power_rotation(&mut self, mut remaining: i32) {
        let mut factor: Matrix2 = self.matrix;
        let moves_origin = self.offset != Xy::default();
        let mut factor_offset: Xy = self.offset;
        loop {
            if remaining % 2 == 1 {
                if moves_origin {
                    self.offset += factor_offset.multiplied_matrix(self.matrix);
                }
                self.matrix.multiply(factor);
            }
            if remaining == 1 {
                return;
            }
            if moves_origin {
                factor_offset += factor_offset.multiplied_matrix(factor);
            }
            factor.square_in_place_aliased();
            remaining /= 2;
        }
    }
    /// The compound branch folds the scale into the diagonal only, then squares the temporary
    /// scale before it scales the temporary offset, as the pinned source does.
    fn power_compound(&mut self, mut remaining: i32) {
        self.form = Compound;
        let m = self.matrix.coefficients;
        self.matrix
            .set_diagonal(self.scale * m[0][0], self.scale * m[1][1]);
        let (mut factor_offset, mut factor_scale, mut factor) =
            (self.offset, self.scale, self.matrix);
        loop {
            if remaining % 2 == 1 {
                self.offset += factor_offset.multiplied_matrix(self.matrix) * self.scale;
                self.scale *= factor_scale;
                self.matrix.multiply(factor);
            }
            if remaining == 1 {
                return;
            }
            factor_scale *= factor_scale;
            factor_offset += factor_offset.multiplied_matrix(factor) * factor_scale;
            factor.square_in_place_aliased();
            remaining /= 2;
        }
    }
}
