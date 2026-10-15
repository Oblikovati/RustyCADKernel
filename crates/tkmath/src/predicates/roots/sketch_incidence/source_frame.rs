// SPDX-License-Identifier: GPL-2.0-only
//! The edited source curve: its exact pieces or implicit equation, its places and
//! its positions relative to its own ends.
//!
//! A line is placed by `t` with `A + t (B - A)`; a spline by its parameter inside
//! its bounds; a conic by its sweep from the sector start ray (or its frame axis
//! when it is a full curve). The support of an arc is its full conic, so the part
//! beyond its end continues to its start: those contacts are `AfterEnd`.
use super::super::DyadicBound;
use super::super::certificate_arithmetic::ExactRatio;
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::canonical::{canonical, kind_name};
use super::conic_source::ConicSource;
use super::failure::SketchIncidenceCause as Cause;
use super::implicit::LineImplicit;
use super::input::{SketchIncidenceCurve, SketchIncidenceSpline};
use super::offset_geometry::circle_radius;
use super::place::{KeyValue, SourcePlace};
use super::rational_polynomial::{RationalPolynomial, ratio};
use super::report::SketchLinePosition;
use super::root_map::{MappedEquation, RationalMap};
use super::spans::{HomogeneousPoint, ParametricSpan, line_span, spline_spans};
use super::sweep::PointPlacer;
use std::cmp::Ordering;

