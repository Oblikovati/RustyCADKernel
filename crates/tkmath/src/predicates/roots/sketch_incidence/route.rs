// SPDX-License-Identifier: GPL-2.0-only
//! Pair routing: each source frame meets each canonical target through one exact
//! equation kind (ADR 0060, 0067, 0080).
use super::budget::Work;
use super::canonical::kind_name;
use super::conic_source;
use super::contact::TargetKey;
use super::failure::SketchIncidenceCause as Cause;
use super::implicit::LineImplicit;
use super::implicit_target::{self, ImplicitTarget};
use super::input::{SketchIncidenceCurve, SketchIncidenceSpline};
use super::offset_geometry::circle_radius;
use super::parametric_target;
use super::report::{SketchLineContact, SketchLineOverlap as Overlap};
use super::sector;
use super::source_frame::SourceFrame;
use super::{conic_pair, spans, spline_pair};

// The report keeps the request target; a periodic spline is certified unwrapped.
pub(super) fn curve_incidences(
    curve: &SketchIncidenceCurve,
    owner: TargetKey,
    frame: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<(Vec<SketchLineContact>, Vec<Overlap>), Cause> {
    Ok(match frame {
        SourceFrame::Spline(spline) => (
            parametric_source(&spline.pieces, curve, owner, frame, work)?,
            Vec::new(),
        ),
        SourceFrame::Line(line) if is_conic(curve) => {
            let pieces = std::slice::from_ref(&line.piece);
            (
                parametric_source(pieces, curve, owner, frame, work)?,
                Vec::new(),
            )
        }
        _ => implicit_source(curve, owner, frame, work)?,
    })
}
fn is_conic(curve: &SketchIncidenceCurve) -> bool {
    match curve {
        SketchIncidenceCurve::Circle { .. } | SketchIncidenceCurve::Ellipse { .. } => true,
        SketchIncidenceCurve::Offset(offset) => circle_radius(offset).is_some(),
        _ => false,
    }
}
// Line and spline sources substitute their pieces into the target's implicit equation.
fn parametric_source(
    pieces: &[spans::ParametricSpan],
    curve: &SketchIncidenceCurve,
    owner: TargetKey,
    frame: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<Vec<SketchLineContact>, Cause> {
    let implicit = match curve {
        SketchIncidenceCurve::Line { points, bounded } => ImplicitTarget::Line {
            implicit: LineImplicit::through(*points),
            bounded: *bounded,
        },
        conic if is_conic(conic) => conic_target(conic),
        SketchIncidenceCurve::Spline(spline) => return spline_target(spline, owner, frame, work),
        other => return Err(unpaired(frame, other)),
    };
    implicit_target::incidences(pieces, &implicit, owner, frame, work)
}
// Spline sources meet spline targets through span-pair elimination.
fn spline_target(
    spline: &SketchIncidenceSpline,
    owner: TargetKey,
    frame: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<Vec<SketchLineContact>, Cause> {
    let SourceFrame::Spline(source) = frame else {
        return Err(Cause::Invariant("spline pair without a spline source"));
    };
    let spans = spans::spline_spans(spline);
    let pair = spline_pair::SplinePair {
        source,
        spans: &spans,
        bounds: spline.bounds,
        owner,
    };
    spline_pair::incidences(&pair, frame, work)
}
fn conic_target(curve: &SketchIncidenceCurve) -> ImplicitTarget<'_> {
    let shape = conic_source::shape(curve);
    let radii = shape.radii;
    ImplicitTarget::Conic {
        implicit: shape.implicit,
        sector: shape
            .bounds
            .map(|bounds| sector::ArcBounds { bounds, radii }),
    }
}
// Line and conic sources substitute line and spline targets into their own equation.
fn implicit_source(
    curve: &SketchIncidenceCurve,
    owner: TargetKey,
    frame: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<(Vec<SketchLineContact>, Vec<Overlap>), Cause> {
    let (spans, bounds) = match curve {
        SketchIncidenceCurve::Line { points, bounded } => (
            vec![spans::line_span(*points, *bounded)],
            bounded.then_some([0., 1.]),
        ),
        SketchIncidenceCurve::Spline(spline) => (spans::spline_spans(spline), Some(spline.bounds)),
        conic if is_conic(conic) => return conic_source(conic, owner, frame, work),
        other => return Err(unpaired(frame, other)),
    };
    let request = parametric_target::ParametricRequest {
        spans,
        bounds,
        owner,
    };
    parametric_target::incidences(request, frame, work)
}
// Conic sources meet conic targets through their chart elimination.
fn conic_source(
    curve: &SketchIncidenceCurve,
    owner: TargetKey,
    frame: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<(Vec<SketchLineContact>, Vec<Overlap>), Cause> {
    let (SourceFrame::Conic(source), ImplicitTarget::Conic { implicit, sector }) =
        (frame, conic_target(curve))
    else {
        return Err(Cause::Invariant("conic pair without a conic source"));
    };
    let pair = conic_pair::ConicPair {
        source,
        target: &implicit,
        sector,
        owner,
    };
    conic_pair::incidences(&pair, frame, work)
}
// A pair without a certified equation in this version.
fn unpaired(frame: &SourceFrame, target: &SketchIncidenceCurve) -> Cause {
    Cause::OffsetPair {
        source: frame.kind_name(),
        target: kind_name(target),
    }
}
