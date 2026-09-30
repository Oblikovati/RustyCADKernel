//! gp_Trsf2d::Multiply: `self = self * other`, one branch per native form pair, in the
//! native if-chain order (match arms are tried in order, as the chain is).
use super::super::{TransformForm, Xy};
use super::Transform2;
use TransformForm::*;
use std::ops;
impl Transform2 {
    /// Returns `self * other` (gp_Trsf2d::Multiplied); `left.multiplied(right)`.
    pub fn multiplied(mut self, other: Self) -> Self {
        self.multiply(other);
        self
    }
    /// Right-composes in place (gp_Trsf2d::Multiply). The operand is a copy, so `t.multiply(t)`
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
            (AxisMirror, AxisMirror) => self.multiply_axis_mirrors(other),
            (Compound | Rotation | AxisMirror, Translation) => {
                self.offset += self.apply_vector(other.offset)
            }
            (Scale | PointMirror, Translation) => self.offset += other.offset * self.scale,
            _ => self.multiply_mixed(other),
        }
    }
    fn multiply_mixed(&mut self, other: Self) {
        match (self.form, other.form) {
            (Translation, Compound | Rotation | AxisMirror) => {
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
            (Compound | Rotation | AxisMirror, Scale | PointMirror) => {
                self.multiply_by_scale(other)
            }
            (Scale | PointMirror, Compound | Rotation | AxisMirror) => {
                self.form = Compound;
                let offset: Xy = other.offset * self.scale;
                self.scale *= other.scale;
                self.offset += offset;
                self.matrix = other.matrix;
            }
            _ => self.multiply_general(other),
        }
    }
    fn multiply_rotations(&mut self, other: Self) {
        // gp_Trsf2d checks this offset, unlike gp_Trsf's check of the right-hand offset.
        if self.offset != Xy::default() {
            self.offset += other.offset.multiplied_matrix(self.matrix);
        }
        self.matrix.multiply(other.matrix);
    }
    fn multiply_axis_mirrors(&mut self, other: Self) {
        self.form = Rotation;
        let offset: Xy = other.offset.multiplied_matrix(self.matrix) * self.scale;
        self.scale *= other.scale;
        self.offset += offset;
        self.matrix.multiply(other.matrix);
    }
    fn multiply_by_scale(&mut self, other: Self) {
        self.form = Compound;
        let mut offset: Xy = other.offset.multiplied_matrix(self.matrix);
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
        let mut offset: Xy = other.offset.multiplied_matrix(self.matrix);
        if self.scale != 1.0 {
            offset *= self.scale;
            self.scale *= other.scale;
        } else {
            self.scale = other.scale;
        }
        self.offset += offset;
        self.matrix.multiply(other.matrix);
    }
}
impl ops::Mul for Transform2 {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        self.multiplied(other)
    }
}
impl ops::MulAssign for Transform2 {
    fn mul_assign(&mut self, other: Self) {
        self.multiply(other);
    }
}
