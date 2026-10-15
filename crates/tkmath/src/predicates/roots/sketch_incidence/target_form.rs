// SPDX-License-Identifier: GPL-2.0-only
//! Canonical target forms and their equation sides (ADR 0060, 0067, 0080).
//!
//! A target is implicit (line, conic, circle offset, line offset), parametric
//! with rational pieces (line, spline) or parametric with offset pieces (line,
//! ellipse or spline offset). A line and a line offset have both forms.
use super::super::certificate_arithmetic::ExactRatio;
use super::conic_source::shape;
use super::contact::TargetKey;
use super::implicit::LineImplicit;
use super::implicit_curve::OffsetLineImplicit;
use super::implicit_target::ImplicitTarget;
use super::input::{SketchIncidenceCurve, SketchIncidenceOffset, SketchIncidenceSpline};
use super::offset_geometry::circle_radius;
use super::offset_spans::offset_spans;
use super::parametric_target::{ParametricRequest, TargetExtent};
use super::rational_polynomial::{RationalPolynomial, ratio};
use super::sector::ArcBounds;
use super::spans::{line_span, spline_spans};
use super::surd::Surd;

/// The routing class of a canonical target.
pub(super) enum TargetForm<'a> {
    Line([[f64; 2]; 2], bool),
    Conic(&'a SketchIncidenceCurve),
    Spline(&'a SketchIncidenceSpline),
    OffsetLine(&'a SketchIncidenceOffset, [[f64; 2]; 2], bool),
    OffsetCurve(&'a SketchIncidenceOffset),
}
impl<'a> TargetForm<'a> {
    pub fn of(curve: &'a SketchIncidenceCurve) -> Self {
        match curve {
            SketchIncidenceCurve::Line { points, bounded } => Self::Line(*points, *bounded),
            SketchIncidenceCurve::Spline(spline) => Self::Spline(spline),
            SketchIncidenceCurve::Offset(offset) if circle_radius(offset).is_none() => {
                match offset.basis.as_ref() {
                    SketchIncidenceCurve::Line { points, bounded } => {
                        Self::OffsetLine(offset, *points, *bounded)
                    }
                    _ => Self::OffsetCurve(offset),
                }
            }
            conic => Self::Conic(conic),
        }
    }
    /// The implicit side of a line, conic or line offset target.
    pub fn implicit(&self) -> Option<ImplicitTarget<'a>> {
        Some(match self {
            Self::Line(points, bounded) => ImplicitTarget::Line {
                implicit: LineImplicit::through(*points),
                bounded: *bounded,
            },
            Self::Conic(curve) => conic_target(curve),
            Self::OffsetLine(offset, points, bounded) => ImplicitTarget::OffsetLine {
                implicit: OffsetLineImplicit::new(*points, offset.distance),
                bounded: *bounded,
            },
            Self::Spline(_) | Self::OffsetCurve(_) => return None,
        })
    }
    /// The rational pieces of a line or spline target.
    pub fn rational_request(
        &self,
        owner: TargetKey,
    ) -> Option<ParametricRequest<'a, RationalPolynomial>> {
        let (spans, bounds) = match self {
            Self::Line(points, bounded) => (
                vec![line_span(*points, *bounded)],
                bounded.then_some([0., 1.]),
            ),
            Self::Spline(spline) => (spline_spans(spline), Some(spline.bounds)),
            _ => return None,
        };
        Some(ParametricRequest {
            spans,
            extent: TargetExtent::Parameter(bounds),
            owner,
        })
    }
    /// The offset pieces of a line, ellipse or spline offset target.
    pub fn offset_request(&self, owner: TargetKey) -> Option<ParametricRequest<'a, Surd>> {
        let offset = match self {
            Self::OffsetLine(offset, ..) | Self::OffsetCurve(offset) => *offset,
            _ => return None,
        };
        let spans = offset_spans(offset);
        let extent = offset_extent(
            offset,
            spans.iter().map(|span| span.basis.clone()).collect(),
        );
        Some(ParametricRequest {
            spans: spans.into_iter().map(|span| span.span).collect(),
            extent,
            owner,
        })
    }
}
pub(super) fn conic_target(curve: &SketchIncidenceCurve) -> ImplicitTarget<'_> {
    let shape = shape(curve);
    let radii = shape.radii;
    ImplicitTarget::Conic {
        implicit: shape.implicit,
        sector: shape.bounds.map(|bounds| ArcBounds { bounds, radii }),
    }
}
fn offset_extent(
    offset: &SketchIncidenceOffset,
    basis: Vec<super::spans::HomogeneousPoint>,
) -> TargetExtent<'_> {
    match offset.basis.as_ref() {
        SketchIncidenceCurve::Line { bounded, .. } => {
            TargetExtent::Parameter(bounded.then_some([0., 1.]))
        }
        SketchIncidenceCurve::Spline(spline) => TargetExtent::Parameter(Some(spline.bounds)),
        SketchIncidenceCurve::Ellipse {
            center,
            radii,
            sector,
            ..
        } => TargetExtent::Sweep {
            basis,
            center: center.map(ratio) as [ExactRatio; 2],
            sector: sector.as_ref().map(|bounds| ArcBounds {
                bounds,
                radii: *radii,
            }),
        },
        other => unreachable!("offset basis {other:?}; expected a canonical non-circle basis"),
    }
}
