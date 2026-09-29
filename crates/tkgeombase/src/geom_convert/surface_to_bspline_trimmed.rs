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
// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_1.cxx
// (SurfaceToBSplineSurface: the Geom_RectangularTrimmedSurface case)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The rectangular trimmed surface branch of `GeomConvert::SurfaceToBSplineSurface`.
use super::surface_to_bspline::{
    RevolutionSweep, approximate, bezier_surface_to_bspline, bspline_surface_builder,
    surface_to_bspline_surface, swept_meridian,
};
use super::{live_bspline, live_bspline_surface};
use crate::array_access::{grid, put};
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomBezierSurface, GeomConicalSurface, GeomCurve,
    GeomCylindricalSurface, GeomOffsetSurface, GeomPlane, GeomRectangularTrimmedSurface,
    GeomSphericalSurface, GeomSurface, GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution,
    GeomSweptSurface, GeomToroidalSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::convert::ElementarySurfaceToBSplineSurface as Elementary;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Vector3};
use std::f64::consts::PI;

/// The trimmed domain `[UFirst, ULast] x [VFirst, VLast]` (the trimmed surface's own
/// bounds, not sorted) with the full-turn tests of the elementary surfaces.
#[derive(Clone, Copy)]
struct TrimDomain {
    u_first: f64,
    u_last: f64,
    v_first: f64,
    v_last: f64,
    is_u_closed: bool,
    eps: f64,
}

/// The Geom_RectangularTrimmedSurface case of SurfaceToBSplineSurface: the basis
/// (an offset basis replaced by its equivalent surface, a trimmed basis re-trimmed
/// and converted again) converted on the trimmed domain.
pub(super) fn trimmed_to_bspline(
    sr: &Handle<dyn GeomSurface>,
    strim: &GeomRectangularTrimmedSurface,
    [u1, u2, v1, v2]: [f64; 4],
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let mut surf: Handle<dyn GeomSurface> = strim.basis_surface();
    if let Some(offset) = surf.cast::<GeomOffsetSurface>().get() {
        let equivalent: Handle<dyn GeomSurface> = offset.surface()?;
        if !equivalent.is_null() {
            surf = equivalent;
        }
    }
    if surf.cast::<GeomRectangularTrimmedSurface>().is_some() {
        let a_strim = GeomRectangularTrimmedSurface::new(&surf, (u1, u2), (v1, v2), (true, true))?;
        return surface_to_bspline_surface(&Handle::new(a_strim).cast());
    }
    // For cylinders, cones, spheres, toruses.
    let domain = TrimDomain {
        u_first: u1,
        u_last: u2,
        v_first: v1,
        v_last: v2,
        is_u_closed: ((u2 - u1) - 2.0 * PI).abs() <= precision::p_confusion(),
        eps: 100.0 * epsilon(2.0 * PI),
    };
    if surf.cast::<GeomPlane>().is_some() {
        return trimmed_plane(strim, [u1, u2, v1, v2]);
    }
    if let Some(elementary) = trimmed_elementary(&surf, strim, domain)? {
        return Ok(elementary);
    }
    trimmed_swept_or_free(sr, &surf, strim, domain)
}

// The plane: the four corners of the trimmed surface as degree-1 poles.
fn trimmed_plane(
    strim: &GeomRectangularTrimmedSurface,
    [u1, u2, v1, v2]: [f64; 4],
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let mut poles: Array2<'static, Point3> = grid(2, 2)?;
    put(&mut poles, 1, 1, strim.value(u1, v1)?, "Poles")?;
    put(&mut poles, 1, 2, strim.value(u1, v2)?, "Poles")?;
    put(&mut poles, 2, 1, strim.value(u2, v1)?, "Poles")?;
    put(&mut poles, 2, 2, strim.value(u2, v2)?, "Poles")?;
    let knots = |a: f64, b: f64| -> Result<Array1<'static, f64>, Failure> {
        let mut k: Array1<'static, f64> = one_based(2)?;
        (k[1], k[2]) = (a, b);
        Ok(k)
    };
    let mut mults: Array1<'static, i32> = one_based(2)?;
    (mults[1], mults[2]) = (2, 2);
    let (u_knots, v_knots) = (knots(u1, u2)?, knots(v1, v2)?);
    Ok(Handle::new(GeomBSplineSurface::new(
        &poles,
        (&u_knots, &mults),
        (&v_knots, &mults),
        (1, 1),
        (false, false),
    )?))
}

