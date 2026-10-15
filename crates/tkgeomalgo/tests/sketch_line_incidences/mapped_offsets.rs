// SPDX-License-Identifier: GPL-2.0-only
//! Mapped Geom2d offset curves and periodic B-splines for the native comparison
//! (ADR 0080), built as `native_oracle.cpp` builds them: the offset of the
//! untrimmed basis, trimmed like the basis; arcs travel in their sector sense.
use super::corpus::{Curve, mapped_line, mapped_target};
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dCircle, Geom2dOffsetCurve, Geom2dTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};
use rustycad_tkmath::predicates::roots::sketch_incidence::*;

/// Offsets and periodic splines; other curves go through `mapped_target`.
pub fn mapped_extension(curve: &SketchIncidenceCurve) -> Curve {
    match curve {
        SketchIncidenceCurve::PeriodicSpline(spline) => mapped_periodic(spline),
        SketchIncidenceCurve::Offset(offset) => mapped_offset(offset),
        other => panic!("curve {other:?}; expected an offset or a periodic spline"),
    }
}
fn mapped_offset(offset: &SketchIncidenceOffset) -> Curve {
    let (basis, range) = untrimmed_basis(&offset.basis);
    let whole: Curve =
        Handle::new(Geom2dOffsetCurve::new(&basis, offset.distance, false).unwrap()).cast();
    match range {
        Some([first, last]) => trimmed(&whole, first, last),
        None => whole,
    }
}
// The basis without its trim, and the trimmed parameter range.
fn untrimmed_basis(basis: &SketchIncidenceCurve) -> (Curve, Option<[f64; 2]>) {
    match basis {
        SketchIncidenceCurve::Line {
            points,
            bounded: true,
        } => {
            let length = (points[1][0] - points[0][0]).hypot(points[1][1] - points[0][1]);
            (mapped_line(*points), Some([0., length]))
        }
        SketchIncidenceCurve::Circle {
            center,
            radius,
            sector: Some(SketchConicBounds::Rays(sector)),
        } => directed_arc(*center, *radius, sector),
        other => (mapped_target(other), None),
    }
}
// A clockwise arc lies on a clockwise (indirect) circle, so its offset side is kept.
fn directed_arc(center: [f64; 2], radius: f64, sector: &SketchConicSector) -> (Curve, Option<[f64; 2]>) {
    let axis = Axis2d::new(Point2::new(center[0], center[1]), Direction2::new(1., 0.).unwrap());
    let ccw = sector.counterclockwise;
    let circle: Curve = Handle::new(Geom2dCircle::from_axis(axis, radius, ccw).unwrap()).cast();
    let turn = if ccw { 1. } else { -1. };
    let angle = |value: [f64; 2]| turn * value[1].atan2(value[0]);
    let (first, mut last) = (angle(sector.start), angle(sector.end));
    if last <= first {
        last += std::f64::consts::TAU;
    }
    (circle, Some([first, last]))
}
fn trimmed(curve: &Curve, first: f64, last: f64) -> Curve {
    Handle::new(Geom2dTrimmedCurve::new(curve, first, last, true, false).unwrap()).cast()
}
fn mapped_periodic(spline: &SketchPeriodicSpline) -> Curve {
    let poles: Vec<Point2> = spline.poles.iter().map(|p| Point2::new(p[0], p[1])).collect();
    let multiplicities: Vec<i32> = spline.multiplicities.iter().map(|&m| m as i32).collect();
    let curve: Curve = Handle::new(
        Geom2dBSplineCurve::new(
            &Array1::from_shared(&poles, 1),
            &Array1::from_shared(&spline.knots, 1),
            &Array1::from_shared(&multiplicities, 1),
            spline.degree as i32,
            true,
        )
        .unwrap(),
    )
    .cast();
    let full = [spline.knots[0], spline.knots[spline.knots.len() - 1]];
    if spline.bounds == full {
        return curve;
    }
    trimmed(&curve, spline.bounds[0], spline.bounds[1])
}
