// Copyright (c) 1991-1999 Matra Datavision
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

//! gp_GTrsf general affine matrix and affinity construction paths.
use super::{Axis1, Axis2, Matrix3, Xyz};
/// An affine transformation allowing independent directional scaling; `GeneralTransform3::default()`.
#[derive(Clone, Copy, Debug)]
pub struct GeneralTransform3 {
    matrix: Matrix3,
    offset: Xyz,
}
impl Default for GeneralTransform3 {
    fn default() -> Self {
        let mut matrix = Matrix3::default();
        matrix.set_identity();
        Self {
            matrix,
            offset: Xyz::default(),
        }
    }
}
impl GeneralTransform3 {
    /// Assigns the general vectorial and translation parts; `GeneralTransform3::new(matrix,offset)`.
    pub const fn new(matrix: Matrix3, offset: Xyz) -> Self {
        Self { matrix, offset }
    }
    /// Sets affinity perpendicular to an oriented line; `transform.set_affinity_axis(axis,2.0)`.
    pub fn set_affinity_axis(&mut self, axis: Axis1, ratio: f64) {
        self.matrix.set_dot(Xyz::from(axis.direction()));
        self.matrix *= 1.0 - ratio;
        self.add_diagonal(ratio);
        let origin = Xyz::from(axis.location());
        self.offset = self.matrix.apply(origin.reversed()) + origin;
    }
    /// Sets affinity along a plane normal (OCC13963); `transform.set_affinity_plane(frame,2.0)`.
    pub fn set_affinity_plane(&mut self, frame: Axis2, ratio: f64) {
        self.matrix.set_dot(Xyz::from(frame.direction()));
        self.matrix *= ratio - 1.0;
        self.offset = self.matrix.apply(Xyz::from(frame.location()).reversed());
        self.add_diagonal(1.0);
    }
    fn add_diagonal(&mut self, value: f64) {
        let diagonal = self.matrix.diagonal();
        self.matrix.set_diagonal(
            diagonal.x() + value,
            diagonal.y() + value,
            diagonal.z() + value,
        );
    }
    /// Applies the matrix then offset; `transform.apply(xyz)`.
    pub fn apply(&self, coordinates: Xyz) -> Xyz {
        self.matrix.apply(coordinates) + self.offset
    }
    /// Updates a coordinate triple in place; `transform.transforms(&mut xyz)`.
    pub fn transforms(&self, coordinates: &mut Xyz) {
        *coordinates = self.apply(*coordinates);
    }
}