// The cylinder, cone, sphere and torus; None for any other basis.
fn trimmed_elementary(
    surf: &Handle<dyn GeomSurface>,
    strim: &GeomRectangularTrimmedSurface,
    d: TrimDomain,
) -> Result<Option<Handle<GeomBSplineSurface>>, Failure> {
    let (uf, ul, vf, vl) = (d.u_first, d.u_last, d.v_first, d.v_last);
    let convert: Elementary = if let Some(cyl) = surf.cast::<GeomCylindricalSurface>().get() {
        let cylinder = cyl.cylinder()?;
        if d.is_u_closed {
            return Ok(Some(checked_u_segment(
                &Elementary::from_cylinder(&cylinder, vf, vl)?,
                d,
            )?));
        }
        Elementary::from_cylinder_patch(&cylinder, uf, ul, vf, vl)?
    } else if let Some(cone) = surf.cast::<GeomConicalSurface>().get() {
        let co = cone.cone()?;
        if d.is_u_closed {
            return Ok(Some(checked_u_segment(
                &Elementary::from_cone(&co, vf, vl)?,
                d,
            )?));
        }
        Elementary::from_cone_patch(&co, uf, ul, vf, vl)?
    } else if let Some(sphere) = surf.cast::<GeomSphericalSurface>().get() {
        let sph = sphere.sphere()?;
        // OCC217
        if d.is_u_closed {
            return Ok(Some(checked_u_segment(
                &Elementary::from_sphere_band(&sph, vf, vl, false)?,
                d,
            )?));
        }
        Elementary::from_sphere_patch(&sph, uf, ul, vf, vl)?
    } else if let Some(torus) = surf.cast::<GeomToroidalSurface>().get() {
        return trimmed_torus(torus, strim, d).map(Some);
    } else {
        return Ok(None);
    };
    bspline_surface_builder(&convert).map(Some)
}

// The torus. When the U trim is a full turn not in the first period, CheckAndSegment
// would shift the U seam (probably a bug), so a non-periodic patch is built instead.
fn trimmed_torus(
    torus: &GeomToroidalSurface,
    strim: &GeomRectangularTrimmedSurface,
    d: TrimDomain,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let tr = torus.torus()?;
    let is_u_first_period: bool = d.u_first >= 0.0 && d.u_last <= 2.0 * PI;
    let is_v_first_period: bool = d.v_first >= 0.0 && d.v_last <= 2.0 * PI;
    if d.is_u_closed && is_u_first_period {
        return checked_u_segment(
            &Elementary::from_torus_band(&tr, d.v_first, d.v_last, false)?,
            d,
        );
    }
    if strim.is_v_closed()? && is_v_first_period {
        let the_surface = bspline_surface_builder(&Elementary::from_torus_band(
            &tr, d.u_first, d.u_last, true,
        )?)?;
        let s: &GeomBSplineSurface = live_bspline_surface(&the_surface)?;
        let nb_k: i32 = s.nb_v_knots();
        if (s.v_knot(1)? - d.v_first).abs() > d.eps || (s.v_knot(nb_k)? - d.v_last).abs() > d.eps {
            check_and_segment(s, d)?;
        }
        return Ok(the_surface);
    }
    bspline_surface_builder(&Elementary::from_torus_patch(
        &tr, d.u_first, d.u_last, d.v_first, d.v_last,
    )?)
}

// The U-periodic conversion of a full-turn band, segmented when its U knots do not
// start and end at the trim within `eps`.
fn checked_u_segment(
    convert: &Elementary,
    d: TrimDomain,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let the_surface: Handle<GeomBSplineSurface> = bspline_surface_builder(convert)?;
    let s: &GeomBSplineSurface = live_bspline_surface(&the_surface)?;
    let nb_k: i32 = s.nb_u_knots();
    if (s.u_knot(1)? - d.u_first).abs() > d.eps || (s.u_knot(nb_k)? - d.u_last).abs() > d.eps {
        check_and_segment(s, d)?;
    }
    Ok(the_surface)
}

fn check_and_segment(s: &GeomBSplineSurface, d: TrimDomain) -> Result<(), Failure> {
    s.check_and_segment(
        (d.u_first, d.u_last),
        (d.v_first, d.v_last),
        (precision::p_confusion(), precision::p_confusion()),
    )
}

// The revolution, extrusion, Bezier and B-spline bases; any other is approximated.
fn trimmed_swept_or_free(
    sr: &Handle<dyn GeomSurface>,
    surf: &Handle<dyn GeomSurface>,
    strim: &GeomRectangularTrimmedSurface,
    d: TrimDomain,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    if let Some(revol) = surf.cast::<GeomSurfaceOfRevolution>().get() {
        return trimmed_revolution(revol, strim, d);
    }
    if let Some(extru) = surf.cast::<GeomSurfaceOfLinearExtrusion>().get() {
        return trimmed_extrusion(extru, strim, d);
    }
    if surf.cast::<GeomBezierSurface>().is_some() {
        let sbez: Handle<GeomBezierSurface> = surf.copy()?.cast();
        sbez.get()
            .ok_or_else(|| failure("Standard_NullObject", "null Bezier copy"))?
            .segment(d.u_first, d.u_last, d.v_first, d.v_last)?;
        return bezier_surface_to_bspline(&sbez);
    }
    if surf.cast::<GeomBSplineSurface>().is_some() {
        return trimmed_bspline(surf, d);
    }
    let adaptor = GeomAdaptorSurface::with_surface(sr)?;
    let cont: Shape =
        if adaptor.nb_u_intervals(Shape::C2)? > 1 || adaptor.nb_v_intervals(Shape::C2)? > 1 {
            Shape::C1
        } else {
            Shape::C2
        };
    approximate(sr, cont, cont)
}

