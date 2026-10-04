// Rust translation of FoundationClasses/TKMath/BSplCLib (BSplCLib.hxx/.lxx/.cxx, BSplCLib_1/2/3.cxx, BSplCLib_BzSyntaxes.cxx, BSplCLib_CurveComputation.pxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! BSplCLib: the B-spline curve library under Geom, Geom2d, Convert and the
//! approximation packages.
//!
//! Three families of functions are ported, with the native operation order:
//! knot and multiplicity management; multi-dimensional schemes on poles stored as
//! interleaved reals (`_flat` suffix on array overloads, slices for `double&`
//! buffers); and 2D/3D curve functions, one generic function over [`CurvePoint`]
//! (gp_Pnt/gp_Pnt2d) for each pair of native overloads. Real-array curve
//! overloads carry a `_scalar` suffix.
//!
//! `BSplCLib::NoWeights()` and `BSplCLib::NoMults()` become `None`. Native
//! exceptions become `Failure` values of the same type. Where OCCT would read or
//! write outside a buffer or array through a raw pointer (undefined behaviour),
//! the port returns a Standard_OutOfRange `Failure` naming the offending index.
#![allow(clippy::too_many_arguments)] // Native entry points take up to fourteen arguments; order is kept.

mod banded;
mod basis;
mod bezier;
mod bohm;
mod build;
mod build_cache;
mod cache;
mod cache_eval;
mod cache_params;
mod curve_eval;
mod curve_ops;
mod degree;
mod extrapolate;
mod forms;
mod function;
mod insert;
mod interpolate;
mod intervals;
mod knots;
mod locate;
mod merge;
mod move_point;
mod move_tangent;
pub(crate) mod native;
mod periodic;
mod point;
mod prepare_insert;
mod remove;
mod resolution;
mod scheme;
mod shared_cache;
mod tang_extend;
mod views;

pub use banded::{
    build_bsp_matrix, factor_banded_matrix, solve_banded_system, solve_banded_system_points,
    solve_banded_system_points_rational, solve_banded_system_rational,
};
pub use basis::eval_bspline_basis;
pub use bezier::{bezier_d0, bezier_d1, bezier_d2, bezier_d3, bezier_knots};
pub use bohm::bohm;
pub use build::{boor_index, build_boor, build_eval, build_eval_scalar, build_knots, get_pole};
pub use build_cache::{
    build_cache, build_cache_matrix, flat_bezier_knots, max_unit_weights_size, poles_coefficients,
    unit_weights, unit_weights_data,
};
pub use cache::Cache;
pub use cache_eval::{
    CacheSpan, cache_d0, cache_d1, cache_d2, cache_d3, coefs_d0, coefs_d1, coefs_d2, coefs_d3,
};
pub use cache_params::CacheParams;
pub use curve_eval::{
    CurveSpan, d0, d0_scalar, d1, d1_scalar, d2, d2_scalar, d3, d3_scalar, dn, dn_scalar,
};
pub use curve_ops::{
    increase_degree, increase_degree_bezier, insert_knot, insert_knots, raise_multiplicity,
    remove_knot, trimming, unperiodize,
};
pub use degree::{increase_degree_count_knots, increase_degree_flat};
pub use extrapolate::{eval_extrapolated, eval_extrapolated_point, eval_extrapolated_rational};
pub use forms::{
    is_rational, knot_analysis, knot_form, mult_form, reparametrize, reverse_knots, reverse_mults,
    reverse_poles,
};
pub use function::{
    EvaluatorFunction, FunctionTarget, function_multiply, function_multiply_flat,
    function_multiply_scalar, function_reparameterise, function_reparameterise_flat,
    function_reparameterise_scalar,
};
pub use insert::insert_knots_flat;
pub use interpolate::{
    build_schoenberg_points, interpolate, interpolate_flat, interpolate_flat_rational,
    interpolate_rational,
};
pub use intervals::intervals;
pub use knots::{
    first_u_knot_index, flat_index, hunt, knot_sequence, knot_sequence_length,
    knot_sequence_with_degree, knots, knots_length, last_u_knot_index, max_knot_mult,
    min_knot_mult, nb_poles, pole_index,
};
pub use locate::{locate_parameter, locate_parameter_flat_knots, locate_parameter_for_eval};
pub use merge::merge_bspline_knots;
pub use move_point::move_point;
pub use move_tangent::{move_point_and_tangent, move_point_and_tangent_flat};
pub use periodic::{prepare_trimming, prepare_unperiodize, trimming_flat, unperiodize_flat};
pub use point::{CurveD3, CurvePoint};
pub use prepare_insert::prepare_insert_knots;
pub use remove::remove_knot_flat;
pub use resolution::{resolution, resolution_flat};
pub use scheme::{anti_boor_scheme, boor_scheme, derivative, eval};
pub use shared_cache::{SharedBSplineCache, SharedCache};
pub use tang_extend::tang_extend_to_constraint;

use crate::plib::PolePoint;
use rustycad_tkernel::ncollection::Array1;

/// Greatest supported degree (`BSplCLib::MaxDegree()`); `bsplclib::max_degree() == 25`.
pub const fn max_degree() -> i32 {
    MAX_DEGREE
}

pub(crate) const MAX_DEGREE: i32 = 25;

/// Uniformity of a knot sequence (`BSplCLib_KnotDistribution`);
/// `KnotDistribution::Uniform as i32 == 1`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum KnotDistribution {
    /// Knots are not equally spaced.
    NonUniform,
    /// Knots are equally spaced.
    Uniform,
}

/// Pattern of a multiplicity sequence (`BSplCLib_MultDistribution`);
/// `MultDistribution::QuasiConstant as i32 == 2`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum MultDistribution {
    /// Multiplicities vary.
    NonConstant,
    /// All multiplicities are equal.
    Constant,
    /// Equal except the first and last, which are equal to each other.
    QuasiConstant,
}

/// The absent weights of non-rational calls (`BSplCLib::NoWeights()`);
/// `bsplclib::no_weights().is_none()`.
pub const fn no_weights<'r, 'a>() -> Option<&'r Array1<'a, f64>> {
    None
}

/// The absent multiplicities of flat-knot calls (`BSplCLib::NoMults()`);
/// `bsplclib::no_mults().is_none()`.
pub const fn no_mults<'r, 'a>() -> Option<&'r Array1<'a, i32>> {
    None
}

// The coordinates of 2D/3D points as the contiguous reals the native code addresses
// through `(double*)&Poles(Poles.Lower())`.
pub(crate) fn flat_points<P: PolePoint>(points: &Array1<'_, P>) -> Vec<f64> {
    let mut flat: Vec<f64> = Vec::with_capacity(points.len() * P::DIMENSION);
    for point in points.as_slice() {
        for k in 0..P::DIMENSION {
            flat.push(point.pole_coordinate(k));
        }
    }
    flat
}

// Writes contiguous reals back into 2D/3D points.
pub(crate) fn store_flat_points<P: PolePoint>(flat: &[f64], points: &mut Array1<'_, P>) {
    for (i, point) in points.as_mut_slice().iter_mut().enumerate() {
        for k in 0..P::DIMENSION {
            point.set_pole_coordinate(k, flat[i * P::DIMENSION + k]);
        }
    }
}
