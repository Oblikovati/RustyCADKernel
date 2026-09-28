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
// Rust translation of FoundationClasses/TKMath/BSplSLib (BSplSLib.hxx/.lxx/.cxx, BSplSLib_BzSyntaxes.cxx, BSplSLib_Cache, BSplSLib_EvaluatorFunction) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! BSplSLib: the B-spline surface library built on BSplCLib.
//!
//! A surface is processed as a curve of curves: along one direction each pole of
//! the curve is a whole row of the pole grid. `BSplSLib::NoWeights()` becomes
//! `None`; C++ output references become returned values; native exceptions become
//! `Failure` values of the same type, and accesses outside an array or buffer that
//! OCCT would perform unchecked return Standard_OutOfRange.
#![allow(clippy::too_many_arguments)] // Native entry points take up to sixteen arguments; order is kept.

mod build_cache;
mod cache;
mod cache_eval;
mod eval;
mod eval_d3;
mod interpolate;
mod iso;
mod move_point;
mod ops;
mod poles;
mod prepare;
mod rational;
mod resolution;

pub use build_cache::{CacheSource, build_cache, build_cache_matrix, poles_coefficients};
pub use cache::Cache;
pub use cache_eval::{
    SurfaceCacheSpan, cache_d0, cache_d1, cache_d2, coefs_d0, coefs_d1, coefs_d2,
};
pub use eval::{SurfaceD2, d0, d1, d2, homogeneous_d0, homogeneous_d1};
pub use eval_d3::{SurfaceD3, d3, dn};
pub use interpolate::{
    EvaluatorFunction, InterpolationGrid, InterpolationKnots, MultiplySource, function_multiply,
    interpolate, interpolate_rational,
};
pub use iso::{is_rational, iso, reverse};
pub use move_point::{MoveRange, move_point};
pub use ops::{increase_degree, insert_knots, remove_knot, unperiodize};
pub use poles::{get_poles, get_rational_poles, set_poles, set_rational_poles};
pub use prepare::SurfaceSpan;
pub use rational::rational_derivative;
pub use resolution::{ResolutionDirection, resolution};

use crate::bsplclib::{max_unit_weights_size, unit_weights_data};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Failure;

/// The absent weights of non-rational calls (`BSplSLib::NoWeights()`);
/// `bsplslib::no_weights().is_none()`.
pub const fn no_weights<'r, 'a>() -> Option<&'r Array2<'a, f64>> {
    None
}

/// Weights (1..U, 1..V) all equal to 1 (`BSplSLib::UnitWeights`): a non-owning view of
/// the shared BSplCLib buffer up to 2049 poles, an owned grid beyond. Counts whose
/// product is negative, which natively build a view of absurd size, are rejected;
/// `bsplslib::unit_weights(4, 5)?.is_deletable() == false`.
pub fn unit_weights(nb_u_poles: i32, nb_v_poles: i32) -> Result<Array2<'static, f64>, Failure> {
    let total: i64 = i64::from(nb_u_poles) * i64::from(nb_v_poles);
    let invalid = |error: rustycad_tkernel::ncollection::ArrayError| {
        Failure::out_of_range(&format!(
            "BSplSLib::UnitWeights: {nb_u_poles} x {nb_v_poles}: {error}"
        ))
    };
    if total <= i64::from(max_unit_weights_size()) {
        // Non-owning view over the shared array (zero allocation).
        return Array2::from_shared(unit_weights_data(), 1, nb_u_poles, 1, nb_v_poles)
            .map_err(invalid);
    }
    let mut result: Array2<'static, f64> =
        Array2::with_bounds(1, nb_u_poles, 1, nb_v_poles).map_err(invalid)?;
    result.init(1.0);
    Ok(result)
}