// A copy of the B-spline basis, the trim clamped to a non-periodic direction, then
// CheckAndSegment for a periodic surface and Segment otherwise.
fn trimmed_bspline(
    surf: &Handle<dyn GeomSurface>,
    d: TrimDomain,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let bs_handle: Handle<GeomBSplineSurface> = surf.copy()?.cast();
    let bs: &GeomBSplineSurface = live_bspline_surface(&bs_handle)?;
    let (umin, umax, vmin, vmax) = bs.bounds();
    let (mut u1, mut u2, mut v1, mut v2) = (d.u_first, d.u_last, d.v_first, d.v_last);
    if !bs.is_u_periodic() {
        if u1 < umin {
            u1 = umin;
        }
        if u2 > umax {
            u2 = umax;
        }
    }
    if !bs.is_v_periodic() {
        if v1 < vmin {
            v1 = vmin;
        }
        if v2 > vmax {
            v2 = vmax;
        }
    }
    let tolerances: (f64, f64) = (precision::p_confusion(), precision::p_confusion());
    if bs.is_u_periodic() || bs.is_v_periodic() {
        bs.check_and_segment((u1, u2), (v1, v2), tolerances)?;
    } else {
        bs.segment((u1, u2), (v1, v2), tolerances)?;
    }
    Ok(bs_handle)
}

// The revolution: the meridian (trimmed to the V range unless the trimmed surface is
// V-closed) swept in spans of at most 150 degrees, or in the three periodic spans of
// a U-periodic trimmed surface.
fn trimmed_revolution(
    revol: &GeomSurfaceOfRevolution,
    strim: &GeomRectangularTrimmedSurface,
    d: TrimDomain,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let c: Handle<GeomBSplineCurve> = swept_meridian(
        &revol.basis_curve(),
        strim.is_v_closed()?,
        (d.v_first, d.v_last),
    )?;
    let sweep: RevolutionSweep = if strim.is_u_periodic() {
        RevolutionSweep {
            axis: revol.axis(),
            u_first: d.u_first,
            alfa_u: PI / 3.0,
            nb_u_poles: 6,
            nb_u_knots: 4,
            periodic: true,
        }
    } else {
        // Number of spans: maximum opening = 150 degrees (= PI / 1.2 rad).
        let nb_u_spans: i32 = (1.2 * (d.u_last - d.u_first) / PI).trunc() as i32 + 1;
        RevolutionSweep {
            axis: revol.axis(),
            u_first: d.u_first,
            alfa_u: (d.u_last - d.u_first) / f64::from(nb_u_spans * 2),
            nb_u_poles: 2 * nb_u_spans + 1,
            nb_u_knots: nb_u_spans + 1,
            periodic: false,
        }
    };
    sweep.build(&c)
}

// The extrusion: the directrix (trimmed to the U range unless the trimmed surface is
// U-closed) translated to the two V bounds, degree 1 in V.
fn trimmed_extrusion(
    extru: &GeomSurfaceOfLinearExtrusion,
    strim: &GeomRectangularTrimmedSurface,
    d: TrimDomain,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let c: Handle<GeomBSplineCurve> = swept_meridian(
        &extru.basis_curve(),
        strim.is_u_closed()?,
        (d.u_first, d.u_last),
    )?;
    let meridian: &GeomBSplineCurve = live_bspline(&c)?;
    let n: i32 = meridian.nb_poles();
    let mut poles: Array2<'static, Point3> = grid(n, 2)?;
    let mut weights: Array2<'static, f64> = grid(n, 2)?;
    let mut v_knots: Array1<'static, f64> = one_based(2)?;
    (v_knots[1], v_knots[2]) = (d.v_first, d.v_last);
    let mut v_mults: Array1<'static, i32> = one_based(2)?;
    v_mults.init(2);
    let direction = Vector3::from(extru.direction());
    let (dv1, dv2): (Vector3, Vector3) = (d.v_first * direction, d.v_last * direction);
    for i in 1..=n {
        put(&mut poles, i, 1, meridian.pole(i)?.translated(dv1), "Poles")?;
        put(&mut poles, i, 2, meridian.pole(i)?.translated(dv2), "Poles")?;
        let w: f64 = meridian.weight(i)?;
        put(&mut weights, i, 1, w, "Weights")?;
        put(&mut weights, i, 2, w, "Weights")?;
    }
    Ok(Handle::new(GeomBSplineSurface::with_weights(
        &poles,
        &weights,
        (&meridian.knots(), &meridian.multiplicities()),
        (&v_knots, &v_mults),
        (meridian.degree(), 1),
        (meridian.is_periodic(), false),
    )?))
}
