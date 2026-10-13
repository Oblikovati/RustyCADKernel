// SPDX-License-Identifier: GPL-2.0-only
mod local;
use super::super::{
    DyadicBound, RootIsolation, RootIsolationOptions, bernstein, polynomial::IntegerPolynomial,
    signs,
};
use super::{BezierLineQuery, BezierRayError, ray, solve};
use std::cmp::Ordering;
/// Exact incidence or signed crossing contribution of one increasing-parameter segment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BezierWinding {
    /// The supplied curve contains the ray origin in the closed query interval.
    Boundary,
    /// Complete contribution; closed-wire assembly must still prove endpoint closure.
    Clear(i32),
}
/// Count oriented positive-ray crossings with exact half-open endpoint ownership.
///
/// This is a segment contribution, not proof of a closed or valid wire.
/// ```
/// use rustycad_tkmath::predicates::roots::{RootIsolationOptions, bezier_line::*};
/// let query = BezierLineQuery { controls: &[[1., -1.], [1., 1.]], weights: None,
///     line: [[0., 0.], [1., 0.]], interval: [0., 1.],
///     options: RootIsolationOptions::new(0.001) };
/// assert_eq!(bezier_winding_contribution(query, || false)?, BezierWinding::Clear(1));
/// # Ok::<(), BezierRayError>(())
/// ```
pub fn bezier_winding_contribution(
    query: BezierLineQuery<'_>,
    mut cancelled: impl FnMut() -> bool,
) -> Result<BezierWinding, BezierRayError> {
    Ok(WindingQuery::prepare(query, &mut cancelled)?
        .evaluate(&mut cancelled)?
        .incidence)
}

pub(super) struct WindingEvaluation {
    pub incidence: BezierWinding,
    pub bisections: u16,
}
pub(super) struct WindingQuery {
    normal: IntegerPolynomial,
    directed: IntegerPolynomial,
    denominator: IntegerPolynomial,
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    options: RootIsolationOptions,
    denominator_work: u16,
}
impl WindingQuery {
    fn prepare(
        query: BezierLineQuery<'_>,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Self, BezierRayError> {
        Self::prepare_captured(query, cancelled, |_, _, _, _| Ok(())).map(|(prepared, ())| prepared)
    }
    pub(super) fn prepare_captured<T, F: FnMut() -> bool>(
        query: BezierLineQuery<'_>,
        cancelled: &mut F,
        capture: impl FnOnce(
            &super::projection::Projection,
            &[num_bigint::BigInt],
            &[DyadicBound; 2],
            &mut F,
        ) -> Result<T, BezierRayError>,
    ) -> Result<(Self, T), BezierRayError> {
        let (bounds, width) = super::checked_query(query).map_err(BezierRayError::Line)?;
        let (projected, captured) = ray::regular_projection_captured(
            query,
            bounds.clone(),
            width.clone(),
            cancelled,
            capture,
        )?;
        let mut options = query.options;
        options.max_bisections -= projected.work;
        Ok((
            Self::from_projection(projected, bounds, width, options, cancelled)?,
            captured,
        ))
    }
    fn from_projection(
        projected: ray::RegularProjection,
        bounds: [DyadicBound; 2],
        width: DyadicBound,
        options: RootIsolationOptions,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Self, BezierRayError> {
        let normal = normal_polynomial(projected.normal, options.max_degree, cancelled)?;
        let directed = ray::directed_product(
            projected.longitudinal,
            projected.denominator.clone(),
            options.max_degree,
            cancelled,
        )?;
        Ok(Self {
            normal,
            directed,
            denominator: projected.denominator,
            bounds,
            width,
            options,
            denominator_work: projected.work,
        })
    }
    pub(super) fn evaluate(
        self,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<WindingEvaluation, BezierRayError> {
        let denominator_work = self.denominator_work;
        let mut result = self.evaluate_numerator(cancelled)?;
        result.bisections += denominator_work;
        Ok(result)
    }
    fn evaluate_numerator(
        self,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<WindingEvaluation, BezierRayError> {
        if self.normal.0.is_empty() {
            return self.coincident(cancelled);
        }
        let evaluated = self.isolated(cancelled)?;
        let incidence = if evaluated.signs().contains(&Ordering::Equal) {
            BezierWinding::Boundary
        } else {
            self.sum(evaluated.isolation(), evaluated.signs(), cancelled)?
        };
        Ok(WindingEvaluation::new(
            incidence,
            evaluated.isolation().bisections(),
        ))
    }
    fn isolated(
        &self,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<signs::RootSignEvaluation, BezierRayError> {
        let roots = solve::solve_polynomial(
            self.normal.clone(),
            self.bounds.clone(),
            self.width.clone(),
            self.options,
            cancelled,
        )
        .map_err(ray::normal_error)?;
        signs::evaluate_isolated(self.normal.clone(), self.directed.clone(), roots, cancelled)
            .map_err(BezierRayError::Signs)
    }
    fn coincident(
        self,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<WindingEvaluation, BezierRayError> {
        if self.directed.0.is_empty() {
            return Ok(WindingEvaluation::new(BezierWinding::Boundary, 0));
        }
        let roots = solve::solve_polynomial(
            self.directed,
            self.bounds,
            self.width,
            self.options,
            cancelled,
        )
        .map_err(BezierRayError::Direction)?;
        let incidence = if roots.roots().is_empty() {
            BezierWinding::Clear(0)
        } else {
            BezierWinding::Boundary
        };
        Ok(WindingEvaluation::new(incidence, roots.bisections()))
    }
    fn sum(
        &self,
        roots: &RootIsolation,
        sides: &[Ordering],
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<BezierWinding, BezierRayError> {
        // ADR 0009: at most u16::MAX admitted-degree unit crossings fit i32.
        let mut count = 0;
        for (root, side) in roots.roots().iter().zip(sides) {
            super::super::error::poll(cancelled).map_err(ray::normal_error)?;
            if *side == Ordering::Greater && self.bounds[0] != self.bounds[1] {
                count += local::contribution(
                    &self.normal,
                    &self.denominator,
                    root,
                    &self.bounds,
                    cancelled,
                )?;
            }
        }
        Ok(BezierWinding::Clear(count))
    }
}

fn normal_polynomial(
    controls: Vec<num_bigint::BigInt>,
    max_degree: u16,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<IntegerPolynomial, BezierRayError> {
    IntegerPolynomial::accept_zero(bernstein::from_integer_controls(
        controls, max_degree, cancelled,
    ))
    .map_err(ray::normal_error)
}

impl WindingEvaluation {
    fn new(incidence: BezierWinding, bisections: u16) -> Self {
        Self {
            incidence,
            bisections,
        }
    }
}
