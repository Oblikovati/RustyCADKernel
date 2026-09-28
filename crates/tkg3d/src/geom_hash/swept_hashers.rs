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
// Rust translation of ModelingData/TKG3d/GeomHash/GeomHash_SurfaceOfRevolutionHasher.pxx, GeomHash_SurfaceOfLinearExtrusionHasher.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Hashers of the swept surfaces: the basis curve through the polymorphic
//! [`GeomHashCurveHasher`], then the revolution axis or the extrusion direction.
use super::curve_hasher::GeomHashCurveHasher;
use super::placement_hashers::{GeomHashDirectionHasher, GeomHashPointHasher};
use crate::geom::{GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution, GeomSweptSurface};
use rustycad_tkg2d::geom2d_hash::hash_key::combined_hash;
use rustycad_tkg2d::hasher_tolerances;

hasher_tolerances!(
    /// `GeomHash_SurfaceOfRevolutionHasher`: a surface of revolution by basis curve and
    /// axis; `GeomHashSurfaceOfRevolutionHasher::default().hash(&surface)`.
    GeomHashSurfaceOfRevolutionHasher
);
hasher_tolerances!(
    /// `GeomHash_SurfaceOfLinearExtrusionHasher`: an extrusion by basis curve and
    /// direction; `GeomHashSurfaceOfLinearExtrusionHasher::default().hash(&surface)`.
    GeomHashSurfaceOfLinearExtrusionHasher
);

// The curve and direction hashers every swept hasher builds from its tolerances.
fn swept_parts(comp: f64, tol: f64) -> (GeomHashCurveHasher, GeomHashDirectionHasher) {
    (
        GeomHashCurveHasher::new(comp, tol),
        GeomHashDirectionHasher::new(comp, tol),
    )
}

impl GeomHashSurfaceOfRevolutionHasher {
    /// Basis curve, axis location and axis direction hashes combined
    /// (`operator()(handle)`); `hasher.hash(&surface)`.
    pub fn hash(&self, surface: &GeomSurfaceOfRevolution) -> usize {
        let (comp, tol) = (self.comp_tolerance, self.hash_tolerance);
        let (curves, directions) = swept_parts(comp, tol);
        let axis = surface.axis();
        combined_hash(&[
            curves.hash(&surface.basis_curve()),
            GeomHashPointHasher::new(comp, tol).hash(axis.location()),
            directions.hash(axis.direction()),
        ])
    }
    /// Equal basis curves and axes (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &GeomSurfaceOfRevolution, second: &GeomSurfaceOfRevolution) -> bool {
        let (comp, tol) = (self.comp_tolerance, self.hash_tolerance);
        let (curves, directions) = swept_parts(comp, tol);
        let (axis1, axis2) = (first.axis(), second.axis());
        curves.equal(&first.basis_curve(), &second.basis_curve())
            && GeomHashPointHasher::new(comp, tol).equal(axis1.location(), axis2.location())
            && directions.equal(axis1.direction(), axis2.direction())
    }
}

impl GeomHashSurfaceOfLinearExtrusionHasher {
    /// Basis curve and direction hashes combined (`operator()(handle)`);
    /// `hasher.hash(&surface)`.
    pub fn hash(&self, surface: &GeomSurfaceOfLinearExtrusion) -> usize {
        let (curves, directions) = swept_parts(self.comp_tolerance, self.hash_tolerance);
        combined_hash(&[
            curves.hash(&surface.basis_curve()),
            directions.hash(surface.direction()),
        ])
    }
    /// Equal basis curves and directions (`operator()(handle, handle)`);
    /// `hasher.equal(&a, &b)`.
    pub fn equal(
        &self,
        first: &GeomSurfaceOfLinearExtrusion,
        second: &GeomSurfaceOfLinearExtrusion,
    ) -> bool {
        let (curves, directions) = swept_parts(self.comp_tolerance, self.hash_tolerance);
        curves.equal(&first.basis_curve(), &second.basis_curve())
            && directions.equal(first.direction(), second.direction())
    }
}
