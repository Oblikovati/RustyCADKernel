// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of constantSquareDistance of ModelingData/TKGeomBase/ExtremaPC2d/
// ExtremaPC2d_GridEvaluator.hxx and ExtremaPC/ExtremaPC_GridEvaluator.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `constantSquareDistance`: the grid evaluators report infinite solutions only when
//! the geometry proves the distance constant. The 2D and 3D evaluators prove it
//! differently: the 2D one looks through a Geom2dAdaptor_Curve at the Geom2d classes
//! (including offsets and rational Bezier curves), the 3D one only at the adaptor.
//! A failure anywhere is caught natively and means "not proven" (`Err` here).
use super::rational_bezier::rational_bezier_square_distance;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, Transient};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dConic, Geom2dCurve,
    Geom2dOffsetCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBezierCurve};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Vector3};
use rustycad_tkmath::math_utils::Domain1D;

/// The 2D `constantSquareDistance`.
pub(super) fn constant_square_distance_2d(
    curve: &dyn Adaptor2dCurve2d,
    point: Point2,
    domain: &Domain1D,
) -> Result<Option<f64>, Failure> {
    match curve.get_type()? {
        CurveType::Circle => {
            let circle = curve.circle()?;
            if point.square_distance(circle.location()) == 0.0 {
                return Ok(Some(circle.radius() * circle.radius()));
            }
        }
        CurveType::Ellipse => {
            let ellipse = curve.ellipse()?;
            if ellipse.major_radius() == ellipse.minor_radius()
                && point.square_distance(ellipse.location()) == 0.0
            {
                return Ok(Some(ellipse.major_radius() * ellipse.major_radius()));
            }
        }
        _ => {}
    }
    let upcast: &dyn Transient = curve;
    let Some(adaptor) = Geom2dAdaptorCurve::project_from(upcast) else {
        return Ok(None);
    };
    let geometry: Handle<dyn Geom2dCurve> = adaptor.curve();
    if geometry.is_null() {
        return Ok(None);
    }
    geometry_square_distance_2d(curve, geometry, point, domain)
}

// The Geom2d part: an offset looks at its basis, then circles and Bezier/BSpline
// curves with coincident poles are constant.
fn geometry_square_distance_2d(
    curve: &dyn Adaptor2dCurve2d,
    mut geometry: Handle<dyn Geom2dCurve>,
    point: Point2,
    domain: &Domain1D,
) -> Result<Option<f64>, Failure> {
    let offset: Handle<Geom2dOffsetCurve> = geometry.cast();
    let is_offset: bool = !offset.is_null();
    if let Some(offset) = offset.get() {
        geometry = offset.basis_curve();
    }
    let at_first =
        || -> Result<f64, Failure> { Ok(point.square_distance(curve.value(domain.min)?)) };
    let circle: Handle<Geom2dCircle> = geometry.cast();
    if let Some(circle) = circle.get()
        && point.square_distance(circle.location()) == 0.0
    {
        return Ok(Some(at_first()?));
    }
    let bezier: Handle<Geom2dBezierCurve> = geometry.cast();
    let bspline: Handle<Geom2dBSplineCurve> = geometry.cast();
    if let Some(distance) = rational_bezier_square_distance(&bezier, point)? {
        return Ok(Some(if is_offset { at_first()? } else { distance }));
    }
    let poles: Option<Vec<Point2>> = match (bezier.get(), bspline.get()) {
        (Some(bezier), _) => Some(bezier.poles().iter().copied().collect()),
        (None, Some(bspline)) => Some(bspline.poles().iter().copied().collect()),
        (None, None) => None,
    };
    let Some(reference) = poles.as_deref().and_then(coincident_poles_2d) else {
        return Ok(None);
    };
    Ok(Some(if is_offset {
        at_first()?
    } else {
        point.square_distance(reference)
    }))
}

// The common pole when every pole is exactly the first one.
fn coincident_poles_2d(poles: &[Point2]) -> Option<Point2> {
    let reference: Point2 = *poles.first()?;
    poles[1..]
        .iter()
        .all(|pole| pole.square_distance(reference) == 0.0)
        .then_some(reference)
}

/// The 3D `constantSquareDistance`.
pub(super) fn constant_square_distance_3d(
    curve: &dyn Adaptor3dCurve,
    point: Point3,
    domain: &Domain1D,
) -> Result<Option<f64>, Failure> {
    let kind: CurveType = curve.get_type()?;
    let on_axis = |center: Point3, axis: Direction3| -> bool {
        let center_to_point: Vector3 = Vector3::between(center, point);
        center_to_point
            .crossed(Vector3::from(axis))
            .square_magnitude()
            == 0.0
    };
    if kind == CurveType::Circle {
        let circle = curve.circle()?;
        if on_axis(circle.location(), circle.axis().direction()) {
            return Ok(Some(point.square_distance(&curve.value(domain.min)?)));
        }
    } else if kind == CurveType::Ellipse {
        let ellipse = curve.ellipse()?;
        if ellipse.major_radius() == ellipse.minor_radius()
            && on_axis(ellipse.location(), ellipse.axis().direction())
        {
            return Ok(Some(point.square_distance(&curve.value(domain.min)?)));
        }
    }
    let poles: Option<Vec<Point3>> = spline_poles_3d(curve, kind)?;
    let Some(reference) = poles.as_deref().and_then(coincident_poles_3d) else {
        return Ok(None);
    };
    Ok(Some(point.square_distance(&reference)))
}

// The poles of the adaptor's Bezier or BSpline curve, when it is one.
fn spline_poles_3d(
    curve: &dyn Adaptor3dCurve,
    kind: CurveType,
) -> Result<Option<Vec<Point3>>, Failure> {
    let bezier: Handle<GeomBezierCurve> = match kind {
        CurveType::BezierCurve => curve.bezier()?,
        _ => Handle::default(),
    };
    let bspline: Handle<GeomBSplineCurve> = match kind {
        CurveType::BSplineCurve => curve.bspline()?,
        _ => Handle::default(),
    };
    Ok(match (bezier.get(), bspline.get()) {
        (Some(bezier), _) => Some(bezier.poles().iter().copied().collect()),
        (None, Some(bspline)) => Some(bspline.poles().iter().copied().collect()),
        (None, None) => None,
    })
}

// The common pole when every pole is exactly the first one.
fn coincident_poles_3d(poles: &[Point3]) -> Option<Point3> {
    let reference: Point3 = *poles.first()?;
    poles[1..]
        .iter()
        .all(|pole| pole.square_distance(&reference) == 0.0)
        .then_some(reference)
}
