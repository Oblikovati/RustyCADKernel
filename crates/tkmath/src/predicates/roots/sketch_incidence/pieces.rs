// SPDX-License-Identifier: GPL-2.0-only
//! Complete roots of a contact equation on consecutive pieces of one curve
//! (spline spans or a single line), merged at shared piece ends.
use super::super::DyadicBound;
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::equation::{cauchy_bounds, isolate, side_signs};
use super::failure::SketchIncidenceCause as Cause;
use super::rational_polynomial::RationalPolynomial;
use num_bigint::Sign;

/// One isolated root on one piece; a side at the piece end is None.
#[derive(Clone, Debug)]
pub(super) struct PieceRoot {
    pub contact: IsolatedRoot,
    pub sides: [Option<Sign>; 2],
    pub multiplicity: [Option<usize>; 2],
    pub piece: usize,
}
/// Every root of a nonzero piece equation; None when the equation vanishes identically.
pub(super) fn piece_roots(
    equation: &RationalPolynomial,
    range: Option<[f64; 2]>,
    piece: usize,
    work: &mut Work<'_>,
) -> Result<Option<Vec<PieceRoot>>, Cause> {
    let Some(polynomial) = equation.to_integer(work.max_degree())? else {
        return Ok(None);
    };
    let bounds = match range {
        Some(range) => range.map(dyadic),
        None => cauchy_bounds(&polynomial),
    };
    let (root_equation, isolation) = isolate(&polynomial, bounds, work)?;
    let roots = isolation.roots().iter().map(|bracket| {
        let contact = IsolatedRoot::from_bracket(&root_equation, bracket);
        let [left, right] = side_signs(&polynomial, bracket);
        let open = open_sides(&contact, range);
        PieceRoot {
            contact,
            sides: [open[0].then_some(left), open[1].then_some(right)],
            multiplicity: open.map(|side| side.then_some(bracket.multiplicity())),
            piece,
        }
    });
    Ok(Some(roots.collect()))
}
pub(super) fn dyadic(value: f64) -> DyadicBound {
    DyadicBound::from_f64(value).expect("admitted finite parameter")
}
// A root exactly at a piece end has no side within this piece.
pub(super) fn open_sides(contact: &IsolatedRoot, range: Option<[f64; 2]>) -> [bool; 2] {
    let ends = range.map(|range| range.map(dyadic));
    let at = |index: usize| {
        ends.as_ref()
            .is_some_and(|ends| contact.exact() == Some(&ends[index]))
    };
    [!at(0), !at(1)]
}
/// Roots of consecutive pieces; a root at a shared end is one contact with both sides.
#[derive(Default)]
pub(super) struct MergedRoots {
    pub roots: Vec<PieceRoot>,
    previous_overlap: bool,
}
impl MergedRoots {
    // A root exactly at a piece start is the same point as the previous piece's end
    // root (C0 continuity). Its two sides come from the two adjacent pieces.
    pub fn add_roots(&mut self, roots: Vec<PieceRoot>) {
        for root in roots {
            if root.sides[0].is_some() {
                self.roots.push(root);
            } else if self.previous_overlap {
                continue;
            } else if let Some(last) = self.roots.last_mut().filter(|last| last.sides[1].is_none())
            {
                last.sides[1] = root.sides[1];
                last.multiplicity[1] = root.multiplicity[1];
            } else {
                self.roots.push(root);
            }
        }
        self.previous_overlap = false;
    }
    /// Around a closed chain of pieces (an ellipse offset's two charts), a root at
    /// the last piece end is the root at the first piece start.
    pub fn close_cycle(&mut self) {
        let count = self.roots.len();
        let wraps = count >= 2
            && self.roots[0].sides[0].is_none()
            && self.roots[count - 1].sides[1].is_none();
        if let Some(last) = wraps.then(|| self.roots.pop()).flatten() {
            self.roots[0].sides[0] = last.sides[0];
            self.roots[0].multiplicity[0] = last.multiplicity[0];
        }
    }
    // An overlap boundary is not an isolated contact.
    pub fn add_overlap(&mut self) {
        if self
            .roots
            .last()
            .is_some_and(|last| last.sides[1].is_none())
        {
            self.roots.pop();
        }
        self.previous_overlap = true;
    }
}
