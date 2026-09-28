// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of GeomGridEval_Surface::applyTransformation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_Surface.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `applyTransformation`: every point and derivative of a grid moved by the
//! transformation of a `GeomAdaptor_TransformedSurface`, cell by cell.
use super::{GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};

/// A grid record that `applyTransformation` moves (`gp_Pnt::Transform`,
/// `gp_Vec::Transform` on each member, in declaration order).
pub(super) trait TransformedRecord {
    /// Transforms the record in place.
    fn transform_record(&mut self, trsf: Transform3);
}

impl TransformedRecord for Point3 {
    fn transform_record(&mut self, trsf: Transform3) {
        self.transform(trsf);
    }
}

impl TransformedRecord for Vector3 {
    fn transform_record(&mut self, trsf: Transform3) {
        self.transform(trsf);
    }
}

impl TransformedRecord for GeomGridEvalSurfD1 {
    fn transform_record(&mut self, trsf: Transform3) {
        self.point.transform(trsf);
        for v in [&mut self.d1u, &mut self.d1v] {
            v.transform(trsf);
        }
    }
}

impl TransformedRecord for GeomGridEvalSurfD2 {
    fn transform_record(&mut self, trsf: Transform3) {
        self.point.transform(trsf);
        for v in [
            &mut self.d1u,
            &mut self.d1v,
            &mut self.d2u,
            &mut self.d2v,
            &mut self.d2uv,
        ] {
            v.transform(trsf);
        }
    }
}

impl TransformedRecord for GeomGridEvalSurfD3 {
    fn transform_record(&mut self, trsf: Transform3) {
        self.point.transform(trsf);
        let vectors = [
            &mut self.d1u,
            &mut self.d1v,
            &mut self.d2u,
            &mut self.d2v,
            &mut self.d2uv,
            &mut self.d3u,
            &mut self.d3v,
            &mut self.d3uuv,
            &mut self.d3uvv,
        ];
        for v in vectors {
            v.transform(trsf);
        }
    }
}

/// `applyTransformation`: nothing without a transformation or for an empty grid.
pub(super) fn apply_transformation<T: TransformedRecord>(
    grid: &mut Array2<'_, T>,
    trsf: Option<Transform3>,
) {
    let Some(trsf) = trsf else {
        return;
    };
    for record in grid.iter_mut() {
        record.transform_record(trsf);
    }
}
