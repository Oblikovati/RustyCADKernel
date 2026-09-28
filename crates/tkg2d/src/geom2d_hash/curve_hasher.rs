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
use rustycad_tkernel::ncollection::HashPolicy;
use rustycad_tkernel::standard::c_string::CStringHasher;
use rustycad_tkernel::standard::{Handle, RuntimeTyped};

hasher_tolerances!(
    /// `Geom2dHash_CurveHasher`: hashes and compares any `Geom2d_Curve` handle, for
    /// geometry deduplication in hashed containers; a null handle hashes to 0;
    /// `Geom2dHashCurveHasher::default().hash(&curve)`.
    Geom2dHashCurveHasher
);

// The typed view of `curve` when its dynamic type is exactly `T`
// (`aType == STANDARD_TYPE(T)`, then `occ::down_cast<T>`).
fn exact<T: RuntimeTyped>(curve: &Handle<dyn Geom2dCurve>) -> Option<Handle<T>> {
    curve
        .is_instance(T::static_type())
        .then(|| curve.cast::<T>())
}

impl Geom2dHashCurveHasher {
    /// The typed hash of the curve's exact class, 0 for a null handle, and the
    /// Standard_CStringHasher hash of the type name for any other class, such as the
    /// Geom2dEval curves (`operator()(const occ::handle<Geom2d_Curve>&)`);
    /// `hasher.hash(&curve)`.
    pub fn hash(&self, curve: &Handle<dyn Geom2dCurve>) -> usize {
        let Some(descriptor) = curve.dynamic_type() else {
            return 0;
        };
        let (comp, hash) = (self.comp_tolerance, self.hash_tolerance);
        if let Some(line) = exact::<Geom2dLine>(curve) {
            return Geom2dHashLineHasher::new(comp, hash).hash(&line);
        }
        if let Some(circle) = exact::<Geom2dCircle>(curve) {
            return Geom2dHashCircleHasher::new(comp, hash).hash(&circle);
        }
        if let Some(ellipse) = exact::<Geom2dEllipse>(curve) {
            return Geom2dHashEllipseHasher::new(comp, hash).hash(&ellipse);
        }
        if let Some(hyperbola) = exact::<Geom2dHyperbola>(curve) {
            return Geom2dHashHyperbolaHasher::new(comp, hash).hash(&hyperbola);
        }
        self.hash_bounded_or_named(curve, descriptor.name())
    }
    fn hash_bounded_or_named(&self, curve: &Handle<dyn Geom2dCurve>, name: &str) -> usize {
        let (comp, hash) = (self.comp_tolerance, self.hash_tolerance);
        if let Some(parabola) = exact::<Geom2dParabola>(curve) {
            return Geom2dHashParabolaHasher::new(comp, hash).hash(&parabola);
        }
        if let Some(bezier) = exact::<Geom2dBezierCurve>(curve) {
            return Geom2dHashBezierCurveHasher::new(comp, hash).hash(&bezier);
        }
        if let Some(bspline) = exact::<Geom2dBSplineCurve>(curve) {
            return Geom2dHashBSplineCurveHasher::new(comp, hash).hash(&bspline);
        }
        if let Some(trimmed) = exact::<Geom2dTrimmedCurve>(curve) {
            return Geom2dHashTrimmedCurveHasher::new(comp, hash).hash(&trimmed);
        }
        if let Some(offset) = exact::<Geom2dOffsetCurve>(curve) {
            return Geom2dHashOffsetCurveHasher::new(comp, hash).hash(&offset);
        }
        CStringHasher.hash_code(name.as_bytes())
    }

    /// Whether two curves are equal for deduplication (`operator()(handle, handle)`):
    /// both null, the same object, or the same exact class compared by its typed
    /// hasher; objects of other classes compare by identity; `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &Handle<dyn Geom2dCurve>, second: &Handle<dyn Geom2dCurve>) -> bool {
        let (Some(first_type), Some(second_type)) = (first.dynamic_type(), second.dynamic_type())
        else {
            return first.is_null() && second.is_null();
        };
        if first == second {
            return true;
        }
        if first_type.rust_type() != second_type.rust_type() {
            return false;
        }
        self.equal_conics(first, second)
            .unwrap_or_else(|| self.equal_others(first, second))
    }
    // Some(result) when the pair has one of the line or conic classes.
    fn equal_conics(
        &self,
        first: &Handle<dyn Geom2dCurve>,
        second: &Handle<dyn Geom2dCurve>,
    ) -> Option<bool> {
        let (comp, hash) = (self.comp_tolerance, self.hash_tolerance);
        if let (Some(a), Some(b)) = (exact::<Geom2dLine>(first), exact::<Geom2dLine>(second)) {
            return Some(Geom2dHashLineHasher::new(comp, hash).equal(&a, &b));
        }
        if let (Some(a), Some(b)) = (exact::<Geom2dCircle>(first), exact::<Geom2dCircle>(second)) {
            return Some(Geom2dHashCircleHasher::new(comp, hash).equal(&a, &b));
        }
        if let (Some(a), Some(b)) = (
            exact::<Geom2dEllipse>(first),
            exact::<Geom2dEllipse>(second),
        ) {
            return Some(Geom2dHashEllipseHasher::new(comp, hash).equal(&a, &b));
        }
        if let (Some(a), Some(b)) = (
            exact::<Geom2dHyperbola>(first),
            exact::<Geom2dHyperbola>(second),
        ) {
            return Some(Geom2dHashHyperbolaHasher::new(comp, hash).equal(&a, &b));
        }
        if let (Some(a), Some(b)) = (
            exact::<Geom2dParabola>(first),
            exact::<Geom2dParabola>(second),
        ) {
            return Some(Geom2dHashParabolaHasher::new(comp, hash).equal(&a, &b));
        }
        None
    }
    // The spline and wrapper classes; any other class compares by object identity,
    // which the caller has already found different.
    fn equal_others(
        &self,
        first: &Handle<dyn Geom2dCurve>,
        second: &Handle<dyn Geom2dCurve>,
    ) -> bool {
        let (comp, hash) = (self.comp_tolerance, self.hash_tolerance);
        if let (Some(a), Some(b)) = (
            exact::<Geom2dBezierCurve>(first),
            exact::<Geom2dBezierCurve>(second),
        ) {
            return Geom2dHashBezierCurveHasher::new(comp, hash).equal(&a, &b);
        }
        if let (Some(a), Some(b)) = (
            exact::<Geom2dBSplineCurve>(first),
            exact::<Geom2dBSplineCurve>(second),
        ) {
            return Geom2dHashBSplineCurveHasher::new(comp, hash).equal(&a, &b);
        }
        if let (Some(a), Some(b)) = (
            exact::<Geom2dTrimmedCurve>(first),
            exact::<Geom2dTrimmedCurve>(second),
        ) {
            return Geom2dHashTrimmedCurveHasher::new(comp, hash).equal(&a, &b);
        }
        if let (Some(a), Some(b)) = (
            exact::<Geom2dOffsetCurve>(first),
            exact::<Geom2dOffsetCurve>(second),
        ) {
            return Geom2dHashOffsetCurveHasher::new(comp, hash).equal(&a, &b);
        }
        first.object_address() == second.object_address()
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
