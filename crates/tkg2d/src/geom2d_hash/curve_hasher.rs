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
// Rust translation of ModelingData/TKG2d/Geom2dHash/Geom2dHash_CurveHasher.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dHash_CurveHasher`: the polymorphic hasher that dispatches on the exact
//! dynamic type of a `Geom2d_Curve` to the typed hashers.
use super::class_dispatch::{decided_equality, unknown_class_hash};
use super::conic_hashers::{
    Geom2dHashCircleHasher, Geom2dHashEllipseHasher, Geom2dHashHyperbolaHasher,
    Geom2dHashLineHasher, Geom2dHashParabolaHasher,
};
use super::spline_hashers::{Geom2dHashBSplineCurveHasher, Geom2dHashBezierCurveHasher};
use super::wrapped_hashers::{Geom2dHashOffsetCurveHasher, Geom2dHashTrimmedCurveHasher};
use crate::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve, Geom2dParabola, Geom2dTrimmedCurve,
};
use crate::hasher_class_dispatch;
use rustycad_tkernel::ncollection::HashPolicy;
use rustycad_tkernel::standard::Handle;

hasher_tolerances!(
    /// `Geom2dHash_CurveHasher`: hashes and compares any `Geom2d_Curve` handle, for
    /// geometry deduplication in hashed containers; a null handle hashes to 0;
    /// `Geom2dHashCurveHasher::default().hash(&curve)`.
    Geom2dHashCurveHasher
);

impl Geom2dHashCurveHasher {
    /// The typed hash of the curve's exact class, 0 for a null handle, and the
    /// Standard_CStringHasher hash of the type name for any other class, such as the
    /// Geom2dEval curves (`operator()(const occ::handle<Geom2d_Curve>&)`);
    /// `hasher.hash(&curve)`.
    pub fn hash(&self, curve: &Handle<dyn Geom2dCurve>) -> usize {
        let (comp, tol) = (self.comp_tolerance, self.hash_tolerance);
        hasher_class_dispatch!(hash curve, comp, tol;
            Geom2dLine => Geom2dHashLineHasher,
            Geom2dCircle => Geom2dHashCircleHasher,
            Geom2dEllipse => Geom2dHashEllipseHasher,
            Geom2dHyperbola => Geom2dHashHyperbolaHasher,
            Geom2dParabola => Geom2dHashParabolaHasher,
            Geom2dBezierCurve => Geom2dHashBezierCurveHasher,
            Geom2dBSplineCurve => Geom2dHashBSplineCurveHasher,
            Geom2dTrimmedCurve => Geom2dHashTrimmedCurveHasher,
            Geom2dOffsetCurve => Geom2dHashOffsetCurveHasher);
        unknown_class_hash(curve)
    }

    /// Whether two curves are equal for deduplication (`operator()(handle, handle)`):
    /// both null, the same object, or the same exact class compared by its typed
    /// hasher; objects of other classes compare by identity; `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &Handle<dyn Geom2dCurve>, second: &Handle<dyn Geom2dCurve>) -> bool {
        if let Some(decided) = decided_equality(first, second) {
            return decided;
        }
        let (comp, tol) = (self.comp_tolerance, self.hash_tolerance);
        hasher_class_dispatch!(equal first, second, comp, tol;
            Geom2dLine => Geom2dHashLineHasher,
            Geom2dCircle => Geom2dHashCircleHasher,
            Geom2dEllipse => Geom2dHashEllipseHasher,
            Geom2dHyperbola => Geom2dHashHyperbolaHasher,
            Geom2dParabola => Geom2dHashParabolaHasher,
            Geom2dBezierCurve => Geom2dHashBezierCurveHasher,
            Geom2dBSplineCurve => Geom2dHashBSplineCurveHasher,
            Geom2dTrimmedCurve => Geom2dHashTrimmedCurveHasher,
            Geom2dOffsetCurve => Geom2dHashOffsetCurveHasher);
        // Other classes compare by object identity, already found different.
        false
    }
}

impl HashPolicy<Handle<dyn Geom2dCurve>> for Geom2dHashCurveHasher {
    fn hash(&self, key: &Handle<dyn Geom2dCurve>) -> usize {
        Geom2dHashCurveHasher::hash(self, key)
    }
    fn equal(&self, first: &Handle<dyn Geom2dCurve>, second: &Handle<dyn Geom2dCurve>) -> bool {
        Geom2dHashCurveHasher::equal(self, first, second)
    }
}
