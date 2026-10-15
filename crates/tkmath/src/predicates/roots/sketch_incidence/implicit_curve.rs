// SPDX-License-Identifier: GPL-2.0-only
//! Contact equations of implicit curves composed with rational or offset pieces.
//!
//! Lines and conics give one equation. A line offset `det(e, p - A) = -d |e|`
//! (the points right of the line at distance `d`, ADR 0080) has an irrational
//! constant, so it is squared: `(w det)^2 - d^2 |e|^2 w^2 = 0`, and the branch
//! `sign(w det) = -sign(d)` is kept by an exact sign filter (`w > 0`).
use super::super::certificate_arithmetic::ExactRatio;
use super::coordinate::Coordinate;
use super::implicit::{ConicImplicit, LineImplicit};
use super::rational_polynomial::ratio;
use super::spans::HomogeneousPoint;
use super::surd_roots::{SurdEquation, SurdFilter};
use num_bigint::Sign;
use std::cmp::Ordering;

/// Offset of the line through two points: same direction, parameter and sides.
#[derive(Clone, Debug)]
pub(super) struct OffsetLineImplicit {
    pub line: LineImplicit,
    pub distance: ExactRatio,
}
impl OffsetLineImplicit {
    /// Offset by a nonzero distance (zero offsets are their basis line).
    pub fn new(points: [[f64; 2]; 2], distance: f64) -> Self {
        Self {
            line: LineImplicit::through(points),
            distance: ratio(distance),
        }
    }
    /// The squared offset equation with its branch filter.
    pub fn equation<T: Coordinate>(&self, point: &HomogeneousPoint<T>) -> SurdEquation {
        let determinant = self.line.compose(point);
        let squared = self.distance.multiply(&self.distance);
        let scale = squared.multiply(&self.line.squared_length()).negated();
        let value = determinant
            .multiply(&determinant)
            .add(&point.w.multiply(&point.w).scale(&scale));
        let branch = match self.distance.sign() {
            Ordering::Greater => Sign::Minus,
            _ => Sign::Plus,
        };
        SurdEquation {
            value: value.to_surd(),
            filters: vec![SurdFilter {
                value: determinant.to_surd(),
                sign: branch,
            }],
        }
    }
}
/// An implicit curve: the equation side of a contact pair.
#[derive(Clone, Debug)]
pub(super) enum ImplicitCurve {
    Line(LineImplicit),
    Conic(ConicImplicit),
    OffsetLine(OffsetLineImplicit),
}
impl ImplicitCurve {
    /// The contact equation of a piece `(x, y) / w` with `w > 0`.
    pub fn equation<T: Coordinate>(&self, point: &HomogeneousPoint<T>) -> SurdEquation {
        match self {
            Self::Line(line) => plain(line.compose(point)),
            Self::Conic(conic) => plain(conic.compose(point)),
            Self::OffsetLine(offset) => offset.equation(point),
        }
    }
}
fn plain<T: Coordinate>(value: T) -> SurdEquation {
    SurdEquation {
        value: value.to_surd(),
        filters: Vec::new(),
    }
}
