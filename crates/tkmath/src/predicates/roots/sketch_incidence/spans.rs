// SPDX-License-Identifier: GPL-2.0-only
//! Exact homogeneous power-basis pieces of line and B-spline targets.
//!
//! Basis functions follow the Cox–de Boor recurrence with exact rational knots,
//! so each span polynomial is the native curve's piece, not an approximation.
use super::super::certificate_arithmetic::{ExactRatio, RatioInterval};
use super::input::SketchIncidenceSpline;
use super::rational_polynomial::{RationalPolynomial, ratio, zero};

/// Homogeneous point `(x, y) / w` as polynomials of one contact variable.
#[derive(Clone, Debug)]
pub(super) struct HomogeneousPoint {
    pub x: RationalPolynomial,
    pub y: RationalPolynomial,
    pub w: RationalPolynomial,
}
/// Homogeneous piece on a closed range; None ranges over every real value.
#[derive(Clone, Debug)]
pub(super) struct ParametricSpan {
    pub point: HomogeneousPoint,
    pub range: Option<[f64; 2]>,
}
impl HomogeneousPoint {
    /// Exact enclosure of `(x, y) / w` over a closed bracket, if `w` excludes zero there.
    pub fn enclose(&self, parameter: &RatioInterval) -> Option<[RatioInterval; 2]> {
        let reciprocal = self.w.enclose(parameter).reciprocal()?;
        let coordinate =
            |polynomial: &RationalPolynomial| polynomial.enclose(parameter).multiply(&reciprocal);
        Some([coordinate(&self.x), coordinate(&self.y)])
    }
    /// Coordinates relative to `origin`, still homogeneous: `(x - ox w, y - oy w)`.
    pub fn relative(&self, origin: &[ExactRatio; 2]) -> [RationalPolynomial; 2] {
        [
            self.x.add(&self.w.scale(&origin[0].negated())),
            self.y.add(&self.w.scale(&origin[1].negated())),
        ]
    }
}
/// Target line `P0 + u (P1 - P0)`, on `[0,1]` or unbounded.
pub(super) fn line_span(points: [[f64; 2]; 2], bounded: bool) -> ParametricSpan {
    let coordinate = |axis: usize| {
        let start = ratio(points[0][axis]);
        let slope = ratio(points[1][axis]).subtract(&start);
        RationalPolynomial::linear(start, slope)
    };
    ParametricSpan {
        point: HomogeneousPoint {
            x: coordinate(0),
            y: coordinate(1),
            w: RationalPolynomial::constant(ratio(1.)),
        },
        range: bounded.then_some([0., 1.]),
    }
}
/// Every nonempty knot span intersected with the trimmed bounds, in parameter order.
pub(super) fn spline_spans(spline: &SketchIncidenceSpline) -> Vec<ParametricSpan> {
    let knots: Vec<ExactRatio> = spline.flat_knots.iter().copied().map(ratio).collect();
    (spline.degree..spline.poles.len())
        .filter_map(|span| {
            let range = clipped_range(spline, span)?;
            Some(span_piece(spline, &knots, span, range))
        })
        .collect()
}
fn clipped_range(spline: &SketchIncidenceSpline, span: usize) -> Option<[f64; 2]> {
    let lower = spline.flat_knots[span].max(spline.bounds[0]);
    let upper = spline.flat_knots[span + 1].min(spline.bounds[1]);
    (lower < upper).then_some([lower, upper])
}
fn span_piece(
    spline: &SketchIncidenceSpline,
    knots: &[ExactRatio],
    span: usize,
    range: [f64; 2],
) -> ParametricSpan {
    let basis = basis_functions(knots, spline.degree, span);
    let first = span - spline.degree;
    let weight = |index: usize| ratio(spline.weights.as_ref().map_or(1., |weights| weights[index]));
    let combine = |value: &dyn Fn(usize) -> ExactRatio| {
        basis.iter().enumerate().fold(
            RationalPolynomial(Vec::new()),
            |total, (offset, function)| total.add(&function.scale(&value(first + offset))),
        )
    };
    ParametricSpan {
        point: HomogeneousPoint {
            x: combine(&|index| weight(index).multiply(&ratio(spline.poles[index][0]))),
            y: combine(&|index| weight(index).multiply(&ratio(spline.poles[index][1]))),
            w: combine(&weight),
        },
        range: Some(range),
    }
}
/// `N(span - degree + r, degree)` for `r = 0..=degree` as exact polynomials on the span.
fn basis_functions(knots: &[ExactRatio], degree: usize, span: usize) -> Vec<RationalPolynomial> {
    let mut level = vec![RationalPolynomial::constant(ratio(1.))];
    for order in 1..=degree {
        level = (0..=order)
            .map(|offset| next_basis(knots, &level, span + offset - order, order, offset))
            .collect();
    }
    level
}
fn next_basis(
    knots: &[ExactRatio],
    level: &[RationalPolynomial],
    index: usize,
    order: usize,
    offset: usize,
) -> RationalPolynomial {
    let mut total = RationalPolynomial(Vec::new());
    if offset > 0 {
        let rising = ramp(&knots[index], &knots[index + order], false);
        total = total.add(&rising.multiply(&level[offset - 1]));
    }
    if offset < order {
        let falling = ramp(&knots[index + 1], &knots[index + order + 1], true);
        total = total.add(&falling.multiply(&level[offset]));
    }
    total
}
/// `(u - a)/(b - a)`, or `(b - u)/(b - a)` when falling; zero on an empty knot interval.
fn ramp(start: &ExactRatio, end: &ExactRatio, falling: bool) -> RationalPolynomial {
    let width = end.subtract(start);
    if width.sign() == std::cmp::Ordering::Equal {
        return RationalPolynomial(vec![zero()]);
    }
    let slope = ratio(1.).divide(&width);
    if falling {
        return RationalPolynomial::linear(end.multiply(&slope), slope.negated());
    }
    RationalPolynomial::linear(start.multiply(&slope).negated(), slope)
}
