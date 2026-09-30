//! gp_GTrsf2d inversion, composition and powers. Similarity forms round-trip through
//! gp_Trsf2d, so its orthogonality checks and form arithmetic apply.
use super::super::{Failure, Matrix2, Transform2, TransformForm, Xy};
use super::GeneralTransform2;
use std::ops;
impl GeneralTransform2 {
    fn is_general(&self) -> bool {
        self.form == TransformForm::Other
    }
    /// Returns the inverse; `transform.inverted()?`.
    pub fn inverted(mut self) -> Result<Self, Failure> {
        self.invert()?;
        Ok(self)
    }
    /// Inverts the matrix of a general form (no zero check in a release build), otherwise
    /// inverts the equivalent gp_Trsf2d; `transform.invert()?`.
    pub fn invert(&mut self) -> Result<(), Failure> {
        if self.is_general() {
            self.matrix.invert()?;
            self.offset = self.offset.multiplied_matrix(self.matrix).reversed();
            return Ok(());
        }
        let mut similarity = self.trsf2d()?;
        similarity.invert()?;
        self.set_trsf2d(similarity);
        Ok(())
    }
    /// Returns `self * other`; `left.multiplied(right)?`.
    pub fn multiplied(mut self, other: Self) -> Result<Self, Failure> {
        self.multiply(other)?;
        Ok(self)
    }
    /// Right-composes; a general operand keeps this scale and forces form Other;
    /// `left.multiply(right)?`.
    pub fn multiply(&mut self, other: Self) -> Result<(), Failure> {
        if self.is_general() || other.is_general() {
            self.form = TransformForm::Other;
            self.offset += other.offset.multiplied_matrix(self.matrix);
            self.matrix.multiply(other.matrix);
            return Ok(());
        }
        let mut similarity: Transform2 = self.trsf2d()?;
        similarity.multiply(other.trsf2d()?);
        self.set_trsf2d(similarity);
        Ok(())
    }
    /// Left-composes, `self = other * self`; `right.pre_multiply(left)?`.
    pub fn pre_multiply(&mut self, other: Self) -> Result<(), Failure> {
        if self.is_general() || other.is_general() {
            self.form = TransformForm::Other;
            self.offset = self.offset.multiplied_matrix(other.matrix) + other.offset;
            self.matrix.pre_multiply(other.matrix);
            return Ok(());
        }
        let mut similarity: Transform2 = self.trsf2d()?;
        similarity.pre_multiply(other.trsf2d()?);
        self.set_trsf2d(similarity);
        Ok(())
    }
    /// Returns the integer power; see `power`; `transform.powered(3)?`.
    pub fn powered(mut self, exponent: i32) -> Result<Self, Failure> {
        self.power(exponent)?;
        Ok(self)
    }
    /// gp_GTrsf2d::Power. A negative exponent inverts first; for a similarity form the
    /// equivalent gp_Trsf2d then receives the same negative exponent and inverts again, so
    /// `T^-n` returns `T^n` there; `transform.power(3)?`.
    pub fn power(&mut self, exponent: i32) -> Result<(), Failure> {
        match exponent {
            0 => *self = Self::default(),
            1 => {}
            -1 => self.invert()?,
            _ => {
                if exponent < 0 {
                    self.invert()?;
                }
                self.power_repeated(exponent)?;
            }
        }
        Ok(())
    }
    fn power_repeated(&mut self, exponent: i32) -> Result<(), Failure> {
        if !self.is_general() {
            let mut similarity = self.trsf2d()?;
            similarity.power(exponent)?;
            self.set_trsf2d(similarity);
            return Ok(());
        }
        let mut remaining = exponent.wrapping_abs().wrapping_sub(1);
        let (mut factor_offset, mut factor): (Xy, Matrix2) = (self.offset, self.matrix);
        loop {
            if remaining % 2 == 1 {
                self.offset += factor_offset.multiplied_matrix(self.matrix);
                self.matrix.multiply(factor);
            }
            if remaining == 1 {
                return Ok(());
            }
            factor_offset += factor_offset.multiplied_matrix(factor);
            factor.square_in_place_aliased();
            remaining /= 2;
        }
    }
}
/// `gp_GTrsf2d::operator*`; the native ConstructionError of a similarity pair that fails
/// gp_GTrsf2d::Trsf2d's orthogonality checks is returned as `Err`. `operator*=` is `multiply`.
impl ops::Mul for GeneralTransform2 {
    type Output = Result<Self, Failure>;
    fn mul(self, other: Self) -> Result<Self, Failure> {
        self.multiplied(other)
    }
}
