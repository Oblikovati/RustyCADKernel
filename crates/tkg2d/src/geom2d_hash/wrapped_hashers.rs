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
// Rust translation of ModelingData/TKG2d/Geom2dHash/Geom2dHash_TrimmedCurveHasher.pxx, Geom2dHash_OffsetCurveHasher.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Hashers of the curves that wrap a basis curve: the basis goes through the
//! polymorphic [`Geom2dHashCurveHasher`], then the trim bounds or the offset distance
//! are rounded (hash) or compared within tolerance (equality).
use super::curve_hasher::Geom2dHashCurveHasher;
use super::hash_key::{leading_and_rounded, values_within};
use crate::geom2d::{Geom2dCurve, Geom2dOffsetCurve, Geom2dTrimmedCurve};

hasher_tolerances!(
    /// `Geom2dHash_TrimmedCurveHasher`: a trimmed curve by basis curve and bounds;
    /// `Geom2dHashTrimmedCurveHasher::default().hash(&trimmed)`.
    Geom2dHashTrimmedCurveHasher
);
hasher_tolerances!(
    /// `Geom2dHash_OffsetCurveHasher`: an offset curve by basis curve and distance;
    /// `Geom2dHashOffsetCurveHasher::default().hash(&offset)`.
    Geom2dHashOffsetCurveHasher
);

// `[basis hash, round(value_i / HashTolerance)...]` combined.
fn basis_and_values(curves: Geom2dHashCurveHasher, basis_hash: usize, values: &[f64]) -> usize {
    leading_and_rounded(&[basis_hash], values, curves.hash_tolerance)
}

impl Geom2dHashTrimmedCurveHasher {
    fn curves(&self) -> Geom2dHashCurveHasher {
        Geom2dHashCurveHasher::new(self.comp_tolerance, self.hash_tolerance)
    }
    /// Basis hash and rounded bounds (`operator()(handle)`); `hasher.hash(&trimmed)`.
    pub fn hash(&self, curve: &Geom2dTrimmedCurve) -> usize {
        let basis_hash: usize = self.curves().hash(&curve.basis_curve());
        let bounds = [curve.first_parameter(), curve.last_parameter()];
        basis_and_values(self.curves(), basis_hash, &bounds)
    }
    /// Equal basis curves and bounds (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &Geom2dTrimmedCurve, second: &Geom2dTrimmedCurve) -> bool {
        self.curves()
            .equal(&first.basis_curve(), &second.basis_curve())
            && values_within(
                self.comp_tolerance,
                &[
                    (first.first_parameter(), second.first_parameter()),
                    (first.last_parameter(), second.last_parameter()),
                ],
            )
    }
}

impl Geom2dHashOffsetCurveHasher {
    fn curves(&self) -> Geom2dHashCurveHasher {
        Geom2dHashCurveHasher::new(self.comp_tolerance, self.hash_tolerance)
    }
    /// Basis hash and rounded offset (`operator()(handle)`); `hasher.hash(&offset)`.
    pub fn hash(&self, curve: &Geom2dOffsetCurve) -> usize {
        let basis_hash: usize = self.curves().hash(&curve.basis_curve());
        basis_and_values(self.curves(), basis_hash, &[curve.offset()])
    }
    /// Equal basis curves and offsets (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &Geom2dOffsetCurve, second: &Geom2dOffsetCurve) -> bool {
        self.curves()
            .equal(&first.basis_curve(), &second.basis_curve())
            && values_within(self.comp_tolerance, &[(first.offset(), second.offset())])
    }
}
