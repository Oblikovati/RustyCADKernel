// SPDX-License-Identifier: GPL-2.0-only
//! Admission checks in input order; the first offending value is reported unchanged.
use super::failure::{SketchIncidenceCause as Cause, SketchIncidenceFailure, SketchIncidenceStage};
use super::input::*;
use crate::predicates::{OrientationSign, exact_dot, exact_point, orient2d};
use num_bigint::Sign;

type Admission = Result<(), Cause>;

pub(super) fn options(options: SketchIncidenceOptions) -> Result<(), SketchIncidenceFailure> {
    let checked = if options.version != 1 {
        Err(Cause::OptionsVersion(options.version))
    } else if !options.max_world_error.is_finite() || options.max_world_error <= 0. {
        Err(Cause::WorldErrorLimit(options.max_world_error))
    } else {
        Ok(())
    };
    checked.map_err(input_failure)
}
pub(super) fn source(source: &SketchIncidenceSource) -> Result<(), SketchIncidenceFailure> {
    distinct_points("source.points", source.points).map_err(input_failure)
}
pub(super) fn target(target: &SketchIncidenceTarget) -> Result<(), SketchIncidenceFailure> {
    curve(&target.curve)
        .map_err(input_failure)
        .map_err(|failure| failure.for_target(target.id))
}
fn input_failure(cause: Cause) -> SketchIncidenceFailure {
    SketchIncidenceFailure::new(SketchIncidenceStage::Input, cause)
}
fn curve(curve: &SketchIncidenceCurve) -> Admission {
    match curve {
        SketchIncidenceCurve::Line { points, .. } => distinct_points("line.points", *points),
        SketchIncidenceCurve::Circle {
            center,
            radius,
            sector,
        } => conic(*center, &[*radius], [1., 0.], sector.as_ref()),
        SketchIncidenceCurve::Ellipse {
            center,
            major_axis,
            radii,
            sector,
        } => conic(*center, radii, *major_axis, sector.as_ref()),
        SketchIncidenceCurve::Spline(spline) => super::spline_admission::spline(spline),
    }
}
pub(super) fn finite(field: &'static str, values: &[f64]) -> Admission {
    match values.iter().position(|value| !value.is_finite()) {
        Some(index) => Err(Cause::NonFinite {
            field,
            index,
            value: values[index],
        }),
        None => Ok(()),
    }
}
pub(super) fn positive(field: &'static str, values: &[f64]) -> Admission {
    finite(field, values)?;
    match values.iter().position(|value| *value <= 0.) {
        Some(index) => Err(Cause::NonPositive {
            field,
            index,
            value: values[index],
        }),
        None => Ok(()),
    }
}
fn distinct_points(field: &'static str, points: [[f64; 2]; 2]) -> Admission {
    finite(
        field,
        &[points[0][0], points[0][1], points[1][0], points[1][1]],
    )?;
    // Binary64 equality is exact; signed zeros denote the same coordinate.
    if points[0] == points[1] {
        return Err(Cause::Degenerate { field });
    }
    Ok(())
}
fn nonzero(field: &'static str, vector: [f64; 2]) -> Admission {
    finite(field, &vector)?;
    if vector[0] == 0. && vector[1] == 0. {
        return Err(Cause::Degenerate { field });
    }
    Ok(())
}
fn conic(
    center: [f64; 2],
    radii: &[f64],
    axis: [f64; 2],
    sector: Option<&SketchConicSector>,
) -> Admission {
    finite("conic.center", &center)?;
    nonzero("conic.major_axis", axis)?;
    positive("conic.radii", radii)?;
    sector.map_or(Ok(()), conic_sector)
}
fn conic_sector(sector: &SketchConicSector) -> Admission {
    nonzero("sector.start", sector.start)?;
    nonzero("sector.end", sector.end)?;
    let turn = orient2d([0., 0.], sector.start, sector.end).expect("finite sector directions");
    let start = exact_point(sector.start, 0).expect("finite sector start");
    let end = exact_point(sector.end, 1).expect("finite sector end");
    if turn == OrientationSign::Zero && exact_dot(&start, &end).sign() == Sign::Plus {
        return Err(Cause::CoincidentSector {
            start: sector.start,
            end: sector.end,
        });
    }
    Ok(())
}
