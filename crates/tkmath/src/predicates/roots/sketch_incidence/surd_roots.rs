// SPDX-License-Identifier: GPL-2.0-only
//! Complete roots of a surd contact equation `E = P + R sqrt(G)` on one piece
//! (ADR 0080): the offset equation is squared, then spurious roots are rejected by
//! exact signs.
//!
//! Every root of `E` is a root of its norm `H = P^2 - R^2 G`; a root of `H` is a
//! root of `E` exactly when `P` and `R` vanish together or have opposite signs
//! (then `|P| = |R| sqrt(G)`). Extra filters (surds with a required sign) select a
//! branch of a squared target equation. Sides are exact surd signs at points
//! strictly between consecutive roots of `H`, where `E` cannot change sign.
use super::super::certificate_arithmetic::ExactRatio;
use super::super::polynomial::IntegerPolynomial;
use super::super::{DyadicBound, RootBracket};
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::equation::{cauchy_bounds, isolate};
use super::failure::SketchIncidenceCause as Cause;
use super::pieces::{PieceRoot, dyadic, open_sides, piece_roots};
use super::rational_polynomial::RationalPolynomial;
use super::root_sign::sign_at_root;
use super::surd::Surd;
use super::surd_square::rational_form;
use num_bigint::Sign;

/// A surd that must have a stated exact sign at every accepted root.
#[derive(Clone, Debug)]
pub(super) struct SurdFilter {
    pub value: Surd,
    pub sign: Sign,
}
/// A contact equation with the branch filters of squared target equations.
#[derive(Clone, Debug)]
pub(super) struct SurdEquation {
    pub value: Surd,
    pub filters: Vec<SurdFilter>,
}
/// The polynomial whose roots are isolated: `E` itself when it is rational, else its norm.
pub(super) enum ContactForm {
    Rational(RationalPolynomial),
    Norm(RationalPolynomial),
}
impl SurdEquation {
    /// The isolated contact polynomial; a vanishing norm with `R != 0` means `G`
    /// is a square, so `E` is rational.
    pub fn form(&self, range: Option<[f64; 2]>) -> Result<ContactForm, Cause> {
        if self.value.is_rational() {
            return Ok(ContactForm::Rational(self.value.rational.clone()));
        }
        let norm = self.value.norm();
        if norm.degree().is_none() {
            return Ok(ContactForm::Rational(rational_form(&self.value, range)?));
        }
        Ok(ContactForm::Norm(norm))
    }
    /// The polynomial carrying the contact roots (image equations are built on it).
    pub fn contact_polynomial(&self, range: Option<[f64; 2]>) -> Result<RationalPolynomial, Cause> {
        Ok(match self.form(range)? {
            ContactForm::Rational(polynomial) | ContactForm::Norm(polynomial) => polynomial,
        })
    }
}
/// Every accepted root on the piece; None when `E` vanishes identically.
pub(super) fn surd_piece_roots(
    equation: &SurdEquation,
    range: Option<[f64; 2]>,
    piece: usize,
    work: &mut Work<'_>,
) -> Result<Option<Vec<PieceRoot>>, Cause> {
    match equation.form(range)? {
        ContactForm::Rational(rational) => filtered_roots(&rational, equation, range, piece, work),
        ContactForm::Norm(norm) => {
            let norm = norm.to_integer(work.max_degree())?.expect("a nonzero norm");
            let request = NormRoots {
                equation,
                range,
                piece,
            };
            request.roots(&norm, work).map(Some)
        }
    }
}
/// Roots of the norm `H` of one piece's surd equation.
struct NormRoots<'a> {
    equation: &'a SurdEquation,
    range: Option<[f64; 2]>,
    piece: usize,
}
impl NormRoots<'_> {
    fn roots(
        &self,
        norm: &IntegerPolynomial,
        work: &mut Work<'_>,
    ) -> Result<Vec<PieceRoot>, Cause> {
        let bounds = match self.range {
            Some(range) => range.map(dyadic),
            None => cauchy_bounds(norm),
        };
        let (root_equation, isolation) = isolate(norm, bounds.clone(), work)?;
        let brackets = isolation.roots();
        let mut roots = Vec::new();
        for index in 0..brackets.len() {
            let mut contact = IsolatedRoot::from_bracket(&root_equation, &brackets[index]);
            if accepted(self.equation, &mut contact, work)? {
                let sides = side_points(brackets, index, &bounds);
                let order = brackets[index].multiplicity();
                roots.push(self.root(contact, sides, order, work)?);
            }
        }
        Ok(roots)
    }
    fn root(
        &self,
        mut contact: IsolatedRoot,
        points: [DyadicBound; 2],
        norm_order: usize,
        work: &mut Work<'_>,
    ) -> Result<PieceRoot, Cause> {
        let value = &self.equation.value;
        let sides = points.map(|point| value.sign_at_ratio(&ExactRatio::from_dyadic(&point)));
        let order = multiplicity(value, &mut contact, norm_order, work)?;
        let open = open_sides(&contact, self.range);
        Ok(PieceRoot {
            contact,
            sides: [0, 1].map(|side| open[side].then_some(sides[side])),
            multiplicity: open.map(|side| side.then_some(order)),
            piece: self.piece,
        })
    }
}
// A rational equation (no radical part): its roots, then the branch filters.
fn filtered_roots(
    rational: &RationalPolynomial,
    equation: &SurdEquation,
    range: Option<[f64; 2]>,
    piece: usize,
    work: &mut Work<'_>,
) -> Result<Option<Vec<PieceRoot>>, Cause> {
    let Some(roots) = piece_roots(rational, range, piece, work)? else {
        return Ok(None);
    };
    let mut kept = Vec::with_capacity(roots.len());
    for mut root in roots {
        if passes_filters(equation, &mut root.contact, work)? {
            kept.push(root);
        }
    }
    Ok(Some(kept))
}
fn accepted(
    equation: &SurdEquation,
    root: &mut IsolatedRoot,
    work: &mut Work<'_>,
) -> Result<bool, Cause> {
    if equation.value.sign_at_root(root, work)? != Sign::NoSign {
        return Ok(false);
    }
    passes_filters(equation, root, work)
}
fn passes_filters(
    equation: &SurdEquation,
    root: &mut IsolatedRoot,
    work: &mut Work<'_>,
) -> Result<bool, Cause> {
    for filter in &equation.filters {
        if filter.value.sign_at_root(root, work)? != filter.sign {
            return Ok(false);
        }
    }
    Ok(true)
}
// Points just left and right of root `index`, with no root of `H` in between.
fn side_points(
    brackets: &[RootBracket],
    index: usize,
    bounds: &[DyadicBound; 2],
) -> [DyadicBound; 2] {
    let (lower, upper) = brackets[index].bounds();
    if lower != upper {
        return [lower.clone(), upper.clone()];
    }
    let previous = index
        .checked_sub(1)
        .map_or(&bounds[0], |before| brackets[before].bounds().1);
    let next = brackets
        .get(index + 1)
        .map_or(&bounds[1], |after| after.bounds().0);
    [previous.midpoint(lower), lower.midpoint(next)]
}
/// Order of `E` at an accepted root, from the norm's multiplicity `norm_order`.
///
/// With `R(t*) != 0` the conjugate `P - R sqrt(G) = 2P` is nonzero, so both
/// orders agree. Otherwise, with orders `p` of `P` and `r` of `R`, the order is
/// `min(p, r)` when they differ. When they are equal (`m`), the leading
/// coefficient is `P^(m) + R^(m) sqrt(G)`: nonzero gives `m`; zero leaves the
/// conjugate at order `m`, so `E` has the rest of the norm's order.
fn multiplicity(
    value: &Surd,
    root: &mut IsolatedRoot,
    norm_order: usize,
    work: &mut Work<'_>,
) -> Result<usize, Cause> {
    if sign_at_root(root, &value.radical, work)? != Sign::NoSign {
        return Ok(norm_order);
    }
    let rational = order(root, &value.rational, work)?;
    let radical = order(root, &value.radical, work)?;
    if rational != radical {
        return Ok(rational.min(radical));
    }
    let leading = Surd {
        rational: derivative(&value.rational, rational),
        radical: derivative(&value.radical, rational),
        radicand: value.radicand.clone(),
    };
    match leading.sign_at_root(root, work)? {
        Sign::NoSign => Ok(norm_order - rational),
        _ => Ok(rational),
    }
}
// Number of derivatives vanishing at the root; unbounded for the zero polynomial.
fn order(
    root: &mut IsolatedRoot,
    polynomial: &RationalPolynomial,
    work: &mut Work<'_>,
) -> Result<usize, Cause> {
    let mut current = polynomial.clone();
    let mut count = 0;
    while current.degree().is_some() && sign_at_root(root, &current, work)? == Sign::NoSign {
        current = current.derivative();
        count += 1;
    }
    Ok(if current.degree().is_none() {
        usize::MAX
    } else {
        count
    })
}
fn derivative(polynomial: &RationalPolynomial, times: usize) -> RationalPolynomial {
    (0..times).fold(polynomial.clone(), |current, _| current.derivative())
}
