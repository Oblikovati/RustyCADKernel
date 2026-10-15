// SPDX-License-Identifier: GPL-2.0-only
//! Pair routing: each source frame meets each canonical target through one exact
//! equation kind (ADR 0060, 0067, 0080).
//!
//! A parametric source (line, spline, offset pieces) is substituted into an
//! implicit target (line, conic, line offset); an implicit source (line, line
//! offset, conic) is composed with a parametric target (rational or offset
//! pieces). Spline pairs and conic pairs use their eliminations. An offset of an
//! ellipse or spline has no implicit form, so it meets only implicit partners;
//! other pairs are refused by name (`OffsetPair`).
use super::budget::Work;
use super::canonical::kind_name;
use super::contact::TargetKey;
use super::coordinate::Coordinate;
use super::failure::SketchIncidenceCause as Cause;
use super::implicit_target::{self, ImplicitTarget};
use super::input::{SketchIncidenceCurve, SketchIncidenceSpline};
use super::parametric_target;
use super::report::{SketchLineContact, SketchLineOverlap as Overlap};
use super::source_frame::SourceFrame;
use super::spans::ParametricSpan;
use super::target_form::{TargetForm, conic_target};
use super::{conic_pair, spans, spline_pair};
use std::slice::from_ref;

type Incidences = (Vec<SketchLineContact>, Vec<Overlap>);

/// One source frame and one canonical target.
struct Pair<'a> {
    curve: &'a SketchIncidenceCurve,
    form: TargetForm<'a>,
    owner: TargetKey,
    frame: &'a SourceFrame,
}
// The report keeps the request target; a periodic spline is certified unwrapped.
pub(super) fn curve_incidences(
    curve: &SketchIncidenceCurve,
    owner: TargetKey,
    frame: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<Incidences, Cause> {
    let form = TargetForm::of(curve);
    let pair = Pair {
        curve,
        form,
        owner,
        frame,
    };
    match frame {
        SourceFrame::Spline(spline) => pair.parametric(&spline.pieces, work),
        SourceFrame::Offset(offset) => pair.parametric(&offset.pieces, work),
        SourceFrame::Line(line) if pair.line_meets_implicitly() => {
            pair.parametric(from_ref(&line.piece), work)
        }
        SourceFrame::OffsetLine(line) if pair.line_meets_implicitly() => {
            pair.parametric(from_ref(&line.piece), work)
        }
        _ => pair.implicit(work),
    }
}
impl Pair<'_> {
    // A line (or line offset) source meets conics and line offsets as a piece.
    fn line_meets_implicitly(&self) -> bool {
        matches!(self.form, TargetForm::Conic(_) | TargetForm::OffsetLine(..))
    }
    // Parametric source pieces substituted into the target's implicit equation; a
    // spline source meets a spline target through span-pair elimination.
    fn parametric<T: Coordinate>(
        &self,
        pieces: &[ParametricSpan<T>],
        work: &mut Work<'_>,
    ) -> Result<Incidences, Cause> {
        let contacts = match (self.form.implicit(), &self.form, self.frame) {
            (Some(implicit), ..) => {
                implicit_target::incidences(pieces, &implicit, self.owner, self.frame, work)?
            }
            (None, TargetForm::Spline(spline), SourceFrame::Spline(_)) => {
                self.spline_target(spline, work)?
            }
            _ => return Err(self.unpaired()),
        };
        Ok((contacts, Vec::new()))
    }
    // Spline sources meet spline targets through span-pair elimination.
    fn spline_target(
        &self,
        spline: &SketchIncidenceSpline,
        work: &mut Work<'_>,
    ) -> Result<Vec<SketchLineContact>, Cause> {
        let SourceFrame::Spline(source) = self.frame else {
            return Err(Cause::Invariant("spline pair without a spline source"));
        };
        let spans = spans::spline_spans(spline);
        let pair = spline_pair::SplinePair {
            source,
            spans: &spans,
            bounds: spline.bounds,
            owner: self.owner,
        };
        spline_pair::incidences(&pair, self.frame, work)
    }
    // Line, line offset and conic sources compose their equation with target pieces.
    fn implicit(&self, work: &mut Work<'_>) -> Result<Incidences, Cause> {
        if let TargetForm::Conic(conic) = self.form {
            return self.conic_source(conic, work);
        }
        if let Some(request) = self.form.rational_request(self.owner) {
            return parametric_target::incidences(request, self.frame, work);
        }
        match self.form.offset_request(self.owner) {
            Some(request) => parametric_target::incidences(request, self.frame, work),
            None => Err(self.unpaired()),
        }
    }
    // Conic sources meet conic targets through their chart elimination.
    fn conic_source(
        &self,
        curve: &SketchIncidenceCurve,
        work: &mut Work<'_>,
    ) -> Result<Incidences, Cause> {
        let (SourceFrame::Conic(source), ImplicitTarget::Conic { implicit, sector }) =
            (self.frame, conic_target(curve))
        else {
            return Err(Cause::Invariant("conic pair without a conic source"));
        };
        let pair = conic_pair::ConicPair {
            source,
            target: &implicit,
            sector,
            owner: self.owner,
        };
        conic_pair::incidences(&pair, self.frame, work)
    }
    // A pair without a certified equation in this version.
    fn unpaired(&self) -> Cause {
        Cause::OffsetPair {
            source: self.frame.kind_name(),
            target: kind_name(self.curve),
        }
    }
}
