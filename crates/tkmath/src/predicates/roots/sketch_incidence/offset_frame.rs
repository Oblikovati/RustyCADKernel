// SPDX-License-Identifier: GPL-2.0-only
//! Offset sources (ADR 0080). An offset keeps its basis parameterization: a line
//! offset is placed by the basis line parameter (the offset is orthogonal to the
//! line, so the projection parameter is shared), a spline offset by the spline
//! parameter, and an ellipse offset by the sweep place of its basis point. Ends,
//! sectors, bounds and domains are the basis's.
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::conic_source::ConicSource;
use super::failure::SketchIncidenceCause as Cause;
use super::implicit_curve::OffsetLineImplicit;
use super::input::{SketchIncidenceCurve, SketchIncidenceOffset};
use super::offset_geometry::tangent;
use super::offset_spans::{OffsetSpan, offset_span, offset_spans};
use super::rational_polynomial::{RationalPolynomial, ratio, zero};
use super::spans::{HomogeneousPoint, ParametricSpan, line_span};
use super::surd::Surd;
use super::sweep::PointPlacer;
use std::cmp::Ordering;

/// A line offset source: the squared implicit equation and its one surd piece.
pub(super) struct OffsetLineSource {
    pub implicit: OffsetLineImplicit,
    pub piece: ParametricSpan<Surd>,
}
/// The basis frame of an ellipse or spline offset.
pub(super) enum OffsetBase {
    Spline { bounds: [f64; 2], closed: bool },
    Conic(Box<ConicSource>),
}
/// An ellipse or spline offset source: surd pieces and their basis points.
pub(super) struct OffsetSource {
    pub pieces: Vec<ParametricSpan<Surd>>,
    pub basis: Vec<HomogeneousPoint>,
    pub base: OffsetBase,
}
impl OffsetLineSource {
    pub fn new(points: [[f64; 2]; 2], distance: f64) -> Self {
        let span = offset_span(line_span(points, false), &ratio(distance));
        Self {
            implicit: OffsetLineImplicit::new(points, distance),
            piece: span.span,
        }
    }
}
impl OffsetSource {
    /// Frame of a canonical ellipse or spline offset.
    pub fn new(offset: &SketchIncidenceOffset, work: &mut Work<'_>) -> Result<Self, Cause> {
        let spans = offset_spans(offset);
        let base = match offset.basis.as_ref() {
            SketchIncidenceCurve::Spline(spline) => OffsetBase::Spline {
                bounds: spline.bounds,
                closed: closed_chain(&spans, spline.bounds),
            },
            conic => OffsetBase::Conic(Box::new(ConicSource::new(conic, work)?)),
        };
        let (pieces, basis) = spans
            .into_iter()
            .map(|span| (span.span, span.basis))
            .unzip();
        Ok(Self {
            pieces,
            basis,
            base,
        })
    }
    /// Basis placer of a piece of an ellipse offset; None for parameter places.
    pub fn piece_placer(
        &self,
        piece: usize,
        root: Option<&IsolatedRoot>,
        work: &mut Work<'_>,
    ) -> Result<Option<PointPlacer>, Cause> {
        let (OffsetBase::Conic(conic), Some(root)) = (&self.base, root) else {
            return Ok(None);
        };
        let contact = RationalPolynomial::from_integer(&root.equation.polynomial);
        let relative = self.basis[piece].relative(&conic.frame.center);
        conic.frame.placer(&relative, &contact, work).map(Some)
    }
}
// A spline offset closes when its basis closes with one tangent direction there.
fn closed_chain(spans: &[OffsetSpan], bounds: [f64; 2]) -> bool {
    let (first, last) = (&spans[0].basis, &spans[spans.len() - 1].basis);
    let [start, end] = bounds.map(ratio);
    let at = |point: &HomogeneousPoint, parameter: &_| {
        let weight = point.w.evaluate(parameter);
        [&point.x, &point.y].map(|c| c.evaluate(parameter).divide(&weight))
    };
    if at(first, &start) != at(last, &end) {
        return false;
    }
    let [a, b] = [(first, &start), (last, &end)]
        .map(|(point, parameter)| tangent(point).map(|component| component.evaluate(parameter)));
    let cross = a[0].multiply(&b[1]).subtract(&a[1].multiply(&b[0]));
    let dot = a[0].multiply(&b[0]).add(&a[1].multiply(&b[1]));
    cross == zero() && dot.sign() == Ordering::Greater
}
