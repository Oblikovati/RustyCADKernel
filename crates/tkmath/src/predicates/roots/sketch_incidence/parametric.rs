// SPDX-License-Identifier: GPL-2.0-only
//! Contacts of one parametric span: exact roots in the target parameter `u`,
//! each paired with the unique root of `R(T)` that is its source line parameter.
use super::super::certificate_arithmetic::{ExactRatio, RatioInterval, rounding};
use super::super::{DyadicBound, RootBracket, RootIsolation, polynomial_solve};
use super::algebraic::{IsolatedRoot, RootEquation};
use super::budget::Work;
use super::construction::WorldLimit;
use super::equation::{cauchy_bounds, isolate, side_signs};
use super::failure::{SketchIncidenceCause as Cause, SketchIncidenceStage};
use super::rational_polynomial::RationalPolynomial;
use super::resultant::line_parameter_equation;
use super::source_line::SourceLine;
use super::spans::ParametricSpan;
use num_bigint::Sign;
use std::sync::Arc;

/// `N = det(d, C - A) W`-numerator, `L = d . (C - A)`-numerator and the weight `W`.
pub(super) struct SpanEquations {
    pub x: RationalPolynomial,
    pub y: RationalPolynomial,
    pub normal: RationalPolynomial,
    pub along: RationalPolynomial,
    pub weight: RationalPolynomial,
}
impl SpanEquations {
    pub fn new(span: &ParametricSpan, source: &SourceLine) -> Self {
        let relative = |coordinate: &RationalPolynomial, axis: usize| {
            coordinate.add(&span.w.scale(&source.origin[axis].negated()))
        };
        let x = relative(&span.x, 0);
        let y = relative(&span.y, 1);
        let [dx, dy] = &source.direction;
        Self {
            x: span.x.clone(),
            y: span.y.clone(),
            normal: y.scale(dx).add(&x.scale(&dy.negated())),
            along: x.scale(dx).add(&y.scale(dy)),
            weight: span.w.clone(),
        }
    }
    /// Exact line parameter `L(u) / (m W(u))` at a rational target parameter.
    pub fn line_parameter_at(&self, parameter: &ExactRatio, source: &SourceLine) -> ExactRatio {
        let denominator = self
            .weight
            .evaluate(parameter)
            .multiply(&source.squared_length);
        self.along.evaluate(parameter).divide(&denominator)
    }
}
/// One certified root of a span, before merging contacts across shared knots.
pub(super) struct SpanRoot {
    pub target: IsolatedRoot,
    pub line: IsolatedRoot,
    pub sides: [Option<Sign>; 2],
    pub multiplicity: [Option<usize>; 2],
}
/// Isolated roots of a span whose normal equation is not identically zero.
pub(super) fn span_roots(
    span: &ParametricSpan,
    equations: &SpanEquations,
    source: &SourceLine,
    work: &mut Work<'_>,
) -> Result<Option<Vec<SpanRoot>>, Cause> {
    let Some(normal) = equations.normal.to_integer(work.max_degree())? else {
        return Ok(None);
    };
    let bounds = match span.range {
        Some(range) => range.map(dyadic),
        None => cauchy_bounds(&normal),
    };
    let (target_equation, isolation) = isolate(&normal, bounds, work)?;
    if isolation.roots().is_empty() {
        return Ok(Some(Vec::new()));
    }
    let line_equation = line_equation(equations, source, work)?;
    let context = RootContext {
        target_equation: &target_equation,
        line_equation: &line_equation,
        equations,
        source,
        range: span.range,
    };
    let roots = isolation
        .roots()
        .iter()
        .map(|bracket| context.span_root(bracket, work))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(roots))
}
pub(super) fn dyadic(value: f64) -> DyadicBound {
    DyadicBound::from_f64(value).expect("admitted finite parameter")
}
fn line_equation(
    equations: &SpanEquations,
    source: &SourceLine,
    work: &mut Work<'_>,
) -> Result<Arc<RootEquation>, Cause> {
    work.enter(SketchIncidenceStage::Equation);
    work.check()?;
    let resultant = line_parameter_equation(
        &equations.normal,
        &equations.along,
        &equations.weight,
        &source.squared_length,
    )
    .ok_or(Cause::DegenerateResultant)?;
    let polynomial = resultant
        .to_integer(work.max_degree())?
        .ok_or(Cause::DegenerateResultant)?;
    RootEquation::new(polynomial, work)
}
struct RootContext<'a> {
    target_equation: &'a Arc<RootEquation>,
    line_equation: &'a Arc<RootEquation>,
    equations: &'a SpanEquations,
    source: &'a SourceLine,
    range: Option<[f64; 2]>,
}
impl RootContext<'_> {
    fn span_root(&self, bracket: &RootBracket, work: &mut Work<'_>) -> Result<SpanRoot, Cause> {
        let mut target = IsolatedRoot::from_bracket(self.target_equation, bracket);
        let [left, right] = side_signs(&self.target_equation.polynomial, bracket);
        let ends = self.range.map(|range| range.map(dyadic));
        let at = |index: usize| {
            ends.as_ref()
                .is_some_and(|ends| target.exact() == Some(&ends[index]))
        };
        let open = [!at(0), !at(1)];
        work.enter(SketchIncidenceStage::Classification);
        let line = self.line_root(&mut target, work)?;
        work.enter(SketchIncidenceStage::Construction);
        let limit = WorldLimit::new(self.source.requested());
        limit.construct(&mut target, work, &|root: &IsolatedRoot| {
            self.target_enclosure(root)
        })?;
        Ok(SpanRoot {
            target,
            line,
            sides: [open[0].then_some(left), open[1].then_some(right)],
            multiplicity: open.map(|side| side.then_some(bracket.multiplicity())),
        })
    }
    /// Narrow `u` until exactly one root of `R` lies in the enclosure of `L/(mW)`.
    fn line_root(
        &self,
        target: &mut IsolatedRoot,
        work: &mut Work<'_>,
    ) -> Result<IsolatedRoot, Cause> {
        loop {
            work.check()?;
            if let Some(bounds) = self.parameter_enclosure(target) {
                let candidates = self.isolate_line(bounds, work)?;
                if let Some(root) = self.unique_candidate(&candidates, target, work)? {
                    return Ok(root);
                }
            } else if target.exact().is_some() {
                return Err(Cause::ConstructionResolution {
                    requested: self.source.requested(),
                    achieved: None,
                });
            }
            target.bisect(work)?;
        }
    }
    fn unique_candidate(
        &self,
        candidates: &RootIsolation,
        target: &IsolatedRoot,
        work: &mut Work<'_>,
    ) -> Result<Option<IsolatedRoot>, Cause> {
        let roots = candidates.roots();
        if roots.len() == 1 {
            return Ok(Some(IsolatedRoot::from_bracket(
                self.line_equation,
                &roots[0],
            )));
        }
        if roots.is_empty() {
            return Err(Cause::Invariant(
                "line parameter enclosure missed its resultant root",
            ));
        }
        match target.exact() {
            Some(exact) => self.exact_candidate(roots, exact, work).map(Some),
            None => Ok(None),
        }
    }
    // An exact target parameter has an exact rational line parameter; select its root.
    fn exact_candidate(
        &self,
        roots: &[RootBracket],
        exact: &DyadicBound,
        work: &mut Work<'_>,
    ) -> Result<IsolatedRoot, Cause> {
        let value = self
            .equations
            .line_parameter_at(&ExactRatio::from_dyadic(exact), self.source);
        for bracket in roots {
            let mut candidate = IsolatedRoot::from_bracket(self.line_equation, bracket);
            if candidate.compare_ratio(&value, work)?.is_eq() {
                return Ok(candidate);
            }
        }
        Err(Cause::Invariant(
            "exact line parameter is not a resultant root",
        ))
    }
    fn isolate_line(
        &self,
        bounds: [DyadicBound; 2],
        work: &mut Work<'_>,
    ) -> Result<RootIsolation, Cause> {
        let width = bounds[1].difference(&bounds[0]);
        let options = work.isolation();
        let polynomial = self.line_equation.polynomial.clone();
        Ok(polynomial_solve::solve_polynomial(
            polynomial,
            bounds,
            width,
            options,
            &mut || work.poll(),
        )?)
    }
    /// Exact enclosure of the target point `(x, y) / w` over the target bracket.
    fn target_enclosure(&self, target: &IsolatedRoot) -> Option<[RatioInterval; 2]> {
        let [lower, upper] = target.ratio_bounds();
        let parameter = RatioInterval { lower, upper };
        let reciprocal = self.equations.weight.enclose(&parameter).reciprocal()?;
        let coordinate =
            |polynomial: &RationalPolynomial| polynomial.enclose(&parameter).multiply(&reciprocal);
        Some([coordinate(&self.equations.x), coordinate(&self.equations.y)])
    }
    /// Outward dyadic enclosure of `L(u)/(m W(u))` over the target bracket, if resolvable.
    fn parameter_enclosure(&self, target: &IsolatedRoot) -> Option<[DyadicBound; 2]> {
        let [lower, upper] = target.ratio_bounds();
        let parameter = RatioInterval { lower, upper };
        let squared = RatioInterval::point(self.source.squared_length.clone());
        let denominator = self.equations.weight.enclose(&parameter).multiply(&squared);
        let quotient = self
            .equations
            .along
            .enclose(&parameter)
            .multiply(&denominator.reciprocal()?);
        let lower = rounding::outward_binary64(&quotient.lower)?[0];
        let upper = rounding::outward_binary64(&quotient.upper)?[1];
        Some([dyadic(lower), dyadic(upper)])
    }
}
