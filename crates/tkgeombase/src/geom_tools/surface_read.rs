// Created on: 1993-07-19
// Created by: Remi LEQUETTE
// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of GeomTools_SurfaceSet::ReadSurface and its static `operator>>`
// overloads (GeomTools_SurfaceSet.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Reading one compact `Geom_Surface` record back.
use super::curve_read::read_curve;
use super::text_in::{get_real, read_direction3, read_flag, read_int, read_knots, read_point3};
use super::type_codes::surface as code;
use super::undefined_type_handler;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, Transient};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomConicalSurface, GeomCylindricalSurface,
    GeomOffsetSurface, GeomPlane, GeomRectangularTrimmedSurface, GeomSphericalSurface, GeomSurface,
    GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution, GeomToroidalSurface,
};
use rustycad_tkmath::gp::{Axis1, Axis3, Point3};

/// `GeomTools_SurfaceSet::ReadSurface`: the type code, then the record of that
/// class; an unknown code goes to the active undefined-type handler. Every failure
/// gives a null surface, with the stream left where the failure happened.
pub(crate) fn read_surface(stream: &mut IoStream) -> Handle<dyn GeomSurface> {
    // An uninitialised native `stype` when the stream already failed reads 0 here.
    let stype = read_int(stream);
    read_record(stype, stream).unwrap_or_default()
}

fn read_record(stype: i32, stream: &mut IoStream) -> Result<Handle<dyn GeomSurface>, Failure> {
    match stype {
        code::PLANE => Ok(upcast(GeomPlane::new(read_axis3(stream)?))),
        code::CYLINDER => read_cylinder(stream).map(upcast),
        code::CONE => read_cone(stream).map(upcast),
        code::SPHERE => read_sphere(stream).map(upcast),
        code::TORUS => read_torus(stream).map(upcast),
        code::LINEAR_EXTRUSION => read_extrusion(stream).map(upcast),
        code::REVOLUTION => read_revolution(stream).map(upcast),
        code::BEZIER => read_bezier(stream).map(upcast),
        code::BSPLINE => read_bspline(stream).map(upcast),
        code::RECTANGULAR => read_trimmed(stream).map(upcast),
        code::OFFSET => read_offset(stream).map(upcast),
        _ => undefined_type_handler().read_surface(stype, stream),
    }
}

/// `occ::handle<Geom_Surface> S = SS;`.
fn upcast<S: Transient + RuntimeTyped>(surface: S) -> Handle<dyn GeomSurface> {
    Handle::new(surface).into_cast()
}

/// `IS >> gp_Ax3`: location and the main, X and Y directions; `gp_Ax3(P, A, AX)` is
/// built (and may raise) before the record goes on, and the frame is made indirect
/// when the stored Y direction lies on the other side (`AY.DotCross(A, AX) < 0`).
fn read_axis3(stream: &mut IoStream) -> Result<Axis3, Failure> {
    let point = read_point3(stream);
    let main = read_direction3(stream)?;
    let x = read_direction3(stream)?;
    let y = read_direction3(stream)?;
    let mut frame = Axis3::new(point, main, x)?;
    if y.dot_cross(main, x) < 0.0 {
        frame.y_reverse();
    }
    Ok(frame)
}

fn read_cylinder(stream: &mut IoStream) -> Result<GeomCylindricalSurface, Failure> {
    let frame = read_axis3(stream)?;
    GeomCylindricalSurface::new(frame, get_real(stream))
}

/// The radius is written before the semi-angle, while the constructor takes the
/// angle first.
fn read_cone(stream: &mut IoStream) -> Result<GeomConicalSurface, Failure> {
    let frame = read_axis3(stream)?;
    let (radius, angle) = (get_real(stream), get_real(stream));
    GeomConicalSurface::new(frame, angle, radius)
}

fn read_sphere(stream: &mut IoStream) -> Result<GeomSphericalSurface, Failure> {
    let frame = read_axis3(stream)?;
    GeomSphericalSurface::new(frame, get_real(stream))
}

fn read_torus(stream: &mut IoStream) -> Result<GeomToroidalSurface, Failure> {
    let frame = read_axis3(stream)?;
    let (major, minor) = (get_real(stream), get_real(stream));
    GeomToroidalSurface::new(frame, major, minor)
}

