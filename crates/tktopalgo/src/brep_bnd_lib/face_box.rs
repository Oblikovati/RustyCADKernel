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
// Rust translation of the static AdjustFaceBox and IsModifySize of BRepBndLib.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The file-static `AdjustFaceBox` and `IsModifySize` of BRepBndLib.cxx: shrink the
//! box of a surface patch towards the box of the face's edges where the surface does
//! not reach the patch box side inside the face.
use super::uv_bounds::{UvPatch, std_max, uv_tolerances};
use crate::brep_top_adaptor::BRepTopAdaptorFClass2d;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkgeombase::extrema::ExtSS;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{self, Axis3, Direction3, Plane3, Point2, Point3};

/// What `IsModifySize` needs besides the plane and the point: the surface, its patch,
/// the face classifier and the UV tolerances.
struct PatchProbe<'p> {
    surface: &'p BRepAdaptorSurface,
    patch: UvPatch,
    classifier: &'p BRepTopAdaptorFClass2d,
    tols: (f64, f64),
}

/// `AdjustFaceBox(BS, umin, umax, vmin, vmax, FaceBox, EdgeBox, Tol)`: the face box
/// becomes the edge box when it is void; else each side of the face box that lies
/// beyond the edge box (x min, x max, y min, y max, z min, z max in turn, each seeing
/// the sides moved before it) moves to the edge box when the surface patch nearest
/// to that side is outside the face.
pub(super) fn adjust_face_box(
    surface: &BRepAdaptorSurface,
    patch: UvPatch,
    face_box: &mut BoundingBox3,
    edge_box: &BoundingBox3,
    tol: f64,
) -> Result<(), Failure> {
    if edge_box.is_void() {
        return Ok(());
    }
    if face_box.is_void() {
        *face_box = edge_box.clone();
        return Ok(());
    }
    let mut f: [f64; 6] = face_box.get()?;
    let e: [f64; 6] = edge_box.get()?;
    let tols = uv_tolerances(surface, tol)?;
    let classifier = BRepTopAdaptorFClass2d::new(&surface.face(), std_max(tols.0, tols.1))?;
    let probe = PatchProbe {
        surface,
        patch,
        classifier: &classifier,
        tols,
    };
    let mut is_modified = false;
    for axis in 0..3 {
        is_modified |= shrink_side(&probe, axis, false, &mut f, &e)?;
        is_modified |= shrink_side(&probe, axis, true, &mut f, &e)?;
    }
    if is_modified {
        face_box.set_void();
        face_box.update_bounds([f[0], f[1], f[2]], [f[3], f[4], f[5]]);
    }
    Ok(())
}

/// One side of [`adjust_face_box`]: the `axis` minimum (or maximum with `upper`) side
/// of the face box `f`, moved to the edge box `e` when it lies beyond it and
/// `IsModifySize` accepts the plane of that side through the face box corner, with
/// the opposite corner of the side as the patch end.
fn shrink_side(
    probe: &PatchProbe<'_>,
    axis: usize,
    upper: bool,
    f: &mut [f64; 6],
    e: &[f64; 6],
) -> Result<bool, Failure> {
    let (near, far) = if upper { (3, 0) } else { (0, 3) };
    let beyond = if upper {
        e[near + axis] < f[near + axis]
    } else {
        e[near + axis] > f[near + axis]
    };
    if !beyond {
        return Ok(false);
    }
    let origin = Point3::new(f[near], f[near + 1], f[near + 2]);
    let mut end: [f64; 3] = [f[far], f[far + 1], f[far + 2]];
    end[axis] = f[near + axis];
    let plane = Plane3::from_position(Axis3::from_normal(origin, axis_direction(axis))?);
    if !is_modify_size(probe, &plane, Point3::new(end[0], end[1], end[2]))? {
        return Ok(false);
    }
    f[near + axis] = e[near + axis];
    Ok(true)
}

/// `gp::DX()`, `gp::DY()` or `gp::DZ()`.
fn axis_direction(axis: usize) -> Direction3 {
    [gp::dx(), gp::dy(), gp::dz()][axis]
}

/// `IsModifySize(theBS, thePln, theP, umin, umax, vmin, vmax, theFClass, theTolU,
/// theTolV)`: true when no extremum joins the rectangle of `plane` from its origin to
/// `point` and the surface patch, or when the nearest extremum lies outside the face.
fn is_modify_size(probe: &PatchProbe<'_>, plane: &Plane3, point: Point3) -> Result<bool, Failure> {
    let (pu2, pv2) = elslib::plane_parameters(plane.position(), point);
    let (pu1, pu2) = reorder(0., pu2);
    let (pv1, pv2) = reorder(0., pv2);
    let geom_plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_pln(*plane)).cast();
    let plane_patch = GeomAdaptorSurface::with_bounds(&geom_plane, (pu1, pu2, pv1, pv2))?;
    let extrema = ExtSS::with_ranges(
        &plane_patch,
        probe.surface,
        [pu1, pu2, pv1, pv2],
        probe.patch,
        probe.tols,
    )?;
    if !extrema.is_done() {
        return Ok(false);
    }
    if extrema.nb_ext()? == 0 {
        return Ok(true); // extrema point seems to be out of face UV bounds
    }
    let Some(nearest) = nearest_extremum(&extrema)? else {
        return Ok(false);
    };
    let (_, on_surface) = extrema.points(nearest)?;
    let (u, v) = on_surface.parameter();
    Ok(probe.classifier.perform(Point2::new(u, v), true)? != State::In)
}

/// The first extremum of smallest square distance (strictly below RealLast); `None`
/// when every distance is RealLast or NaN.
fn nearest_extremum(extrema: &ExtSS<'_>) -> Result<Option<i32>, Failure> {
    let (mut imin, mut dmin) = (0, f64::MAX);
    for i in 1..=extrema.nb_ext()? {
        let d: f64 = extrema.square_distance(i)?;
        if d < dmin {
            (imin, dmin) = (i, d);
        }
    }
    Ok((imin > 0).then_some(imin))
}

/// `Reorder(a, b)`: the pair in increasing order.
fn reorder(a: f64, b: f64) -> (f64, f64) {
    if a > b { (b, a) } else { (a, b) }
}
