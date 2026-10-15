// SPDX-License-Identifier: GPL-2.0-only
//! Internal invariants checked against independent exact constructions.
use super::super::certificate_arithmetic::ExactRatio;
use super::super::{DyadicBound, polynomial::IntegerPolynomial};
use super::algebraic::{IsolatedRoot, RootEquation, compare_roots, sign_at_ratio};
use super::budget::Work;
use super::equation::{cauchy_bounds, isolate};
use super::input::{SketchIncidenceOptions, SketchIncidenceSpline};
use super::rational_polynomial::{RationalPolynomial, ratio};
use super::resultant::image_equation;
use super::spans::spline_spans;
use num_bigint::{BigInt, Sign};
use std::cmp::Ordering;

fn integers(coefficients: &[i64]) -> IntegerPolynomial {
    IntegerPolynomial(
        coefficients
            .iter()
            .map(|value| BigInt::from(*value))
            .collect(),
    )
}
fn polynomial(coefficients: &[f64]) -> RationalPolynomial {
    RationalPolynomial(coefficients.iter().copied().map(ratio).collect())
}
fn isolated(coefficients: &[i64], work: &mut Work<'_>) -> Vec<IsolatedRoot> {
    let polynomial = integers(coefficients);
    let (equation, isolation) = isolate(&polynomial, cauchy_bounds(&polynomial), work).unwrap();
    isolation
        .roots()
        .iter()
        .map(|bracket| IsolatedRoot::from_bracket(&equation, bracket))
        .collect()
}
#[test]
fn resultant_vanishes_exactly_at_the_line_parameters_of_each_root() {
    // Parabola (u, u^2 - 1) against the line y = 0 through (0,0)-(2,0):
    // N = 2(u^2 - 1), L = 2u, m = 4, so t = u/2 at u = -1, 1.
    let normal = polynomial(&[-2., 0., 2.]);
    let along = polynomial(&[0., 2.]);
    let weight = polynomial(&[1.]);
    let equation = image_equation(&normal, &along, &weight.scale(&ratio(4.))).unwrap();
    assert_eq!(equation.degree(), Some(2));
    for root in [-0.5, 0.5] {
        assert_eq!(equation.evaluate(&ratio(root)), ratio(0.));
    }
    assert_ne!(equation.evaluate(&ratio(0.)), ratio(0.));
}
#[test]
fn constant_longitudinal_projection_gives_its_exact_parameter() {
    // A vertical piece x = 1 seen from the line x-axis through (0,0)-(4,0): t = 1/4.
    let equation = image_equation(
        &polynomial(&[1., 3.]),
        &polynomial(&[4.]),
        &polynomial(&[16.]),
    )
    .unwrap();
    assert_eq!(equation.evaluate(&ratio(0.25)), ratio(0.));
}
// Independent exact de Boor evaluation of the curve point at `u`.
fn de_boor(spline: &SketchIncidenceSpline, span: usize, u: &ExactRatio) -> [ExactRatio; 2] {
    let knots: Vec<ExactRatio> = spline.flat_knots.iter().copied().map(ratio).collect();
    let degree = spline.degree;
    let mut points: Vec<[ExactRatio; 2]> = (0..=degree)
        .map(|offset| spline.poles[span - degree + offset].map(ratio))
        .collect();
    for level in 1..=degree {
        for offset in (level..=degree).rev() {
            let index = span - degree + offset;
            let width = knots[index + degree + 1 - level].subtract(&knots[index]);
            let alpha = u.subtract(&knots[index]).divide(&width);
            let complement = ratio(1.).subtract(&alpha);
            points[offset] = std::array::from_fn(|axis| {
                complement
                    .multiply(&points[offset - 1][axis])
                    .add(&alpha.multiply(&points[offset][axis]))
            });
        }
    }
    points[degree].clone()
}
#[test]
fn cox_de_boor_span_pieces_match_independent_de_boor_points() {
    let spline = SketchIncidenceSpline {
        degree: 3,
        poles: vec![
            [0., 0.],
            [1., 2.],
            [2.5, -1.],
            [4., 3.],
            [5., 0.25],
            [6., 1.],
        ],
        weights: None,
        flat_knots: vec![0., 0., 0., 0., 0.75, 2., 3., 3., 3., 3.],
        bounds: [0., 3.],
    };
    let spans = spline_spans(&spline);
    assert_eq!(spans.len(), 3);
    for (piece, span) in spans.iter().zip(3..6) {
        let [lower, upper] = piece.range.unwrap();
        for step in 0..=4 {
            let u = ratio(lower)
                .add(&ratio(upper - lower).multiply(&ExactRatio::new(step.into(), 4.into())));
            assert_eq!(piece.point.w.evaluate(&u), ratio(1.));
            let expected = de_boor(&spline, span, &u);
            assert_eq!(
                [piece.point.x.evaluate(&u), piece.point.y.evaluate(&u)],
                expected,
                "span {span} step {step}"
            );
        }
    }
}
#[test]
fn cauchy_bounds_strictly_contain_large_and_tiny_roots() {
    let mut cancelled = || false;
    let mut work = Work::new(SketchIncidenceOptions::v1(1e-9), &mut cancelled);
    let roots = isolated(&[-1_000_000, 0, 1], &mut work);
    assert_eq!(roots.len(), 2);
    let bounds = cauchy_bounds(&integers(&[-1_000_000, 0, 1]));
    assert!(bounds[1] > DyadicBound::from_f64(1000.).unwrap());
    assert!(bounds[0] < DyadicBound::from_f64(-1000.).unwrap());
    assert_eq!(isolated(&[1, 0, 1_000_000], &mut work).len(), 0);
}
#[test]
fn equal_roots_of_different_equations_are_proved_by_their_common_factor() {
    let mut cancelled = || false;
    let mut work = Work::new(SketchIncidenceOptions::v1(1e-9), &mut cancelled);
    let mut square_two = isolated(&[-2, 0, 1], &mut work);
    // (x^2 - 2)(x - 3) = x^3 - 3x^2 - 2x + 6.
    let mut product = isolated(&[6, -2, -3, 1], &mut work);
    assert_eq!(product.len(), 3);
    assert_eq!(
        compare_roots(&mut square_two[1], &mut product[1], &mut work).unwrap(),
        Ordering::Equal
    );
    assert_eq!(
        compare_roots(&mut square_two[1], &mut product[2], &mut work).unwrap(),
        Ordering::Less
    );
    assert_eq!(
        compare_roots(&mut product[0], &mut square_two[0], &mut work).unwrap(),
        Ordering::Equal
    );
}
#[test]
fn nearby_distinct_roots_are_separated_by_refinement() {
    let mut cancelled = || false;
    let mut work = Work::new(SketchIncidenceOptions::v1(1e-9), &mut cancelled);
    let mut root_two = isolated(&[-2, 0, 1], &mut work).remove(1);
    // 1.4142135623730951 is the binary64 rounding of sqrt(2), a different number.
    let value = 2f64.sqrt();
    let rational = ratio(value);
    let equation = RootEquation::new(
        IntegerPolynomial(vec![
            -rational.numerator().clone(),
            rational.denominator().clone(),
        ]),
        &mut work,
    )
    .unwrap();
    let point = DyadicBound::from_f64(value).unwrap();
    let mut rounded = IsolatedRoot {
        equation,
        lower: point.clone(),
        upper: point,
    };
    assert_eq!(
        compare_roots(&mut root_two, &mut rounded, &mut work).unwrap(),
        Ordering::Less
    );
    assert_eq!(sign_at_ratio(&integers(&[-2, 0, 1]), &rational), Sign::Plus);
}
fn constant_rows(rows: &[&[f64]]) -> super::bivariate::Bivariate {
    super::bivariate::Bivariate(
        rows.iter()
            .map(|row| RationalPolynomial(row.iter().copied().map(ratio).collect()))
            .collect(),
    )
}
#[test]
fn first_subresultant_recovers_the_single_common_root() {
    // (u - 1)(u - 2) and (u - 1)(u - 5) share u = 1; their first subresultant is -3u + 3.
    let first = constant_rows(&[&[2.], &[-3.], &[1.]]);
    let second = constant_rows(&[&[5.], &[-6.], &[1.]]);
    let row = super::bivariate::fibre_row(&first, [2, 2], &second).unwrap();
    assert_eq!(row.alpha.evaluate(&ratio(7.)), ratio(-3.));
    assert_eq!(row.beta.evaluate(&ratio(7.)), ratio(3.));
    let resultant = super::bivariate::resultant(&first, [2, 2], &second);
    assert_eq!(resultant.degree(), None);
}
#[test]
fn bivariate_resultant_vanishes_at_the_shared_coefficient_value() {
    // f = u - s and g = u^2 - 4 share a root exactly when s = 2 or s = -2.
    let first = super::bivariate::Bivariate(vec![polynomial(&[0., -1.]), polynomial(&[1.])]);
    let second = constant_rows(&[&[-4.], &[0.], &[1.]]);
    let resultant = super::bivariate::resultant(&first, [1, 2], &second);
    for root in [2., -2.] {
        assert_eq!(resultant.evaluate(&ratio(root)), ratio(0.));
    }
    assert_ne!(resultant.evaluate(&ratio(1.)), ratio(0.));
}
