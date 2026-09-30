//! gp_Trsf::Invert and gp_Trsf::Power with their per-form binary exponentiation.
use super::super::{Failure, Matrix3, Transform3, TransformForm, Xyz, construction};
use TransformForm::*;
impl Transform3 {
    /// Returns the inverse; a zero scale raises before any change; `transform.inverted()?`.
    pub fn inverted(mut self) -> Result<Self, Failure> {
        self.invert()?;
        Ok(self)
    }
    /// Inverts in place: the matrix is orthogonal, so its inverse is its transpose and the
    /// scale is inverted separately; `transform.invert()?`.
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
                self.offset = self.matrix.apply(self.offset) * -self.scale;
            }
        }
        Ok(())
    }
    fn check_invertible_scale(&self) -> Result<(), Failure> {
        if self.scale.abs() <= f64::MIN_POSITIVE {
            return Err(construction(&format!(
                "gp_Trsf::Invert() - transformation has zero scale {}",
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
    /// gp_Trsf::Power: 0 resets to identity, negative powers invert first, even powers of a
    /// mirror give identity, and the compound form squares its temporary scale before it
    /// scales the temporary offset; `transform.power(3)?`.
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
            PointMirror | AxisMirror | PlaneMirror if exponent % 2 == 0 => *self = Self::default(),
            PointMirror | AxisMirror | PlaneMirror => {}
            _ => self.power_compound(remaining),
        }
    }
    fn power_translation(&mut self, mut remaining: i32) {
        let mut factor: Xyz = self.offset;
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
        let (mut factor, mut factor_scale): (Xyz, f64) = (self.offset, self.scale);
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
    fn power_rotation(&mut self, mut remaining: i32) {
        let mut factor: Matrix3 = self.matrix;
        let moves_origin = self.offset != Xyz::default();
        let mut factor_offset: Xyz = self.offset;
        loop {
            if remaining % 2 == 1 {
                if moves_origin {
                    self.offset += self.matrix.apply(factor_offset);
                }
                self.matrix *= factor;
            }
            if remaining == 1 {
                return;
            }
            if moves_origin {
                factor_offset += factor.apply(factor_offset);
            }
            factor *= factor;
            remaining /= 2;
        }
    }
    fn power_compound(&mut self, mut remaining: i32) {
        self.form = Compound;
        let (mut factor_offset, mut factor_scale, mut factor) =
            (self.offset, self.scale, self.matrix);
        loop {
            if remaining % 2 == 1 {
                self.offset += self.matrix.apply(factor_offset) * self.scale;
                self.scale *= factor_scale;
                self.matrix *= factor;
            }
            if remaining == 1 {
                return;
            }
            // Pinned gp_Trsf::Power squares Tempscale before using it to update Temploc.
            factor_scale *= factor_scale;
            factor_offset += factor.apply(factor_offset) * factor_scale;
            factor *= factor;
            remaining /= 2;
        }
    }
}
