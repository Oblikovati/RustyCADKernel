// SPDX-License-Identifier: GPL-2.0-only
//! Bounded adapters over existing exact captures; no geometry is synthesized or evaluated here.
use super::{failure::*, work::*};
use crate::sketch_region_geometry_capture::{CapturedMetricCurve, capture_raw_entity_tolerance};
use crate::sketch_region_source_conformance::CapturedSpaceRepresentation;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, ShapeIterator, edge, face},
};
use rustycad_tkg2d::geom2d::Geom2dBezierCurve;
use rustycad_tkg3d::{
    geom::GeomBezierCurve,
    top_abs::{Orientation, ShapeEnum},
};

pub(super) fn children<'a>(
    shape: &Shape,
    expected: ShapeEnum,
    maximum: usize,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, Vec<Shape>> {
    let mut rows = Vec::new();
    let mut iterator = ShapeIterator::new(shape, true, true);
    while iterator.more() {
        work.holds("bounded direct-child multiplicity", rows.len() < maximum)?;
        let child = iterator
            .value()
            .map_err(|cause| work.failure(TwoLoopRegionCause::Native(cause)))?;
        kind(child, expected, work)?;
        work.units("directed child operand", 1)?;
        rows.push(child.clone());
        iterator.next();
    }
    Ok(rows)
}
pub(super) fn kind<'a>(
    shape: &Shape,
    expected: ShapeEnum,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    let actual = shape
        .shape_type()
        .map_err(|cause| work.failure(TwoLoopRegionCause::Native(cause)))?;
    if actual != expected {
        return Err(work.failure(TwoLoopRegionCause::Kind { actual, expected }));
    }
    Ok(())
}
pub(super) fn reversed<'a>(
    orientation: Orientation,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, bool> {
    work.holds(
        "Forward/Reversed directed occurrence",
        matches!(orientation, Orientation::Forward | Orientation::Reversed),
    )?;
    Ok(orientation == Orientation::Reversed)
}
pub(super) fn space<'a>(
    shape: &Shape,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, CapturedSpaceRepresentation> {
    let actual = edge(shape).map_err(|cause| work.failure(TwoLoopRegionCause::Native(cause)))?;
    let raw = BRepTool::curve_with_location(&actual)
        .map_err(|cause| work.failure(TwoLoopRegionCause::Native(cause)))?;
    let handle = raw.curve.cast::<GeomBezierCurve>();
    let curve = handle.get().ok_or_else(|| {
        work.failure(TwoLoopRegionCause::Missing {
            hypothesis: "stored polynomial Bézier evaluator",
        })
    })?;
    work.holds("non-rational polynomial source", !curve.is_rational())?;
    poles(curve.nb_poles(), work)?;
    super::super::super::surface_charts::capture_bezier_edge(&actual)
        .map_err(|cause| work.failure(TwoLoopRegionCause::Capture(std::sync::Arc::new(cause))))
}
pub(super) fn pcurve<'a>(
    shape: &Shape,
    support: &Shape,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, CapturedMetricCurve<2>> {
    let actual = edge(shape).map_err(|cause| work.failure(TwoLoopRegionCause::Native(cause)))?;
    let support_face =
        face(support).map_err(|cause| work.failure(TwoLoopRegionCause::Native(cause)))?;
    let raw = BRepTool::curve_on_face(&actual, &support_face)
        .map_err(|cause| work.failure(TwoLoopRegionCause::Native(cause)))?;
    let handle = raw.pcurve.cast::<Geom2dBezierCurve>();
    let curve = handle.get().ok_or_else(|| {
        work.failure(TwoLoopRegionCause::Missing {
            hypothesis: "stored polynomial Bézier pcurve",
        })
    })?;
    work.holds("non-rational polynomial pcurve", !curve.is_rational())?;
    poles(curve.nb_poles(), work)?;
    let (captured, interval) = super::super::super::surface_charts::pcurve_on(shape, support)
        .map_err(|cause| work.failure(TwoLoopRegionCause::Capture(std::sync::Arc::new(cause))))?;
    work.holds("full native pcurve interval", interval == [0., 1.])?;
    Ok(captured)
}
fn poles<'a>(actual: i32, work: &mut TwoLoopWork<'_>) -> TwoLoopCheck<'a, ()> {
    let count = usize::try_from(actual).map_err(|_| {
        work.failure(TwoLoopRegionCause::Scalar {
            operand: "pole count",
            actual: f64::from(actual),
            expected: "nonnegative bounded polynomial pole count",
        })
    })?;
    if count > work.requested.maximum_poles {
        return Err(work.failure(TwoLoopRegionCause::Limit {
            operand: "polynomial poles",
            requested: work.requested.maximum_poles,
            measured: count,
        }));
    }
    work.units("captured polynomial pole", count)
}
pub(super) fn raw_tolerance<'a>(shape: &Shape, work: &TwoLoopWork<'_>) -> TwoLoopCheck<'a, f64> {
    let raw = capture_raw_entity_tolerance(shape, "Martensite two-loop domain metadata")
        .map_err(|cause| work.failure(TwoLoopRegionCause::Native(cause)))?;
    let raw = raw.ok_or_else(|| {
        work.failure(TwoLoopRegionCause::Missing {
            hypothesis: "stored entity tolerance",
        })
    })?;
    if !raw.is_finite() || raw < 0. {
        return Err(work.failure(TwoLoopRegionCause::Scalar {
            operand: "stored tolerance",
            actual: raw,
            expected: "finite nonnegative metadata; no tolerance acceptance implied",
        }));
    }
    Ok(raw)
}