/// The direction, then the basis curve record.
fn read_extrusion(stream: &mut IoStream) -> Result<GeomSurfaceOfLinearExtrusion, Failure> {
    let direction = read_direction3(stream)?;
    let curve = read_curve(stream);
    GeomSurfaceOfLinearExtrusion::new(&curve, direction)
}

/// The axis location and direction, then the basis curve record.
fn read_revolution(stream: &mut IoStream) -> Result<GeomSurfaceOfRevolution, Failure> {
    let point = read_point3(stream);
    let direction = read_direction3(stream)?;
    let curve = read_curve(stream);
    GeomSurfaceOfRevolution::new(&curve, Axis1::new(point, direction))
}

/// `NCollection_Array2<T>(1, rows, 1, cols)`: a negative extent is refused as the
/// allocation failure it produces natively.
fn array2<T: Default>(rows: i32, cols: i32) -> Result<Array2<'static, T>, Failure> {
    Array2::with_bounds(1, rows, 1, cols).map_err(|error| error.into_failure("NCollection_Array2"))
}

/// The pole grid, row by row, each pole followed by its weight when `weighted`.
fn read_pole_grid(
    stream: &mut IoStream,
    (rows, cols): (i32, i32),
    weighted: bool,
) -> Result<(Array2<'static, Point3>, Array2<'static, f64>), Failure> {
    let mut poles: Array2<'static, Point3> = array2(rows, cols)?;
    let mut weights: Array2<'static, f64> = array2(rows, cols)?;
    let stored = |error: rustycad_tkernel::ncollection::ArrayError| {
        error.into_failure("NCollection_Array2::ChangeValue")
    };
    for row in 1..=rows {
        for col in 1..=cols {
            poles
                .set_value(row, col, read_point3(stream))
                .map_err(stored)?;
            if weighted {
                weights
                    .set_value(row, col, get_real(stream))
                    .map_err(stored)?;
            }
        }
    }
    Ok((poles, weights))
}

/// `IS >> urational >> vrational; IS >> udegree >> vdegree;`, then the poles.
fn read_bezier(stream: &mut IoStream) -> Result<GeomBezierSurface, Failure> {
    let weighted = read_flag(stream) | read_flag(stream);
    let (udegree, vdegree) = (read_int(stream), read_int(stream));
    // `udegree + 1` wraps as the native int arithmetic does in practice.
    let size = (udegree.wrapping_add(1), vdegree.wrapping_add(1));
    let (poles, weights) = read_pole_grid(stream, size, weighted)?;
    if weighted {
        GeomBezierSurface::with_weights(&poles, &weights)
    } else {
        GeomBezierSurface::new(&poles)
    }
}

/// The B-spline flags, degrees and counts, the poles, then the U and V knots.
fn read_bspline(stream: &mut IoStream) -> Result<GeomBSplineSurface, Failure> {
    let weighted = read_flag(stream) | read_flag(stream);
    let periodic = (read_flag(stream), read_flag(stream));
    let degrees = (read_int(stream), read_int(stream));
    let nb_poles = (read_int(stream), read_int(stream));
    let (nb_uknots, nb_vknots) = (read_int(stream), read_int(stream));
    let (poles, weights) = read_pole_grid(stream, nb_poles, weighted)?;
    let (uknots, umults) = read_knots(stream, nb_uknots)?;
    let (vknots, vmults) = read_knots(stream, nb_vknots)?;
    let (u, v) = ((&uknots, &umults), (&vknots, &vmults));
    if weighted {
        GeomBSplineSurface::with_weights(&poles, &weights, u, v, degrees, periodic)
    } else {
        GeomBSplineSurface::new(&poles, u, v, degrees, periodic)
    }
}

/// `U1 U2 V1 V2`, then the basis surface record.
fn read_trimmed(stream: &mut IoStream) -> Result<GeomRectangularTrimmedSurface, Failure> {
    let u = (get_real(stream), get_real(stream));
    let v = (get_real(stream), get_real(stream));
    let basis = read_surface(stream);
    GeomRectangularTrimmedSurface::new(&basis, u, v, (true, true))
}

/// The offset, then the basis surface record; the reader does not check the basis
/// continuity (`isNotCheckC0` true).
fn read_offset(stream: &mut IoStream) -> Result<GeomOffsetSurface, Failure> {
    let offset = get_real(stream);
    let basis = read_surface(stream);
    GeomOffsetSurface::new(&basis, offset, true)
}
