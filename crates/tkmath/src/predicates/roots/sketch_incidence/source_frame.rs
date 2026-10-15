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
use super::canonical::canonical;
use super::conic_source::ConicSource;
use super::failure::SketchIncidenceCause as Cause;
use super::implicit::LineImplicit;
use super::implicit_curve::ImplicitCurve;
use super::input::{SketchIncidenceCurve, SketchIncidenceOffset, SketchIncidenceSpline};
use super::offset_frame::{OffsetBase, OffsetLineSource, OffsetSource};
use super::offset_geometry::circle_radius;
use super::pieces::PieceRoot;
use super::place::{KeyValue, SourcePlace};
use super::rational_polynomial::ratio;
use super::report::SketchLinePosition;
use super::spans::{ParametricSpan, line_span, spline_spans};
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
    OffsetLine(Box<OffsetLineSource>),
    Offset(Box<OffsetSource>),
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
                Self::offset(offset, work)?
            }
            conic => Self::Conic(Box::new(ConicSource::new(conic, work)?)),
        })
    }
    fn offset(offset: &SketchIncidenceOffset, work: &mut Work<'_>) -> Result<Self, Cause> {
        Ok(match offset.basis.as_ref() {
            SketchIncidenceCurve::Line { points, .. } => {
                Self::OffsetLine(Box::new(OffsetLineSource::new(*points, offset.distance)))
            }
            _ => Self::Offset(Box::new(OffsetSource::new(offset, work)?)),
        })
    }
    /// The conic whose sweep places this source: a conic, or an ellipse offset's basis.
    pub fn conic(&self) -> Option<&ConicSource> {
        match self {
            Self::Conic(conic) => Some(conic),
            Self::Offset(offset) => match &offset.base {
                OffsetBase::Conic(conic) => Some(conic),
                OffsetBase::Spline { .. } => None,
            },
            _ => None,
        }
    }
    /// Parameter range of a parameter-placed source.
    pub fn parameter_range(&self) -> Option<[f64; 2]> {
        match self {
            Self::Line(_) | Self::OffsetLine(_) => Some([0., 1.]),
            Self::Spline(spline) => Some(spline.bounds),
            Self::Offset(offset) => match offset.base {
                OffsetBase::Spline { bounds, .. } => Some(bounds),
                OffsetBase::Conic(_) => None,
            },
            Self::Conic(_) => None,
        }
    }
    /// Normalize a sweep place of an angle-bounded arc (no effect elsewhere).
    pub fn normalize(&self, place: &mut SourcePlace, work: &mut Work<'_>) -> Result<(), Cause> {
        match self.conic() {
            Some(conic) => conic.normalize(place, work),
            None => Ok(()),
        }
    }
    /// Exact position of a place relative to the source ends.
    pub fn position(
        &self,
        place: &mut SourcePlace,
        work: &mut Work<'_>,
    ) -> Result<SketchLinePosition, Cause> {
        if let Some(conic) = self.conic() {
            return conic.position(place, work);
        }
        match (self.parameter_range(), place) {
            (Some(range), SourcePlace::Parameter(value)) => value_position(value, range, work),
            _ => Err(Cause::Invariant("place kind differs from its source")),
        }
    }
    /// Representation name used by named refusals.
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Line(_) => "line",
            Self::Spline(_) => "spline",
            Self::Conic(_) => "conic",
            Self::OffsetLine(_) => "offset line",
            Self::Offset(_) if self.conic().is_some() => "offset ellipse",
            Self::Offset(_) => "offset spline",
        }
    }
    /// True for a full conic (or its offset), or a spline (or spline offset) whose
    /// ends are the same exact point.
    pub fn closed(&self) -> bool {
        if let Some(conic) = self.conic() {
            return conic.end.is_none();
        }
        match self {
            Self::Spline(spline) => spline.closed,
            Self::Offset(offset) => matches!(offset.base, OffsetBase::Spline { closed: true, .. }),
            _ => false,
        }
    }
    /// Exact places of the source start and end.
    pub fn ends(&self) -> [SourcePlace; 2] {
        let parameter = |value: f64| SourcePlace::Parameter(KeyValue::Ratio(ratio(value)));
        match (self.conic(), self.parameter_range()) {
            (Some(conic), _) => [
                conic.start.clone(),
                conic.end.clone().unwrap_or_else(|| conic.start.clone()),
            ],
            (None, range) => range.unwrap_or([0., 1.]).map(parameter),
        }
    }
    /// The implicit equation of a line, conic or line offset source.
    pub fn implicit_curve(&self) -> Option<ImplicitCurve> {
        match self {
            Self::Line(line) => Some(ImplicitCurve::Line(line.implicit.clone())),
            Self::Conic(conic) => Some(ImplicitCurve::Conic(conic.implicit.clone())),
            Self::OffsetLine(line) => Some(ImplicitCurve::OffsetLine(line.implicit.clone())),
            _ => None,
        }
    }
    /// Basis placer of a source piece (ellipse offsets); None for parameter places.
    pub fn piece_placer(
        &self,
        piece: usize,
        root: Option<&PieceRoot>,
        work: &mut Work<'_>,
    ) -> Result<Option<PointPlacer>, Cause> {
        match self {
            Self::Offset(offset) => offset.piece_placer(piece, root.map(|r| &r.contact), work),
            _ => Ok(None),
        }
    }
    /// True when the source pieces close into a cycle (an ellipse offset's charts).
    pub fn cyclic_pieces(&self) -> bool {
        matches!(self, Self::Offset(_)) && self.conic().is_some()
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
        KeyValue::Surd(key) => {
            let [start, end] = range.map(ratio);
            Ok(match key.compare_ratio(&start, work)? {
                Ordering::Less => SketchLinePosition::BeforeStart,
                Ordering::Equal => SketchLinePosition::AtStart,
                Ordering::Greater => match key.compare_ratio(&end, work)? {
                    Ordering::Less => SketchLinePosition::Interior,
                    Ordering::Equal => SketchLinePosition::AtEnd,
                    Ordering::Greater => SketchLinePosition::AfterEnd,
                },
            })
        }
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
