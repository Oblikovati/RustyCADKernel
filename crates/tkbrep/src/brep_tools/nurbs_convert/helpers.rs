// Created on: 1996-07-12
// Created by: Stagiaire Mary FABIEN
// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of the anonymous-namespace helpers and the static IsConvert of
// ModelingData/TKBRep/BRepTools/BRepTools_NurbsConvertModification.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The lookups and projections shared by the members of the NURBS conversion.
use super::ConversionMap;
use crate::bintools::handle_table::HandleKey;
use crate::brep::BRepTool;
use crate::brep::access::edge_tshape;
use crate::brep::update_curves::as_gcurve;
use crate::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomBezierSurface, GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::extrema::{ExtPC2d, GenLocateExtPS, LocateExtPC, LocateExtPC2d};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{Point2, Point3};

/// `GeomLib_ChangeUBounds` or `GeomLib_ChangeVBounds`: the knots of one direction
/// reparametrised onto `[new1, new2]`.
pub(super) fn change_bounds(
    surface: &GeomBSplineSurface,
    u_direction: bool,
    (new1, new2): (f64, f64),
) -> Result<(), Failure> {
    let mut knots = if u_direction {
        surface.u_knots()
    } else {
        surface.v_knots()
    };
    bsplclib::reparametrize(new1, new2, &mut knots)?;
    if u_direction {
        surface.set_u_knots(&knots)
    } else {
        surface.set_v_knots(&knots)
    }
}

/// The conversion of `original` in `map`, if any.
fn converted(
    map: &ConversionMap,
    original: Handle<StandardTransient>,
) -> Option<Handle<StandardTransient>> {
    map.find_from_key(&HandleKey(original)).ok().cloned()
}

/// `newCurve(theMap, theEdge, theFirst, theLast)`: the converted 3D curve of the edge
/// moved by the edge location (null when not converted), with the edge range.
pub(super) fn new_curve_of(
    map: &ConversionMap,
    edge: &Edge,
) -> Result<(Handle<dyn GeomCurve>, f64, f64), Failure> {
    let found = BRepTool::curve_with_location(edge)?;
    let range = (found.first, found.last);
    if found.curve.is_null() {
        return Ok((Handle::default(), range.0, range.1));
    }
    let Some(new_curve) = converted(map, found.curve.cast()) else {
        return Ok((Handle::default(), range.0, range.1));
    };
    let new_curve = new_curve.cast::<dyn GeomCurve>();
    let Some(stored) = new_curve.get() else {
        return Ok((Handle::default(), range.0, range.1));
    };
    let moved = stored.transformed(found.location.transformation())?.cast();
    Ok((moved, range.0, range.1))
}

/// `newCurve(theMap, theEdge, theFace, theFirst, theLast)`: the converted p-curve of the
/// edge on the face (null when not converted), with the p-curve range.
pub(super) fn new_pcurve_of(
    map: &ConversionMap,
    edge: &Edge,
    face: &Face,
) -> Result<(Handle<dyn Geom2dCurve>, f64, f64), Failure> {
    let found = BRepTool::curve_on_face(edge, face)?;
    if found.pcurve.is_null() {
        return Ok((Handle::default(), found.first, found.last));
    }
    let new_pcurve = converted(map, found.pcurve.cast())
        .map(|curve| curve.cast::<dyn Geom2dCurve>())
        .unwrap_or_default();
    Ok((new_pcurve, found.first, found.last))
}

/// `newSurface(theMap, theFace)`: the converted surface of the face moved by the face
/// location, null when not converted.
pub(super) fn new_surface_of(
    map: &ConversionMap,
    face: &Face,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let (surface, location) = BRepTool::surface_with_location(face)?;
    if surface.is_null() {
        return Ok(Handle::default());
    }
    let Some(new_surface) = converted(map, surface.cast()) else {
        return Ok(Handle::default());
    };
    let new_surface = new_surface.cast::<dyn GeomSurface>();
    let Some(stored) = new_surface.get() else {
        return Ok(Handle::default());
    };
    Ok(stored.transformed(location.transformation())?.cast())
}

