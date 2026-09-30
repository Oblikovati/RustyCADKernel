//! gp_Trsf::PreMultiply: `self = other * self`, in the native if-chain order.
use super::super::{Transform3, TransformForm};
use TransformForm::*;
impl Transform3 {
    /// Left-composes in place (gp_Trsf::PreMultiply); `right.pre_multiply(left)`.
    pub fn pre_multiply(&mut self, other: Self) {
        if other.form == Identity {
            return;
        }
        if self.form == Identity {
            *self = other;
            return;
        }
        match (self.form, other.form) {
            (Rotation, Rotation) | (AxisMirror, AxisMirror) => {
                if self.form == AxisMirror {
                    self.form = Rotation;
                }
                self.offset = other.matrix.apply(self.offset) + other.offset;
                self.matrix.pre_multiply(other.matrix);
            }
            (Translation, Translation) => self.offset += other.offset,
            (Scale, Scale) => {
                self.offset = self.offset * other.scale + other.offset;
                self.scale *= other.scale;
            }
            (PointMirror, PointMirror) => {
                self.scale = 1.0;
                self.form = Translation;
                self.offset = self.offset.reversed() + other.offset;
            }
            (Compound | Rotation | AxisMirror | PlaneMirror | Scale | PointMirror, Translation) => {
                self.offset += other.offset
            }
            _ => self.pre_multiply_mixed(other),
        }
    }
    fn pre_multiply_mixed(&mut self, other: Self) {
        match (self.form, other.form) {
            (Translation, Compound | Rotation | AxisMirror | PlaneMirror) => {
                self.pre_multiply_matrix(other)
            }
            (Translation, Scale | PointMirror) => {
                self.offset = self.offset * other.scale + other.offset;
                self.scale = other.scale;
                self.form = other.form;
            }
            (PointMirror | Scale, PointMirror | Scale)
            | (Compound | Rotation | AxisMirror | PlaneMirror, Scale | PointMirror) => {
                self.form = Compound;
                self.offset = self.offset * other.scale + other.offset;
                self.scale *= other.scale;
            }
            (Scale | PointMirror, Compound | Rotation | AxisMirror | PlaneMirror) => {
                self.pre_multiply_matrix(other)
            }
            _ => self.pre_multiply_general(other),
        }
    }
    /// Translation and Scale/PointMirror left operands both adopt the other matrix. For a
    /// non-unit other scale, a translation takes that scale and a scale form multiplies it.
    fn pre_multiply_matrix(&mut self, other: Self) {
        let translation = self.form == Translation;
        self.form = Compound;
        self.matrix = other.matrix;
        if other.scale == 1.0 {
            self.offset = other.matrix.apply(self.offset);
        } else {
            self.offset = self.matrix.apply(self.offset) * other.scale;
            self.scale = if translation {
                other.scale
            } else {
                other.scale * self.scale
            };
        }
        self.offset += other.offset;
    }
    fn pre_multiply_general(&mut self, other: Self) {
        self.form = Compound;
        self.offset = other.matrix.apply(self.offset);
        if other.scale != 1.0 {
            self.offset *= other.scale;
            self.scale *= other.scale;
        }
        self.offset += other.offset;
        self.matrix.pre_multiply(other.matrix);
    }
}
