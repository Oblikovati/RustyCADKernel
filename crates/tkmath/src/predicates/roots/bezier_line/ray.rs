// SPDX-License-Identifier: GPL-2.0-only
mod error;
mod prepare;
use super::super::{
    DyadicBound, RootIsolationError, RootIsolationOptions, bernstein,
    polynomial::IntegerPolynomial,
    signs::{self, RootSignEvaluation},
};
use super::{BezierLineError, BezierLineQuery, BezierStage, solve};
pub use error::BezierRayError;
use num_bigint::BigInt;
pub(super) use prepare::{
    RegularProjection, regular_projection, regular_projection_accounted,
    regular_projection_captured,
};

/// Classify each isolated supporting-line contact relative to the ray from A towards B.
///
/// The query's line endpoints are A and B. Returned signs mean behind (Less),
/// at the origin (Equal), or ahead (Greater). Behind contacts are retained.
/// Supporting-line coincidence declines instead of returning a partial overlap.
/// ```
/// use rustycad_tkmath::predicates::roots::{RootIsolationOptions, bezier_line::*};
/// let query = BezierLineQuery { controls: &[[-1., -1.], [-1., 1.]], weights: None,
///     line: [[0., 0.], [1., 0.]], interval: [0., 1.],
///     options: RootIsolationOptions::new(0.001) };
/// let contacts = classify_bezier_ray_contacts(query, || false)?;
/// assert_eq!(contacts.signs(), &[std::cmp::Ordering::Less]);
/// # Ok::<(), BezierRayError>(())
/// ```
pub fn classify_bezier_ray_contacts(
    query: BezierLineQuery<'_>,
    mut cancelled: impl FnMut() -> bool,
) -> Result<RootSignEvaluation, BezierRayError> {
    let (bounds, width) = super::checked_query(query).map_err(BezierRayError::Line)?;
    let prepared = prepare(
        query,
        bounds.clone(),
        width.clone(),
        query.options,
        &mut cancelled,
    )?;
    classify(prepared, bounds, width, query.options, &mut cancelled)
}
struct PreparedRay {
    normal: IntegerPolynomial,
    directed: IntegerPolynomial,
    denominator_work: u16,
}
fn prepare(
    query: BezierLineQuery<'_>,
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<PreparedRay, BezierRayError> {
    let projected: RegularProjection = regular_projection(query, bounds, width, cancelled)?;
    let normal = bernstein::from_integer_controls(projected.normal, options.max_degree, cancelled)
        .map_err(normal_error)?;
    let directed = directed_product(
        projected.longitudinal,
        projected.denominator,
        options.max_degree,
        cancelled,
    )?;
    Ok(PreparedRay {
        normal,
        directed,
        denominator_work: projected.work,
    })
}
pub(super) fn directed_product(
    controls: Vec<BigInt>,
    denominator: IntegerPolynomial,
    max_degree: u16,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<IntegerPolynomial, BezierRayError> {
    let longitudinal = IntegerPolynomial::accept_zero(bernstein::from_integer_controls(
        controls, max_degree, cancelled,
    ))
    .map_err(BezierRayError::Direction)?;
    longitudinal
        .multiply(&denominator, cancelled)
        .map_err(BezierRayError::Direction)
}

fn classify(
    prepared: PreparedRay,
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    mut options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<RootSignEvaluation, BezierRayError> {
    options.max_bisections -= prepared.denominator_work;
    let mut isolation =
        solve::solve_polynomial(prepared.normal.clone(), bounds, width, options, cancelled)
            .map_err(normal_error)?;
    isolation.bisections += prepared.denominator_work;
    signs::evaluate_isolated(prepared.normal, prepared.directed, isolation, cancelled)
        .map_err(BezierRayError::Signs)
}
pub(super) fn normal_error(cause: RootIsolationError) -> BezierRayError {
    match cause {
        RootIsolationError::ZeroPolynomial { .. } => BezierRayError::SupportingLineCoincidence,
        cause => BezierRayError::Line(BezierLineError::Root {
            stage: BezierStage::Numerator,
            cause,
        }),
    }
}
