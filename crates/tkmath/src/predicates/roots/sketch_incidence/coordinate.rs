// SPDX-License-Identifier: GPL-2.0-only
//! Values of the contact variable that position tests read: rational polynomials
//! and quadratic surds (ADR 0080). Both give exact signs at isolated roots and
//! outward enclosures over brackets; a surd also needs a square-root precision.
use super::super::certificate_arithmetic::RatioInterval;
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::contact::TargetKey;
use super::failure::SketchIncidenceCause as Cause;
use super::overlap::span_overlap;
use super::rational_polynomial::{RationalPolynomial, Ring};
use super::report::SketchLineOverlap;
use super::root_sign::sign_at_root;
use super::source_frame::SourceFrame;
use super::spans::{HomogeneousPoint, ParametricSpan};
use super::surd::Surd;
use super::sweep::PointPlacer;
use num_bigint::Sign;

/// A ring element of the contact variable with exact signs and enclosures.
pub(super) trait Coordinate: Ring {
    /// Exact sign at an isolated root; the bracket may narrow.
    fn sign_at(&self, root: &mut IsolatedRoot, work: &mut Work<'_>) -> Result<Sign, Cause>;
    /// Outward enclosure over a closed interval, square roots to `bits` bits.
    fn enclose_at(&self, parameter: &RatioInterval, bits: u32) -> RatioInterval;
    /// The same value as a surd (a rational one has a zero radical part).
    fn to_surd(&self) -> Surd;
    /// Placer of a target point of this kind on an implicit source.
    fn place_on(
        point: &HomogeneousPoint<Self>,
        source: &SourceFrame,
        contact: &RationalPolynomial,
        work: &mut Work<'_>,
    ) -> Result<PointPlacer, Cause>;
    /// The coincident target span of this kind on a line source.
    fn overlap_on(
        span: &ParametricSpan<Self>,
        source: &SourceFrame,
        owner: TargetKey,
        work: &mut Work<'_>,
    ) -> Result<SketchLineOverlap, Cause>;
}
impl Coordinate for RationalPolynomial {
    fn sign_at(&self, root: &mut IsolatedRoot, work: &mut Work<'_>) -> Result<Sign, Cause> {
        sign_at_root(root, self, work)
    }
    fn enclose_at(&self, parameter: &RatioInterval, _bits: u32) -> RatioInterval {
        self.enclose(parameter)
    }
    fn to_surd(&self) -> Surd {
        Surd::rational_only(self.clone())
    }
    fn place_on(
        point: &HomogeneousPoint<Self>,
        source: &SourceFrame,
        contact: &RationalPolynomial,
        work: &mut Work<'_>,
    ) -> Result<PointPlacer, Cause> {
        source.placer(point, contact, work)
    }
    fn overlap_on(
        span: &ParametricSpan<Self>,
        source: &SourceFrame,
        owner: TargetKey,
        work: &mut Work<'_>,
    ) -> Result<SketchLineOverlap, Cause> {
        span_overlap(span, source, owner, work)
    }
}
impl Coordinate for Surd {
    fn sign_at(&self, root: &mut IsolatedRoot, work: &mut Work<'_>) -> Result<Sign, Cause> {
        self.sign_at_root(root, work)
    }
    fn enclose_at(&self, parameter: &RatioInterval, bits: u32) -> RatioInterval {
        self.enclose(parameter, bits)
    }
    fn to_surd(&self) -> Surd {
        self.clone()
    }
    fn place_on(
        point: &HomogeneousPoint<Self>,
        source: &SourceFrame,
        _contact: &RationalPolynomial,
        _work: &mut Work<'_>,
    ) -> Result<PointPlacer, Cause> {
        // Surd keys are compared lazily; no image equation is built here.
        source.surd_placer(point)
    }
    // A straight offset piece on a line is parallel to it, so its offset normal is
    // orthogonal to the line and the basis piece (the radical parts) has the same
    // line parameters (ADR 0080).
    fn overlap_on(
        span: &ParametricSpan<Self>,
        source: &SourceFrame,
        owner: TargetKey,
        work: &mut Work<'_>,
    ) -> Result<SketchLineOverlap, Cause> {
        let basis = ParametricSpan {
            point: HomogeneousPoint {
                x: span.point.x.radical.clone(),
                y: span.point.y.radical.clone(),
                w: span.point.w.radical.clone(),
            },
            range: span.range,
        };
        span_overlap(&basis, source, owner, work)
    }
}
/// Square-root precision for enclosures over a root bracket: about twice the
/// bracket's binary digits (so the square root error stays below the bracket
/// effect), and a fixed 2200 bits for an exact root, below any binary64 gap.
pub(super) fn precision(root: &IsolatedRoot) -> u32 {
    if root.exact().is_some() {
        return 2200;
    }
    let [lower, upper] = root.ratio_bounds();
    let width = upper.subtract(&lower);
    let digits = width.denominator().bits() as i64 - width.numerator().bits() as i64;
    (2 * digits + 96).clamp(96, 4400) as u32
}