/// `newParameter(thePoint, theCurve, ...)`: the local projection of the point on the
/// curve from `parameter` within `[first, last]`; true, with `parameter` updated, when
/// it is nearer than `tolerance`.
pub(super) fn new_parameter_on_curve(
    point: Point3,
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    tolerance: f64,
    parameter: &mut f64,
) -> Result<bool, Failure> {
    let adaptor = GeomAdaptorCurve::with_curve(curve)?;
    let projection =
        LocateExtPC::with_range(point, &adaptor, *parameter, range, precision::p_confusion())?;
    if projection.is_done() && projection.square_distance()? < tolerance * tolerance {
        *parameter = projection.point()?.parameter();
        return Ok(true);
    }
    Ok(false)
}

/// `newParameter(theUV, theCurve2d, ...)`: the local projection of the UV point on the
/// p-curve from `parameter`; when it fails, the nearest extremum of the global
/// projection on `[first, last]` (bugs/modalg_7/bug28722). True, with `parameter`
/// updated, when nearer than `tolerance`.
pub(super) fn new_parameter_on_pcurve(
    uv: Point2,
    pcurve: &Handle<dyn Geom2dCurve>,
    (first, last): (f64, f64),
    tolerance: f64,
    parameter: &mut f64,
) -> Result<bool, Failure> {
    let adaptor = Geom2dAdaptorCurve::with_curve(pcurve)?;
    let local = LocateExtPC2d::with_curve(uv, &adaptor, *parameter, precision::p_confusion())?;
    if local.is_done() {
        if local.square_distance()? < tolerance * tolerance {
            *parameter = local.point()?.parameter();
            return Ok(true);
        }
        return Ok(false);
    }
    let global = ExtPC2d::with_range(uv, &adaptor, first, last, 1.0e-10)?;
    if !global.is_done() {
        return Ok(false);
    }
    let (mut min_index, mut min_square) = (0, precision::infinite());
    for index in 1..=global.nb_ext()? {
        if global.square_distance(index)? < min_square {
            min_square = global.square_distance(index)?;
            min_index = index;
        }
    }
    if min_square < tolerance * tolerance {
        *parameter = global.point(min_index)?.parameter();
        return Ok(true);
    }
    Ok(false)
}

/// `newUV(thePoint, theSurf, theTol, theUV)`: the local projection of the point on the
/// surface from `uv`; true, with `uv` updated, when nearer than `tolerance`.
pub(super) fn new_uv(
    point: Point3,
    surface: &Handle<dyn GeomSurface>,
    tolerance: f64,
    uv: &mut Point2,
) -> Result<bool, Failure> {
    let adaptor = GeomAdaptorSurface::with_surface(surface)?;
    let mut projection =
        GenLocateExtPS::new(&adaptor, precision::p_confusion(), precision::p_confusion());
    projection.perform(point, uv.x(), uv.y(), false)?;
    if projection.is_done() && projection.square_distance()? < tolerance * tolerance {
        let (u, v) = projection.point()?.parameter();
        *uv = Point2::new(u, v);
        return Ok(true);
    }
    Ok(false)
}

/// The static `IsConvert(E)`: some p-curve of the edge lies on a surface that is not a
/// B-spline or Bezier surface, or is not itself a B-spline or Bezier curve.
pub(super) fn is_convert(edge: &Edge) -> Result<bool, Failure> {
    let tedge = edge_tshape(edge, "BRepTools_NurbsConvertModification::IsConvert")?;
    for representation in tedge.curves() {
        let gcurve = as_gcurve(&representation);
        let Some(gcurve) = gcurve.get().filter(|g| g.is_curve_on_surface()) else {
            continue;
        };
        let surface = gcurve.surface()?;
        let pcurve = gcurve.pcurve()?;
        let spline_surface = surface.cast::<GeomBSplineSurface>().is_some()
            || surface.cast::<GeomBezierSurface>().is_some();
        let spline_pcurve = pcurve.cast::<Geom2dBSplineCurve>().is_some()
            || pcurve.cast::<Geom2dBezierCurve>().is_some();
        if !spline_surface || !spline_pcurve {
            return Ok(true);
        }
    }
    Ok(false)
}
