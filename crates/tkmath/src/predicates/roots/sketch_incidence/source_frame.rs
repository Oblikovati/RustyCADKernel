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
use super::failure::SketchIncidenceCause as Cause;
use super::implicit::{ConicImplicit, LineImplicit};
use super::input::{SketchIncidenceCurve, SketchIncidenceSpline};
use super::place::{KeyValue, SketchSweepChart, SourcePlace, compare_places};
use super::rational_polynomial::{RationalPolynomial, ratio};
use super::report::SketchLinePosition;
use super::root_map::{MappedEquation, RationalMap};
use super::spans::{HomogeneousPoint, ParametricSpan, line_span, spline_spans};
use super::sweep::{PointPlacer, SweepFrame};
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
pub(super) struct ConicSource {
    pub implicit: ConicImplicit,
    pub frame: SweepFrame,
    /// Exact place of the sector end ray; None for a full conic.
    pub end: Option<SourcePlace>,
}
pub(super) enum SourceFrame {
    Line(Box<LineSource>),
    Spline(SplineSource),
    Conic(Box<ConicSource>),
}
impl SourceFrame {
    /// Exact frame of an admitted source curve.
    pub fn new(curve: &SketchIncidenceCurve) -> Self {
        match curve {
            SketchIncidenceCurve::Line { points, .. } => Self::Line(Box::new(LineSource {
                implicit: LineImplicit::through(*points),
                piece: line_span(*points, false),
            })),
            SketchIncidenceCurve::Spline(spline) => Self::Spline(SplineSource::new(spline)),
            conic => Self::Conic(Box::new(ConicSource::new(conic))),
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
            Self::Conic(conic) => [conic.start(), conic.end_or_start()],
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
impl ConicSource {
    fn new(curve: &SketchIncidenceCurve) -> Self {
        let (implicit, center, axis, sector) = match curve {
            SketchIncidenceCurve::Circle {
                center,
                radius,
                sector,
            } => (
                ConicImplicit::circle(*center, *radius),
                *center,
                [1., 0.],
                sector,
            ),
            SketchIncidenceCurve::Ellipse {
                center,
                major_axis,
                radii,
                sector,
            } => (
                ConicImplicit::ellipse(*center, *major_axis, *radii),
                *center,
                *major_axis,
                sector,
            ),
            other => unreachable!("curve {other:?}; expected a circle or ellipse"),
        };
        let (reference, counterclockwise) = sector.map_or((axis, true), |sector| {
            (sector.start, sector.counterclockwise)
        });
        let frame = SweepFrame::new(center, reference, counterclockwise);
        let end = sector.map(|sector| frame.exact_place(&sector.end.map(ratio)));
        Self {
            implicit,
            frame,
            end,
        }
    }
    fn start(&self) -> SourcePlace {
        SourcePlace::Sweep {
            chart: SketchSweepChart::Reference,
            cotangent: None,
        }
    }
    fn end_or_start(&self) -> SourcePlace {
        self.end.clone().unwrap_or_else(|| self.start())
    }
    fn position(
        &self,
        place: &mut SourcePlace,
        work: &mut Work<'_>,
    ) -> Result<SketchLinePosition, Cause> {
        let Some(end) = &self.end else {
            return Ok(SketchLinePosition::Interior);
        };
        if *place == self.start() {
            return Ok(SketchLinePosition::AtStart);
        }
        Ok(match compare_places(place, &mut end.clone(), work)? {
            Ordering::Less => SketchLinePosition::Interior,
            Ordering::Equal => SketchLinePosition::AtEnd,
            Ordering::Greater => SketchLinePosition::AfterEnd,
        })
    }
}
/// Dyadic value of an admitted binary64 parameter.
pub(super) fn dyadic(value: f64) -> DyadicBound {
    DyadicBound::from_f64(value).expect("admitted finite parameter")
}
