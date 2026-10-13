// SPDX-License-Identifier: GPL-2.0-only
mod evidence;
mod local;
use super::super::{
    DyadicBound, RootIsolation, RootIsolationOptions, bernstein, polynomial::IntegerPolynomial,
    signs,
};
use super::{BezierLineQuery, BezierRayError, ray, solve};
use CapturedWindingEvidence as WindingEvidence;
pub use evidence::CapturedWindingEvidence;
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

#[derive(Clone)]
pub(super) struct WindingEvaluation {
    pub incidence: BezierWinding,
    pub bisections: u16,
    pub evidence: WindingEvidence,
    pub denominator: RootIsolation,
}
pub(super) struct WindingQuery {
    normal: IntegerPolynomial,
    directed: IntegerPolynomial,
    denominator: IntegerPolynomial,
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    options: RootIsolationOptions,
    denominator_isolation: RootIsolation,
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
        Self::prepare_accounted(query, cancelled, capture, &mut || {})
    }
    pub(super) fn prepare_accounted<T, F: FnMut() -> bool>(
        query: BezierLineQuery<'_>,
        cancelled: &mut F,
        capture: impl FnOnce(
            &super::projection::Projection,
            &[num_bigint::BigInt],
            &[DyadicBound; 2],
            &mut F,
        ) -> Result<T, BezierRayError>,
        completed_split: &mut impl FnMut(),
    ) -> Result<(Self, T), BezierRayError> {
        let (bounds, width) = super::checked_query(query).map_err(BezierRayError::Line)?;
        let (projected, captured) = ray::regular_projection_accounted(
            query,
            bounds.clone(),
            width.clone(),
            cancelled,
            capture,
            completed_split,
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
            denominator_isolation: projected.denominator_isolation,
        })
    }
    pub(super) fn evaluate(
        self,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<WindingEvaluation, BezierRayError> {
        self.evaluate_accounted(cancelled, &mut || {})
    }
    pub(super) fn evaluate_accounted(
        self,
        cancelled: &mut impl FnMut() -> bool,
        completed_split: &mut impl FnMut(),
    ) -> Result<WindingEvaluation, BezierRayError> {
        let denominator = self.denominator_isolation.clone();
        let (incidence, evidence, work) = self.evaluate_numerator(cancelled, completed_split)?;
        Ok(WindingEvaluation {
            incidence,
            bisections: work + denominator.bisections(),
            evidence,
            denominator,
        })
    }
    fn evaluate_numerator(
        self,
        cancelled: &mut impl FnMut() -> bool,
        completed_split: &mut impl FnMut(),
    ) -> Result<(BezierWinding, WindingEvidence, u16), BezierRayError> {
        if self.normal.0.is_empty() {
            return self.coincident(cancelled, completed_split);
        }
        let evaluated = self.isolated(cancelled, completed_split)?;
        let (incidence, contributions) = if evaluated.signs().contains(&Ordering::Equal) {
            (
                BezierWinding::Boundary,
                vec![None; evaluated.isolation().roots().len()],
            )
        } else {
            self.sum(evaluated.isolation(), evaluated.signs(), cancelled)?
        };
        Ok(evidence::normal_result(incidence, evaluated, contributions))
    }

    fn isolated(
        &self,
        cancelled: &mut impl FnMut() -> bool,
        completed_split: &mut impl FnMut(),
    ) -> Result<signs::RootSignEvaluation, BezierRayError> {
        let roots = solve::solve_polynomial_accounted(
            self.normal.clone(),
            self.bounds.clone(),
            self.width.clone(),
            self.options,
            cancelled,
            completed_split,
        )
        .map_err(ray::normal_error)?;
        signs::evaluate_isolated(self.normal.clone(), self.directed.clone(), roots, cancelled)
            .map_err(BezierRayError::Signs)
    }
    fn coincident(
        self,
        cancelled: &mut impl FnMut() -> bool,
        completed_split: &mut impl FnMut(),
    ) -> Result<(BezierWinding, WindingEvidence, u16), BezierRayError> {
        if self.directed.0.is_empty() {
            return Ok((BezierWinding::Boundary, WindingEvidence::ZeroDirected, 0));
        }
        let roots = solve::solve_polynomial_accounted(
            self.directed,
            self.bounds,
            self.width,
            self.options,
            cancelled,
            completed_split,
        )
        .map_err(BezierRayError::Direction)?;
        Ok(evidence::coincident_result(roots))
    }

    fn sum(
        &self,
        roots: &RootIsolation,
        sides: &[Ordering],
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(BezierWinding, Vec<Option<i32>>), BezierRayError> {
        // ADR 0009: at most u16::MAX admitted-degree unit crossings fit i32.
        let mut count = 0;
        let mut contributions = Vec::with_capacity(roots.roots().len());
        for (root, side) in roots.roots().iter().zip(sides) {
            super::super::error::poll(cancelled).map_err(ray::normal_error)?;
            let contribution = self.local_contribution(root, *side, cancelled)?;
            count += contribution.unwrap_or(0);
            contributions.push(contribution);
        }
        Ok((BezierWinding::Clear(count), contributions))
    }
    fn local_contribution(
        &self,
        root: &super::super::RootBracket,
        side: Ordering,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Option<i32>, BezierRayError> {
        if side != Ordering::Greater || self.bounds[0] == self.bounds[1] {
            return Ok(None);
        }
        local::contribution(
            &self.normal,
            &self.denominator,
            root,
            &self.bounds,
            cancelled,
        )
        .map(Some)
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
