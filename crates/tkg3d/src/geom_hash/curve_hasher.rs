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
// Rust translation of ModelingData/TKG3d/GeomHash/GeomHash_CurveHasher.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomHash_CurveHasher`: the polymorphic hasher that dispatches on the exact
//! dynamic type of a `Geom_Curve` to the typed hashers.
use super::conic_hashers::{
    GeomHashCircleHasher, GeomHashEllipseHasher, GeomHashHyperbolaHasher, GeomHashLineHasher,
    GeomHashParabolaHasher,
};
use super::spline_hashers::{GeomHashBSplineCurveHasher, GeomHashBezierCurveHasher};
use super::wrapped_hashers::{GeomHashOffsetCurveHasher, GeomHashTrimmedCurveHasher};
use crate::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomHyperbola, GeomLine,
    GeomOffsetCurve, GeomParabola, GeomTrimmedCurve,
};
use rustycad_tkernel::ncollection::HashPolicy;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d_hash::class_dispatch::{decided_equality, unknown_class_hash};
use rustycad_tkg2d::{hasher_class_dispatch, hasher_tolerances};

hasher_tolerances!(
    /// `GeomHash_CurveHasher`: hashes and compares any `Geom_Curve` handle, for
    /// geometry deduplication in hashed containers; a null handle hashes to 0;
    /// `GeomHashCurveHasher::default().hash(&curve)`.
    GeomHashCurveHasher
);

impl GeomHashCurveHasher {
    /// The typed hash of the curve's exact class, 0 for a null handle, and the
    /// Standard_CStringHasher hash of the type name for any other class, such as the
    /// GeomEval curves (`operator()(const occ::handle<Geom_Curve>&)`);
    /// `hasher.hash(&curve)`.
    pub fn hash(&self, curve: &Handle<dyn GeomCurve>) -> usize {
        let (comp, tol) = (self.comp_tolerance, self.hash_tolerance);
        hasher_class_dispatch!(hash curve, comp, tol;
            GeomLine => GeomHashLineHasher,
            GeomCircle => GeomHashCircleHasher,
            GeomEllipse => GeomHashEllipseHasher,
            GeomHyperbola => GeomHashHyperbolaHasher,
            GeomParabola => GeomHashParabolaHasher,
            GeomBezierCurve => GeomHashBezierCurveHasher,
            GeomBSplineCurve => GeomHashBSplineCurveHasher,
            GeomTrimmedCurve => GeomHashTrimmedCurveHasher,
            GeomOffsetCurve => GeomHashOffsetCurveHasher);
        unknown_class_hash(curve)
    }

    /// Whether two curves are equal for deduplication (`operator()(handle, handle)`):
    /// both null, the same object, or the same exact class compared by its typed
    /// hasher; objects of other classes compare by identity; `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &Handle<dyn GeomCurve>, second: &Handle<dyn GeomCurve>) -> bool {
        if let Some(decided) = decided_equality(first, second) {
            return decided;
        }
        let (comp, tol) = (self.comp_tolerance, self.hash_tolerance);
        hasher_class_dispatch!(equal first, second, comp, tol;
            GeomLine => GeomHashLineHasher,
            GeomCircle => GeomHashCircleHasher,
            GeomEllipse => GeomHashEllipseHasher,
            GeomHyperbola => GeomHashHyperbolaHasher,
            GeomParabola => GeomHashParabolaHasher,
            GeomBezierCurve => GeomHashBezierCurveHasher,
            GeomBSplineCurve => GeomHashBSplineCurveHasher,
            GeomTrimmedCurve => GeomHashTrimmedCurveHasher,
            GeomOffsetCurve => GeomHashOffsetCurveHasher);
        // Other classes compare by object identity, already found different.
        false
    }
}

impl HashPolicy<Handle<dyn GeomCurve>> for GeomHashCurveHasher {
    fn hash(&self, key: &Handle<dyn GeomCurve>) -> usize {
        GeomHashCurveHasher::hash(self, key)
    }
    fn equal(&self, first: &Handle<dyn GeomCurve>, second: &Handle<dyn GeomCurve>) -> bool {
        GeomHashCurveHasher::equal(self, first, second)
    }
}
