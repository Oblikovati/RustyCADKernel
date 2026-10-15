// SPDX-License-Identifier: GPL-2.0-only
//! Canonical forms of admitted curves (ADR 0080): periodic splines are unwrapped
//! exactly and zero-distance offsets are their basis. Reports keep the request.
use super::failure::SketchIncidenceCause as Cause;
use super::input::{SketchIncidenceCurve, SketchIncidenceOffset};
use super::periodic::unwrap;
use std::borrow::Cow;

/// The admitted curve in canonical form; example: `let curve = canonical(&target.curve)?;`.
pub(super) fn canonical(
    curve: &SketchIncidenceCurve,
) -> Result<Cow<'_, SketchIncidenceCurve>, Cause> {
    match curve {
        SketchIncidenceCurve::PeriodicSpline(periodic) => {
            Ok(Cow::Owned(SketchIncidenceCurve::Spline(unwrap(periodic)?)))
        }
        SketchIncidenceCurve::Offset(offset) if offset.distance == 0. => canonical(&offset.basis),
        SketchIncidenceCurve::Offset(offset) => Ok(match canonical(&offset.basis)? {
            Cow::Borrowed(_) => Cow::Borrowed(curve),
            Cow::Owned(basis) => Cow::Owned(SketchIncidenceCurve::Offset(SketchIncidenceOffset {
                basis: Box::new(basis),
                distance: offset.distance,
            })),
        }),
        other => Ok(Cow::Borrowed(other)),
    }
}
/// Representation name used by named refusals, e.g. `"offset spline"`.
pub(super) fn kind_name(curve: &SketchIncidenceCurve) -> &'static str {
    match curve {
        SketchIncidenceCurve::Line { .. } => "line",
        SketchIncidenceCurve::Circle { .. } => "circle",
        SketchIncidenceCurve::Ellipse { .. } => "ellipse",
        SketchIncidenceCurve::Spline(_) | SketchIncidenceCurve::PeriodicSpline(_) => "spline",
        SketchIncidenceCurve::Offset(offset) => match offset.basis.as_ref() {
            SketchIncidenceCurve::Line { .. } => "offset line",
            SketchIncidenceCurve::Circle { .. } => "offset circle",
            SketchIncidenceCurve::Ellipse { .. } => "offset ellipse",
            _ => "offset spline",
        },
    }
}
