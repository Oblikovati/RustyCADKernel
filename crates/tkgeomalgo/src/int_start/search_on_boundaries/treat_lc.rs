// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of TreatLC, IsRegularity and the two IsDegenerated of
// ModelingAlgorithms/TKGeomAlgo/IntStart/IntStart_SearchOnBoundaries.gxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The tests that read the edge behind an arc (a line tangent to a cylinder, the
//! regularity of the edge) and the degeneracy tests of the exact intersection.
use crate::int_start::tools::{TheArc, ThePathPoint, TheTopolTool};
use crate::int_surf::IntSurfQuadric;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::extrema::{ExtCC, POnCurv};
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Cylinder3, Line3, Point3};

/// `IsDegenerated(theCurve)`: a circle of radius at most Precision::Confusion.
pub(super) fn is_degenerated_curve(curve: &Adaptor3dCurveOnSurface) -> Result<bool, Failure> {
    if curve.get_type()? != CurveType::Circle {
        return Ok(false);
    }
    Ok(curve.circle()?.radius() <= precision::confusion())
}

/// `IsDegenerated(theQuadric)`: a cone whose semi-angle is below 0.02 or above 1.55.
pub(super) fn is_degenerated_quadric(quadric: &IntSurfQuadric) -> Result<bool, Failure> {
    if quadric.type_quadric() != SurfaceType::Cone {
        return Ok(false);
    }
    let semi_angle: f64 = quadric.cone()?.semi_angle().abs();
    Ok(semi_angle < 0.02 || semi_angle > 1.55)
}

/// `IsRegularity(A, aDomain)`: the edge of the arc has a regularity; false without an
/// edge.
pub(super) fn is_regularity<D: TheTopolTool + ?Sized>(domain: &D) -> Result<bool, Failure> {
    let Some(edge) = domain.edge() else {
        return Ok(false);
    };
    BRepTool::has_continuity(&edge)
}

/// `TreatLC(A, aDomain, aQuadric, TolBoundary, pnt)`: a straight edge tangent to a
/// cylinder gives one solution point, its point nearest to the axis (PKV, 23 March 2001).
/// Returns 0 after appending that point, 1 otherwise.
pub(super) fn treat_lc<D: TheTopolTool + ?Sized, P: ThePathPoint>(
    arc: &TheArc,
    domain: &D,
    quadric: &IntSurfQuadric,
    tol_boundary: f64,
    pnt: &mut Vec<P>,
) -> Result<i32, Failure> {
    let Some(edge) = domain.edge() else {
        return Ok(1);
    };
    if BRepTool::degenerated(&edge)?
        || BRepAdaptorCurve::with_edge(&edge)?.get_type()? != CurveType::Line
        || quadric.type_quadric() != SurfaceType::Cylinder
    {
        return Ok(1);
    }
    let Some((pedge, uedge)) = tangent_point(&edge, &quadric.cylinder()?, tol_boundary)? else {
        return Ok(1);
    };
    // Do not wonder: it was done as in PointProcess.
    let mut tol: f64 = 1000. * tol_boundary;
    if tol > 0.001 {
        tol = 0.001;
    }
    let mut ptsol: P = P::default();
    ptsol.set_value_on_arc(pedge, tol, arc, uedge);
    pnt.push(ptsol);
    Ok(0)
}

/// The point of the edge nearest to the cylinder axis and its parameter, when that point
/// is on the cylinder within `tol_boundary` (relative) and inside the edge range.
fn tangent_point(
    edge: &Edge,
    cyl: &Cylinder3,
    tol_boundary: f64,
) -> Result<Option<(Point3, f64)>, Failure> {
    let (c_edge, f, l) = BRepTool::curve(edge)?;
    let c_edge_ref: &dyn GeomCurve = c_edge
        .get()
        .ok_or_else(|| errors::null_object("IntStart_SearchOnBoundaries::TreatLC: no 3D curve"))?;
    let axis_line = GeomLine::from_lin(Line3::from_axis(cyl.axis()));
    let r: f64 = cyl.radius();
    let u1: (f64, f64) = (axis_line.first_parameter(), axis_line.last_parameter());
    let u2: (f64, f64) = (c_edge_ref.first_parameter(), c_edge_ref.last_parameter());
    let c_axis: Handle<dyn GeomCurve> = Handle::new(axis_line).cast();
    let c1 = GeomAdaptorCurve::with_curve(&c_axis)?;
    let c2 = GeomAdaptorCurve::with_curve(&c_edge)?;
    let tol: f64 = precision::p_confusion();
    let ext_cc = ExtCC::with_ranges(&c1 as &dyn Adaptor3dCurve, &c2, u1, u2, tol, tol)?;
    if ext_cc.nb_ext()? != 1 {
        return Ok(None);
    }
    let (pc1, pc2): (POnCurv, POnCurv) = ext_cc.points(1)?;
    let p1: Point3 = *pc1.value();
    let pedge: Point3 = *pc2.value();
    let uedge: f64 = pc2.parameter();
    let dist: f64 = pedge.distance(&p1);
    let r_rel: f64 = (dist - r).abs() / r;
    if r_rel > tol_boundary || uedge < (f + tol_boundary) || uedge > (l - tol_boundary) {
        return Ok(None);
    }
    Ok(Some((pedge, uedge)))
}
