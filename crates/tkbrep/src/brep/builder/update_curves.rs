// Created on: 1991-07-02
// Created by: Remi LEQUETTE
// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of ModelingData/TKBRep/BRep/BRep_Builder.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The static `UpdateCurves` and `UpdatePoints` helpers of BRep_Builder.cxx: they edit
//! the representation list of an edge or a vertex in place.
use crate::brep::{
    BRepCurve3D, BRepCurveOn2Surfaces, BRepCurveOnClosedSurface, BRepCurveOnSurface,
    BRepCurveRepresentation, BRepGCurve, BRepPointOnCurve, BRepPointOnCurveOnSurface,
    BRepPointOnSurface, BRepPointRepresentation,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::toploc::Location;

/// An edge's list of curve representations.
pub type CurveList = Vec<Handle<dyn BRepCurveRepresentation>>;
/// A vertex's list of point representations.
pub type PointList = Vec<Handle<dyn BRepPointRepresentation>>;

/// `occ::down_cast<BRep_GCurve>`: the parametric view of a representation, if any.
pub fn as_gcurve(representation: &Handle<dyn BRepCurveRepresentation>) -> Handle<dyn BRepGCurve> {
    representation.cast::<dyn BRepGCurve>()
}

/// `UpdateCurves(lcr, C, L)`: sets the 3D curve and location of the first 3D curve
/// representation, or appends a new one.
///
/// The native loop reassigns `GC` at every item, so after an unsuccessful search `GC`
/// is the cast of the last item: the new representation takes the range of the last
/// parametric curve read only when the list ends with a parametric curve.
pub fn update_curve3d(
    list: &mut CurveList,
    curve: &Handle<dyn GeomCurve>,
    location: &Location,
) -> Result<(), Failure> {
    let mut range = (0.0, 0.0);
    let mut last_is_gcurve = false;
    for representation in list.iter() {
        let gcurve = as_gcurve(representation);
        last_is_gcurve = false;
        let Some(gcurve) = gcurve.get() else { continue };
        last_is_gcurve = true;
        range = gcurve.range();
        if gcurve.is_curve3d() {
            gcurve.set_curve3d(curve)?;
            gcurve.set_location(location);
            return Ok(());
        }
    }
    let curve3d = BRepCurve3D::new(curve, location);
    if last_is_gcurve {
        curve3d.set_range(range.0, range.1)?;
    }
    list.push(Handle::new(curve3d).into_cast());
    Ok(())
}

/// The range of the 3D curve and the removal of every curve on (`surface`,
/// `location`), the search loop of the single p-curve `UpdateCurves`.
fn remove_curves_on(
    list: &mut CurveList,
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
) -> (f64, f64) {
    let mut range = (-precision::infinite(), precision::infinite());
    let mut index = 0;
    while index < list.len() {
        let gcurve = as_gcurve(&list[index]);
        let Some(gcurve) = gcurve.get() else {
            index += 1;
            continue;
        };
        if gcurve.is_curve3d() {
            range = gcurve.range();
        }
        if gcurve.is_curve_on_surface_at(surface, location) {
            list.remove(index);
        } else {
            index += 1;
        }
    }
    range
}

/// The range of a new p-curve representation: its own range, replaced by each
/// finite bound of the 3D curve range.
fn merged_range(own: (f64, f64), range3d: (f64, f64)) -> (f64, f64) {
    let first = if precision::is_infinite(range3d.0) {
        own.0
    } else {
        range3d.0
    };
    let last = if precision::is_infinite(range3d.1) {
        own.1
    } else {
        range3d.1
    };
    (first, last)
}

/// `UpdateCurves(lcr, C, S, L[, Pf, Pl])`: removes every curve on (`surface`,
/// `location`) and, for a non-null `pcurve`, appends a new one with the 3D curve
/// range; `uv` sets its UV points.
pub fn update_pcurve(
    list: &mut CurveList,
    pcurve: &Handle<dyn Geom2dCurve>,
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
    uv: Option<(Point2, Point2)>,
) -> Result<(), Failure> {
    let range3d = remove_curves_on(list, surface, location);
    if pcurve.is_null() {
        return Ok(());
    }
    let representation = BRepCurveOnSurface::new(pcurve, surface, location)?;
    let (first, last) = merged_range(representation.range(), range3d);
    representation.set_range(first, last)?;
    if let Some((first_uv, last_uv)) = uv {
        representation.set_uv_points(first_uv, last_uv);
    }
    list.push(Handle::new(representation).into_cast());
    Ok(())
}

/// The search loop of the two p-curve `UpdateCurves`: the range of the 3D curves met
/// before the first curve on (`surface`, `location`), which is removed.
fn remove_first_curve_on(
    list: &mut CurveList,
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
) -> (f64, f64) {
    let mut range = (-precision::infinite(), precision::infinite());
    for index in 0..list.len() {
        let gcurve = as_gcurve(&list[index]);
        let Some(gcurve) = gcurve.get() else { continue };
        if gcurve.is_curve3d() {
            range = gcurve.range();
        }
        if gcurve.is_curve_on_surface_at(surface, location) {
            list.remove(index);
            break;
        }
    }
    range
}

/// `UpdateCurves(lcr, C1, C2, S, L[, Pf, Pl])`: removes the first curve on (`surface`,
/// `location`) and, when both p-curves are non-null, appends a C0 curve on the closed
/// surface with the 3D curve range; `uv` sets the UV points of the SECOND p-curve, as
/// natively (`SetUVPoints2`).
pub fn update_pcurves(
    list: &mut CurveList,
    pcurves: (&Handle<dyn Geom2dCurve>, &Handle<dyn Geom2dCurve>),
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
    uv: Option<(Point2, Point2)>,
) -> Result<(), Failure> {
    let range3d = remove_first_curve_on(list, surface, location);
    if pcurves.0.is_null() || pcurves.1.is_null() {
        return Ok(());
    }
    let representation =
        BRepCurveOnClosedSurface::new(pcurves.0, pcurves.1, surface, location, Continuity::C0)?;
    let (first, last) = merged_range(representation.range(), range3d);
    representation.set_range(first, last)?;
    if let Some((first_uv, last_uv)) = uv {
        representation.set_uv_points2(first_uv, last_uv);
    }
    list.push(Handle::new(representation).into_cast());
    Ok(())
}

/// `UpdateCurves(lcr, S1, S2, L1, L2, C)`: sets the continuity of the first regularity
/// between the surfaces, or appends a new `BRep_CurveOn2Surfaces`.
pub fn update_regularity(
    list: &mut CurveList,
    surfaces: (&Handle<dyn GeomSurface>, &Handle<dyn GeomSurface>),
    locations: (&Location, &Location),
    continuity: Continuity,
) -> Result<(), Failure> {
    let found = list.iter().find(|representation| {
        representation.get().is_some_and(|r| {
            r.is_regularity_between(surfaces.0, surfaces.1, locations.0, locations.1)
        })
    });
    if let Some(representation) = found.and_then(|r| r.get()) {
        return representation.set_continuity(continuity);
    }
    let regularity =
        BRepCurveOn2Surfaces::new(surfaces.0, surfaces.1, locations.0, locations.1, continuity);
    list.push(Handle::new(regularity).into_cast());
    Ok(())
}

/// The first point representation accepted by `matches`.
fn find_point(
    list: &PointList,
    matches: impl Fn(&dyn BRepPointRepresentation) -> bool,
) -> Option<&dyn BRepPointRepresentation> {
    list.iter()
        .filter_map(|point| point.get())
        .find(|point| matches(*point))
}

/// `UpdatePoints(lpr, p, C, L)`: sets the parameter on the curve, or appends a new
/// `BRep_PointOnCurve`.
pub fn update_point_on_curve(
    list: &mut PointList,
    parameter: f64,
    curve: &Handle<dyn GeomCurve>,
    location: &Location,
) {
    if let Some(point) = find_point(list, |p| p.is_point_on_curve_at(curve, location)) {
        point.set_parameter(parameter);
        return;
    }
    list.push(Handle::new(BRepPointOnCurve::new(parameter, curve, location)).into_cast());
}

/// `UpdatePoints(lpr, p, PC, S, L)`: sets the parameter on the p-curve, or appends a
/// new `BRep_PointOnCurveOnSurface`.
pub fn update_point_on_pcurve(
    list: &mut PointList,
    parameter: f64,
    pcurve: &Handle<dyn Geom2dCurve>,
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
) {
    if let Some(point) = find_point(list, |p| {
        p.is_point_on_curve_on_surface_at(pcurve, surface, location)
    }) {
        point.set_parameter(parameter);
        return;
    }
    let point = BRepPointOnCurveOnSurface::new(parameter, pcurve, surface, location);
    list.push(Handle::new(point).into_cast());
}

/// `UpdatePoints(lpr, p1, p2, S, L)`: sets both parameters on the surface, or appends
/// a new `BRep_PointOnSurface`.
pub fn update_point_on_surface(
    list: &mut PointList,
    parameters: (f64, f64),
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
) -> Result<(), Failure> {
    if let Some(point) = find_point(list, |p| p.is_point_on_surface_at(surface, location)) {
        point.set_parameter(parameters.0);
        return point.set_parameter2(parameters.1);
    }
    let point = BRepPointOnSurface::new(parameters.0, parameters.1, surface, location);
    list.push(Handle::new(point).into_cast());
    Ok(())
}
