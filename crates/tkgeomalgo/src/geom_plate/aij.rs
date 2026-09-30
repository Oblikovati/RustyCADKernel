// Copyright (c) 1998-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_Aij.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomPlate_Aij`: the indices of two normals and their cross product.
use rustycad_tkmath::gp::Vector3;

/// Two normal indices and the unit cross product of the normals (`GeomPlate_Aij`).
///
/// The fields are private natively, with `GeomPlate_BuildAveragePlane` as friend; they
/// are visible to the `geom_plate` module only.
/// `let aij = GeomPlateAij::with_values(1, 2, Vector3::new(0.0, 0.0, 1.0));`
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GeomPlateAij {
    pub(super) ind1: i32,
    pub(super) ind2: i32,
    pub(super) vec: Vector3,
}

impl GeomPlateAij {
    /// Null indices and a null vector (`GeomPlate_Aij()`); `GeomPlateAij::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// The indices `ind1`, `ind2` and the vector `vec`
    /// (`GeomPlate_Aij(anInd1, anInd2, aVec)`);
    /// `GeomPlateAij::with_values(1, 2, Vector3::new(0.0, 0.0, 1.0))`.
    pub fn with_values(ind1: i32, ind2: i32, vec: Vector3) -> Self {
        Self { ind1, ind2, vec }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_and_valued_constructors_store_the_fields() {
        let empty = GeomPlateAij::new();
        assert_eq!((empty.ind1, empty.ind2), (0, 0));
        assert_eq!(empty.vec, Vector3::new(0.0, 0.0, 0.0));
        let aij = GeomPlateAij::with_values(3, 5, Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(
            (aij.ind1, aij.ind2, aij.vec),
            (3, 5, Vector3::new(1.0, 2.0, 3.0))
        );
    }
}
