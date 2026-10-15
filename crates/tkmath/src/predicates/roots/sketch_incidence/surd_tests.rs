// SPDX-License-Identifier: GPL-2.0-only
//! Hand-derived checks of surd signs, square radicands and surd root filtering.
use super::budget::Work;
use super::input::SketchIncidenceOptions;
use super::pieces::PieceRoot;
use super::rational_polynomial::{RationalPolynomial, ratio};
use super::surd::Surd;
use super::surd_roots::{SurdEquation, SurdFilter, surd_piece_roots};
use super::surd_square::polynomial_sqrt;
use num_bigint::Sign;
use std::sync::Arc;

fn polynomial(coefficients: &[f64]) -> RationalPolynomial {
    RationalPolynomial(coefficients.iter().copied().map(ratio).collect())
}
fn surd(rational: &[f64], radical: &[f64], radicand: &[f64]) -> Surd {
    Surd {
        rational: polynomial(rational),
        radical: polynomial(radical),
        radicand: Arc::new(polynomial(radicand)),
    }
}
fn roots(value: Surd, filters: Vec<SurdFilter>) -> Option<Vec<PieceRoot>> {
    let mut never = || false;
    let mut work = Work::new(SketchIncidenceOptions::v1(1e-12), &mut never);
    let equation = SurdEquation { value, filters };
    surd_piece_roots(&equation, Some([-2., 2.]), 0, &mut work).unwrap()
}
#[test]
fn signs_follow_the_norm_when_parts_disagree() {
    let point = ratio(0.);
    assert_eq!(
        surd(&[1.], &[-1.], &[2.]).sign_at_ratio(&point),
        Sign::Minus
    );
    assert_eq!(surd(&[3.], &[-1.], &[4.]).sign_at_ratio(&point), Sign::Plus);
    assert_eq!(
        surd(&[2.], &[-1.], &[4.]).sign_at_ratio(&point),
        Sign::NoSign
    );
    assert_eq!(
        surd(&[0.], &[-1.], &[4.]).sign_at_ratio(&point),
        Sign::Minus
    );
}
#[test]
fn polynomial_square_roots_are_exact_or_absent() {
    let root = polynomial(&[3., 2., 1.]);
    assert_eq!(polynomial_sqrt(&root.multiply(&root)), Some(root));
    assert_eq!(polynomial_sqrt(&polynomial(&[1., 0., 1.])), None);
    assert_eq!(polynomial_sqrt(&polynomial(&[2.])), None);
}
// 2t = sqrt(t^2 + 3) at t = 1 only; t = -1 solves the conjugate branch.
#[test]
fn spurious_roots_of_the_squared_equation_are_rejected() {
    let found = roots(surd(&[0., 2.], &[-1.], &[3., 0., 1.]), Vec::new()).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].contact.exact(), Some(&super::pieces::dyadic(1.)));
    assert_eq!(found[0].sides, [Some(Sign::Minus), Some(Sign::Plus)]);
    assert_eq!(found[0].multiplicity, [Some(1), Some(1)]);
    let rejected = SurdFilter {
        value: surd(&[0., 1.], &[], &[3., 0., 1.]),
        sign: Sign::Minus,
    };
    assert!(
        roots(surd(&[0., 2.], &[-1.], &[3., 0., 1.]), vec![rejected])
            .unwrap()
            .is_empty()
    );
}
// t (1 + sqrt(1 + t^2)) is simple at 0; t (1 - sqrt(1 + t^2)) ~ -t^3 / 2 has order 3.
#[test]
fn common_zeros_of_both_parts_get_the_branch_order() {
    let simple = roots(surd(&[0., 1.], &[0., 1.], &[1., 0., 1.]), Vec::new()).unwrap();
    assert_eq!(simple.len(), 1);
    assert_eq!(simple[0].multiplicity, [Some(1), Some(1)]);
    assert_eq!(simple[0].sides, [Some(Sign::Minus), Some(Sign::Plus)]);
    let flat = roots(surd(&[0., 1.], &[0., -1.], &[1., 0., 1.]), Vec::new()).unwrap();
    assert_eq!(flat.len(), 1);
    assert_eq!(flat[0].multiplicity, [Some(3), Some(3)]);
    assert_eq!(flat[0].sides, [Some(Sign::Plus), Some(Sign::Minus)]);
}
// sqrt(4) = 2: 2 - sqrt(4) vanishes identically, 2 + sqrt(4) never vanishes.
#[test]
fn square_radicands_reduce_to_rational_equations() {
    assert!(roots(surd(&[2.], &[-1.], &[4.]), Vec::new()).is_none());
    assert!(
        roots(surd(&[2.], &[1.], &[4.]), Vec::new())
            .unwrap()
            .is_empty()
    );
    let found = roots(surd(&[0., 1.], &[1.], &[4.]), Vec::new()).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].contact.exact(), Some(&super::pieces::dyadic(-2.)));
}
