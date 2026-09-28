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

//! gp_Trsf2d::PreMultiply: `self = other * self`, in the native if-chain order.
use super::super::TransformForm;
use super::Transform2;
use TransformForm::*;
impl Transform2 {
    /// Left-composes in place (gp_Trsf2d::PreMultiply); `right.pre_multiply(left)`.
    pub fn pre_multiply(&mut self, other: Self) {
        if other.form == Identity {
            return;
        }
        if self.form == Identity {
            *self = other;
            return;
        }
        match (self.form, other.form) {
            (Rotation, Rotation) => {
                self.offset = self.offset.multiplied_matrix(other.matrix) + other.offset;
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
            (AxisMirror, AxisMirror) => self.pre_multiply_axis_mirrors(other),
            (Compound | Rotation | AxisMirror | Scale | PointMirror, Translation) => {
                self.offset += other.offset
            }
            _ => self.pre_multiply_mixed(other),
        }
    }
    fn pre_multiply_mixed(&mut self, other: Self) {
        match (self.form, other.form) {
            (Translation, Compound | Rotation | AxisMirror) => self.pre_multiply_matrix(other),
            (Translation, Scale | PointMirror) => {
                self.offset = self.offset * other.scale + other.offset;
                self.scale = other.scale;
                self.form = other.form;
            }
            (PointMirror | Scale, PointMirror | Scale)
            | (Compound | Rotation | AxisMirror, Scale | PointMirror) => {
                self.form = Compound;
                self.offset = self.offset * other.scale + other.offset;
                self.scale *= other.scale;
            }
            (Scale | PointMirror, Compound | Rotation | AxisMirror) => {
                self.pre_multiply_matrix(other)
            }
            _ => self.pre_multiply_general(other),
        }
    }
    fn pre_multiply_axis_mirrors(&mut self, other: Self) {
        self.form = Rotation;
        self.offset = self.offset.multiplied_matrix(other.matrix) * other.scale;
        self.scale *= other.scale;
        self.offset += other.offset;
        self.matrix.pre_multiply(other.matrix);
    }
    /// Translation and Scale/PointMirror left operands both adopt the other matrix. For a
    /// non-unit other scale, a translation takes that scale and a scale form multiplies it.
    fn pre_multiply_matrix(&mut self, other: Self) {
        let translation = self.form == Translation;
        self.form = Compound;
        self.matrix = other.matrix;
        if other.scale == 1.0 {
            self.offset = self.offset.multiplied_matrix(other.matrix);
        } else {
            self.offset = self.offset.multiplied_matrix(self.matrix) * other.scale;
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
        self.offset = self.offset.multiplied_matrix(other.matrix);
        if other.scale != 1.0 {
            self.offset *= other.scale;
            self.scale *= other.scale;
        }
        self.offset += other.offset;
        self.matrix.pre_multiply(other.matrix);
    }
}
