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

//! The Rust twin of tools/oracle/tkgeombase_probe_geomprojlib.cxx.
use super::common::{self as p, Reader, attempt, end};
use super::geom_convert::{curve as print_curve, curve2d as print_curve2d};
use super::geom_tools::build::{ax3, curve, surface};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomPlane, GeomSurface, SurfaceBounds};
use rustycad_tkgeombase::geom_proj_lib as gpl;

/// The fields of a `gpl_curve2d` request before the curve and the surface.
struct Curve2dRequest {
    variant: i32,
    range: (f64, f64),
    bounds: SurfaceBounds,
    tol: f64,
}

// The overload of `request.variant`, and the tolerance reached when it has one.
fn curve2d_of(
    request: &Curve2dRequest,
    c: &Handle<dyn GeomCurve>,
    s: &Handle<dyn GeomSurface>,
) -> Result<(Handle<dyn Geom2dCurve>, Option<f64>), Failure> {
    let (range, bounds, tol) = (request.range, request.bounds, request.tol);
    Ok(match request.variant {
        0 => with_tol(gpl::curve2d_range_bounds_tolerance(
            c, range, s, bounds, tol,
        )?),
        1 => with_tol(gpl::curve2d_range_tolerance(c, range, s, tol)?),
        2 => (gpl::curve2d_range(c, range, s)?, None),
        3 => (gpl::curve2d(c, s)?, None),
        4 => (gpl::curve2d_bounds(c, s, bounds)?, None),
        _ => with_tol(gpl::curve2d_bounds_tolerance(c, s, bounds, tol)?),
    })
}

fn with_tol(result: gpl::Curve2dWithTolerance) -> (Handle<dyn Geom2dCurve>, Option<f64>) {
    (result.0, Some(result.1))
}

/// gb_gpl_curve2d_case.
pub fn curve2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let variant = r.integer();
    let range = (r.real(), r.real());
    let bounds = (r.real(), r.real(), r.real(), r.real());
    let request = Curve2dRequest {
        variant,
        range,
        bounds,
        tol: r.real(),
    };
    attempt(|| {
        let (c, s) = (curve(&mut r)?, surface(&mut r)?);
        let (projected, reached) = curve2d_of(&request, &c, &s)?;
        print_curve2d(&projected);
        reached.into_iter().for_each(p::real);
        Ok(())
    });
    end();
}

/// gb_gpl_project_case.
pub fn project_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (c, s) = (curve(&mut r)?, surface(&mut r)?);
        print_curve(&gpl::project(&c, &s)?);
        Ok(())
    });
    end();
}

/// gb_gpl_on_plane_case.
pub fn on_plane_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let keep = r.flag();
    let (pl, d) = (ax3(&mut r), r.dir());
    attempt(|| {
        let c = curve(&mut r)?;
        let plane: Handle<GeomPlane> = Handle::new(GeomPlane::new(pl));
        print_curve(&gpl::project_on_plane(&c, &plane, d, keep)?);
        Ok(())
    });
    end();
}