pub(super) struct LineSource {
    pub implicit: LineImplicit,
    pub piece: ParametricSpan,
}
pub(super) struct SplineSource {
    pub pieces: Vec<ParametricSpan>,
    pub bounds: [f64; 2],
    /// The two ends are the same exact point.
    pub closed: bool,
}
pub(super) enum SourceFrame {
    Line(Box<LineSource>),
    Spline(SplineSource),
    Conic(Box<ConicSource>),
}
impl SourceFrame {
    /// Exact frame of an admitted source curve; angle bounds are placed with work.
    pub fn new(curve: &SketchIncidenceCurve, work: &mut Work<'_>) -> Result<Self, Cause> {
        let curve = canonical(curve)?;
        Ok(match curve.as_ref() {
            SketchIncidenceCurve::Line { points, .. } => Self::Line(Box::new(LineSource {
                implicit: LineImplicit::through(*points),
                piece: line_span(*points, false),
            })),
            SketchIncidenceCurve::Spline(spline) => Self::Spline(SplineSource::new(spline)),
            SketchIncidenceCurve::Offset(offset) if circle_radius(offset).is_none() => {
                return Err(Cause::OffsetPair {
                    source: kind_name(curve.as_ref()),
                    target: "any",
                });
            }
            conic => Self::Conic(Box::new(ConicSource::new(conic, work)?)),
        })
    }
    /// Normalize a sweep place of an angle-bounded arc (no effect elsewhere).
    pub fn normalize(&self, place: &mut SourcePlace, work: &mut Work<'_>) -> Result<(), Cause> {
        match self {
            Self::Conic(conic) => conic.normalize(place, work),
            _ => Ok(()),
        }
    }
    /// Exact position of a place relative to the source ends.
    pub fn position(
        &self,
        place: &mut SourcePlace,
        work: &mut Work<'_>,
    ) -> Result<SketchLinePosition, Cause> {
        match (self, place) {
            (Self::Line(_), SourcePlace::Parameter(value)) => value_position(value, [0., 1.], work),
            (Self::Spline(spline), SourcePlace::Parameter(value)) => {
                value_position(value, spline.bounds, work)
            }
            (Self::Conic(conic), place) => conic.position(place, work),
            _ => Err(Cause::Invariant("place kind differs from its source")),
        }
    }
    /// Representation name used by named refusals.
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Line(_) => "line",
            Self::Spline(_) => "spline",
            Self::Conic(_) => "conic",
        }
    }
    /// True for a full conic or a spline whose ends are the same exact point.
    pub fn closed(&self) -> bool {
        match self {
            Self::Line(_) => false,
            Self::Spline(spline) => spline.closed,
            Self::Conic(conic) => conic.end.is_none(),
        }
    }
    /// Exact places of the source start and end.
    pub fn ends(&self) -> [SourcePlace; 2] {
        let parameter = |value: f64| SourcePlace::Parameter(KeyValue::Ratio(ratio(value)));
        match self {
            Self::Line(_) => [parameter(0.), parameter(1.)],
            Self::Spline(spline) => spline.bounds.map(parameter),
            Self::Conic(conic) => [
                conic.start.clone(),
                conic.end.clone().unwrap_or_else(|| conic.start.clone()),
            ],
        }
    }
    /// Placer of target points given by polynomials of the target parameter.
    pub fn placer(
        &self,
        point: &HomogeneousPoint,
        contact: &RationalPolynomial,
        work: &mut Work<'_>,
    ) -> Result<PointPlacer, Cause> {
        match self {
            Self::Line(line) => {
                let map = RationalMap {
                    numerator: line.implicit.along(point),
                    denominator: point.w.scale(&line.implicit.squared_length()),
                };
                Ok(PointPlacer::Parameter(MappedEquation::new(
                    contact, map, work,
                )?))
            }
            Self::Conic(conic) => {
                let relative = point.relative(&conic.frame.center);
                conic.frame.placer(&relative, contact, work)
            }
            Self::Spline(_) => Err(Cause::Invariant("a spline source has no implicit placer")),
        }
    }
}
/// Exact position of a key value relative to two increasing binary64 values.
pub(super) fn value_position(
    value: &mut KeyValue,
    range: [f64; 2],
    work: &mut Work<'_>,
) -> Result<SketchLinePosition, Cause> {
    match value {
        KeyValue::Root(root) => root_position(root, range, work),
        KeyValue::Ratio(value) => Ok(ratio_position(value, range)),
        KeyValue::Angle(_) => Err(Cause::Invariant("a parameter place holds an angle")),
    }
}
/// Exact position of an isolated root relative to two increasing binary64 values.
pub(super) fn root_position(
    root: &mut IsolatedRoot,
    range: [f64; 2],
    work: &mut Work<'_>,
) -> Result<SketchLinePosition, Cause> {
    let [start, end] = range.map(dyadic);
    Ok(match root.split_at(&start, work)? {
        Ordering::Less => SketchLinePosition::BeforeStart,
        Ordering::Equal => SketchLinePosition::AtStart,
        Ordering::Greater => match root.split_at(&end, work)? {
            Ordering::Less => SketchLinePosition::Interior,
            Ordering::Equal => SketchLinePosition::AtEnd,
            Ordering::Greater => SketchLinePosition::AfterEnd,
        },
    })
}
/// Exact position of a rational value relative to two increasing binary64 values.
pub(super) fn ratio_position(value: &ExactRatio, range: [f64; 2]) -> SketchLinePosition {
    match (value.cmp(&ratio(range[0])), value.cmp(&ratio(range[1]))) {
        (Ordering::Less, _) => SketchLinePosition::BeforeStart,
        (Ordering::Equal, _) => SketchLinePosition::AtStart,
        (_, Ordering::Less) => SketchLinePosition::Interior,
        (_, Ordering::Equal) => SketchLinePosition::AtEnd,
        _ => SketchLinePosition::AfterEnd,
    }
}
impl SplineSource {
    fn new(spline: &SketchIncidenceSpline) -> Self {
        let pieces = spline_spans(spline);
        let closed = end_point(&pieces[0], spline.bounds[0])
            == end_point(&pieces[pieces.len() - 1], spline.bounds[1]);
        Self {
            pieces,
            bounds: spline.bounds,
            closed,
        }
    }
}
fn end_point(piece: &ParametricSpan, parameter: f64) -> [ExactRatio; 2] {
    let parameter = ratio(parameter);
    let weight = piece.point.w.evaluate(&parameter);
    [&piece.point.x, &piece.point.y]
        .map(|coordinate| coordinate.evaluate(&parameter).divide(&weight))
}
/// Dyadic value of an admitted binary64 parameter.
pub(super) fn dyadic(value: f64) -> DyadicBound {
    DyadicBound::from_f64(value).expect("admitted finite parameter")
}
