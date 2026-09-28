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
// Rust translation of CreateBezierCache2d/CreateBezierCache and the DN loops of ModelingData/TKG2d/Geom2dGridEval/Geom2dGridEval_BezierCurve.cxx and ModelingData/TKG3d/GeomGridEval/GeomGridEval_BezierCurve.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Bezier-curve data that the 2D and 3D grid evaluators read (degree, poles,
//! weights, flat knots), its single-span `BSplCLib_Cache` and the `BSplCLib::DN` call;
//! the two native files differ only in the point type.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib::{self, Cache, CurvePoint, CurveSpan};

/// The arrays that the native evaluators read by reference (`Degree()`, `Poles()`,
/// `Weights()`, `KnotSequence()`); here they are copies taken once per call;
/// `BezierCurveArrays { degree: 2, poles, weights: None, flat_knots }.cache()?`.
pub struct BezierCurveArrays<P: 'static> {
    /// The degree.
    pub degree: i32,
    /// The poles.
    pub poles: Array1<'static, P>,
    /// The weights of a rational curve.
    pub weights: Option<Array1<'static, f64>>,
    /// The flat knots `0...0 1...1`.
    pub flat_knots: Array1<'static, f64>,
}

impl<P: CurvePoint + 'static> BezierCurveArrays<P> {
    /// `CreateBezierCache`: the cache of the single span, built at 0.5;
    /// `arrays.cache()?.d0::<Point3>(0.25)?`.
    pub fn cache(&self) -> Result<Cache, Failure> {
        let weights: Option<&Array1<'static, f64>> = self.weights.as_ref();
        let mut cache: Cache =
            Cache::new(self.degree, false, &self.flat_knots, &self.poles, weights)?;
        cache.build_cache(0.5, &self.flat_knots, &self.poles, weights)?;
        Ok(cache)
    }
    /// `BSplCLib::DN(U, N, 0, Degree, false, Poles, Weights, KnotSequence, nullptr)`;
    /// `arrays.dn(0.25, 2)?`.
    pub fn dn(&self, u: f64, n: i32) -> Result<P::Vector, Failure> {
        let span = CurveSpan {
            u,
            index: 0,
            degree: self.degree,
            periodic: false,
            knots: &self.flat_knots,
            mults: None,
        };
        bsplclib::dn(&span, n, &self.poles, self.weights.as_ref())
    }
}
