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

//! gp_Trsf2d partial updates that re-derive the form, plus the derived parts.
use super::super::{Failure, Matrix2, TransformForm, Vector2, Xy, construction};
use super::Transform2;
use TransformForm::*;
impl Transform2 {
    /// Replaces the translation and re-derives the form from the old one;
    /// `transform.set_translation_part(vector)`.
    pub fn set_translation_part(&mut self, vector: Vector2) {
        self.offset = vector.xy();
        let null = self.offset.x().abs() <= f64::MIN_POSITIVE
            && self.offset.y().abs() <= f64::MIN_POSITIVE;
        self.form = match (null, self.form) {
            (true, Identity | PointMirror | Scale | Rotation | AxisMirror) => self.form,
            (true, Translation) => Identity,
            (false, Translation | Scale | PointMirror) => self.form,
            (false, Identity) => Translation,
            _ => Compound,
        };
    }
    /// Replaces the scale and re-derives the form; no zero check, as in gp_Trsf2d.cxx;
    /// `transform.set_scale_factor(2.0)`.
    pub fn set_scale_factor(&mut self, scale: f64) {
        self.form = if scale == 1.0 {
            self.unit_scale_form()
        } else if scale == -1.0 {
            match self.form {
                PointMirror | AxisMirror => self.form,
                Identity | Scale => PointMirror,
                _ => Compound,
            }
        } else {
            match self.form {
                Scale | Identity | Translation | PointMirror => Scale,
                _ => Compound,
            }
        };
        self.scale = scale;
    }
    fn unit_scale_form(&self) -> TransformForm {
        let null = self.offset.x().abs() <= f64::MIN_POSITIVE
            && self.offset.y().abs() <= f64::MIN_POSITIVE;
        match (null, self.form) {
            (_, Identity | Rotation) => self.form,
            (true, Scale) => Identity,
            (false, Scale) => Scale,
            (_, PointMirror) => Translation,
            _ => Compound,
        }
    }
    /// Returns the scaled matrix; scale and point-mirror forms scale only the diagonal;
    /// `transform.vectorial_part()`.
    pub fn vectorial_part(&self) -> Matrix2 {
        if self.scale == 1.0 {
            return self.matrix;
        }
        let mut matrix = self.matrix;
        if matches!(self.form, Scale | PointMirror) {
            let m = self.matrix.coefficients;
            matrix.set_diagonal(m[0][0] * self.scale, m[1][1] * self.scale);
        } else {
            matrix.multiply_scalar(self.scale);
        }
        matrix
    }
    /// Returns `atan2(a21, a11)` of the unscaled matrix; `transform.rotation_part()`.
    pub fn rotation_part(&self) -> f64 {
        let m = self.matrix.coefficients;
        m[1][0].atan2(m[0][0])
    }
    /// Sets a general similarity from a 2x3 matrix, factoring `sqrt(|det|)` out and
    /// orthogonalizing the rest. A failed orthogonalization leaves the scale, form and divided
    /// matrix already assigned, as the native exception does; `transform.set_values(1.,0.,2.,0.,1.,3.)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn set_values(
        &mut self,
        a11: f64,
        a12: f64,
        a13: f64,
        a21: f64,
        a22: f64,
        a23: f64,
    ) -> Result<(), Failure> {
        let mut matrix = Matrix2::from_columns(Xy::new(a11, a21), Xy::new(a12, a22));
        let determinant = matrix.determinant();
        if determinant.abs() < f64::MIN_POSITIVE {
            return Err(construction(&format!(
                "gp_Trsf2d::SetValues, null determinant {determinant}"
            )));
        }
        let scale = if determinant > 0.0 {
            determinant.sqrt()
        } else {
            (-determinant).sqrt()
        };
        matrix.divide(scale);
        self.scale = scale;
        self.form = Compound;
        self.matrix = matrix;
        self.orthogonalize()?;
        self.offset = Xy::new(a13, a23);
        Ok(())
    }
    /// Gram-Schmidt on the columns and then on the rows (gp_Trsf2d::Orthogonalize).
    fn orthogonalize(&mut self) -> Result<(), Failure> {
        let mut matrix = self.matrix;
        let (first, second) = orthonormal_pair(matrix.column(1)?, matrix.column(2)?)?;
        matrix.set_columns(first, second);
        let (first, second) = orthonormal_pair(matrix.row(1)?, matrix.row(2)?)?;
        matrix.set_rows(first, second);
        self.matrix = matrix;
        Ok(())
    }
}
fn orthonormal_pair(mut first: Xy, mut second: Xy) -> Result<(Xy, Xy), Failure> {
    first.normalize()?;
    second -= first * second.dot(first);
    second.normalize()?;
    Ok((first, second))
}
