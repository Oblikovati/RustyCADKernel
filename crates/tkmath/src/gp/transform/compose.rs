//! gp_Trsf::Multiply: `self = self * other`, one branch per native form pair, in the native
//! if-chain order (match arms are tried in order, as the chain is).
use super::super::{Transform3, TransformForm, Xyz};
use TransformForm::*;
use std::ops;
impl Transform3 {
    /// Returns `self * other` (gp_Trsf::Multiplied); `left.multiplied(right)`.
    pub fn multiplied(mut self, other: Self) -> Self {
        self.multiply(other);
        self
    }
    /// Right-composes in place (gp_Trsf::Multiply). The operand is a copy, so `t.multiply(t)`
    /// never sees the partial updates a native self-product would; `left.multiply(right)`.
    pub fn multiply(&mut self, other: Self) {
        if other.form == Identity {
            return;
        }
        if self.form == Identity {
            *self = other;
            return;
        }
        match (self.form, other.form) {
            (Rotation, Rotation) => self.multiply_rotations(other),
            (Translation, Translation) => self.offset += other.offset,
            (Scale, Scale) => {
                self.offset += other.offset * self.scale;
                self.scale *= other.scale;
            }
            (PointMirror, PointMirror) => {
                self.scale = 1.0;
                self.form = Translation;
                self.offset += other.offset.reversed();
            }
            (AxisMirror, AxisMirror) => {
                self.form = Rotation;
                self.offset += self.matrix.apply(other.offset);
                self.matrix *= other.matrix;
            }
            (Compound | Rotation | AxisMirror | PlaneMirror, Translation) => {
                self.offset += self.scaled_matrix_offset(other.offset)
            }
            (Scale | PointMirror, Translation) => self.offset += other.offset * self.scale,
            _ => self.multiply_mixed(other),
        }
    }
    fn multiply_mixed(&mut self, other: Self) {
        match (self.form, other.form) {
            (Translation, Compound | Rotation | AxisMirror | PlaneMirror) => {
                self.form = Compound;
                self.scale = other.scale;
                self.offset += other.offset;
                self.matrix = other.matrix;
            }
            (Translation, Scale | PointMirror) => {
                self.form = other.form;
                self.offset += other.offset;
                self.scale = other.scale;
            }
            (PointMirror | Scale, PointMirror | Scale) => {
                self.form = Compound;
                self.offset += other.offset * self.scale;
                self.scale *= other.scale;
            }
            (Compound | Rotation | AxisMirror | PlaneMirror, Scale | PointMirror) => {
                self.multiply_by_scale(other)
            }
            (Scale | PointMirror, Compound | Rotation | AxisMirror | PlaneMirror) => {
                self.form = Compound;
                self.offset += other.offset * self.scale;
                self.scale *= other.scale;
                self.matrix = other.matrix;
            }
            _ => self.multiply_general(other),
        }
    }
    /// `Tloc * matrix`, then `* scale` unless the scale is 1.
    fn scaled_matrix_offset(&self, offset: Xyz) -> Xyz {
        let mut result = self.matrix.apply(offset);
        if self.scale != 1.0 {
            result *= self.scale;
        }
        result
    }
    fn multiply_rotations(&mut self, other: Self) {
        // gp_Trsf checks the right-hand offset, unlike gp_Trsf2d's check of its own offset.
        if other.offset != Xyz::default() {
            self.offset += self.matrix.apply(other.offset);
        }
        self.matrix *= other.matrix;
    }
    fn multiply_by_scale(&mut self, other: Self) {
        self.form = Compound;
        let mut offset = self.matrix.apply(other.offset);
        if self.scale == 1.0 {
            self.scale = other.scale;
        } else {
            offset *= self.scale;
            self.scale *= other.scale;
        }
        self.offset += offset;
    }
    fn multiply_general(&mut self, other: Self) {
        self.form = Compound;
        let mut offset = self.matrix.apply(other.offset);
        if self.scale != 1.0 {
            offset *= self.scale;
            self.scale *= other.scale;
        } else {
            self.scale = other.scale;
        }
        self.offset += offset;
        self.matrix *= other.matrix;
    }
}
impl ops::Mul for Transform3 {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        self.multiplied(other)
    }
}
impl ops::MulAssign for Transform3 {
    fn mul_assign(&mut self, other: Self) {
        self.multiply(other);
    }
}
