// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_CurveOnSurface.cxx (isIsoLine, buildC3dOnIsoLine)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The exact 3D curve of a 2D line along u or v on a Geom surface: the iso curve
//! converted to a B-spline with the parametrization of the 2D curve.
use super::CurveOnSurface;
use crate::approx::curve_3d::null_table;
use crate::geom_convert::curve_to_bspline_curve;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomCurve, GeomRectangularTrimmedSurface, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Direction2, Point2, Vector2, dx2d, dy2d};

/// An iso-line: `(isU, param, isForward)` (U constant along a vertical line).
type IsoLine = (bool, f64, bool);

/// `aNbPnt`: the intervals of the deviation samples.
const DEVIATION_SAMPLES: i32 = 23;

// The location and direction of a straight 2D curve (a line, or a degree 1 Bezier
// or B-spline of two poles that are not too close).
fn line_frame(c2d: &dyn Adaptor2dCurve2d) -> Result<Option<(Point2, Direction2)>, Failure> {
    let poles: [Point2; 2] = match c2d.get_type()? {
        CurveType::Line => {
            let line = c2d.line()?;
            return Ok(Some((line.location(), line.direction())));
        }
        CurveType::BSplineCurve => {
            let bspline = c2d.bspline()?;
            let b = bspline.get().ok_or_else(|| null_table("BSpline"))?;
            if b.degree() != 1 || b.nb_poles() != 2 {
                return Ok(None);
            }
            [b.pole(1)?, b.pole(2)?]
        }
        CurveType::BezierCurve => {
            let bezier = c2d.bezier()?;
            let b = bezier.get().ok_or_else(|| null_table("Bezier"))?;
            if b.degree() != 1 || b.nb_poles() != 2 {
                return Ok(None);
            }
            [b.pole(1)?, b.pole(2)?]
        }
        _ => return Ok(None),
    };
    let vec: Vector2 = Vector2::between(poles[0], poles[1]);
    // A squared length is compared with a linear tolerance, as natively.
    if vec.square_magnitude() < precision::confusion() {
        return Ok(None);
    }
    Ok(Some((poles[0], Direction2::from_vector(vec)?)))
}

/// `isIsoLine(theC2D, theIsU, theParam, theIsForward)`.
fn iso_line(c2d: &dyn Adaptor2dCurve2d) -> Result<Option<IsoLine>, Failure> {
    let Some((loc, dir)) = line_frame(c2d)? else {
        return Ok(None);
    };
    if dir.is_parallel(dx2d(), precision::angular()) {
        return Ok(Some((false, loc.y(), dir.dot(dx2d()) > 0.0)));
    }
    if dir.is_parallel(dy2d(), precision::angular()) {
        return Ok(Some((true, loc.x(), dir.dot(dy2d()) > 0.0)));
    }
    Ok(None)
}

/// `std::min`/`std::max` as natively (the first argument unless the second is
/// strictly smaller/greater).
fn std_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}
fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

impl CurveOnSurface {
    /// Builds the 3D curve of an iso-line and returns true when it is within the
    /// tolerance (`isIsoLine` then `buildC3dOnIsoLine`).
    pub(super) fn iso_line_curve(
        &mut self,
        c2d: &Handle<dyn Adaptor2dCurve2d>,
    ) -> Result<bool, Failure> {
        let adaptor: &dyn Adaptor2dCurve2d = c2d.get().ok_or_else(|| null_table("C2D"))?;
        let Some(iso) = iso_line(adaptor)? else {
            return Ok(false);
        };
        self.build_c3d_on_iso_line(adaptor, iso)
    }

    // `buildC3dOnIsoLine`: false for a surface adaptor that is not a
    // GeomAdaptor_Surface, a sphere, or an iso range out of the surface.
    fn build_c3d_on_iso_line(
        &mut self,
        c2d: &dyn Adaptor2dCurve2d,
        (is_u, param, is_forward): IsoLine,
    ) -> Result<bool, Failure> {
        let geom_adaptor: Handle<GeomAdaptorSurface> = self.surf.cast();
        let Some(geom_adaptor) = geom_adaptor.get() else {
            return Ok(false);
        };
        let surf = self.surf.get().ok_or_else(|| null_table("mySurf"))?;
        if surf.get_type()? == SurfaceType::Sphere {
            return Ok(false);
        }
        let f2d: Point2 = c2d.value(c2d.first_parameter()?)?;
        let l2d: Point2 = c2d.value(c2d.last_parameter()?)?;
        let Some(c3d) = self.iso_curve(geom_adaptor.surface(), is_u, param, [f2d, l2d])? else {
            return Ok(false);
        };
        let bspline: Handle<GeomBSplineCurve> =
            curve_to_bspline_curve(&c3d, ParameterisationType::QuasiAngular)?;
        let curve: &GeomBSplineCurve = bspline.get().ok_or_else(|| null_table("myCurve3d"))?;
        if !is_forward {
            curve.reverse()?;
        }
        let mut knots = curve.knots();
        bsplclib::reparametrize(c2d.first_parameter()?, c2d.last_parameter()?, &mut knots)?;
        curve.set_knots(&knots)?;
        self.curve_3d = bspline;
        self.error_3d = self.iso_deviation(c2d)?;
        // Isolines of a sphere (excluded above) and some others cannot keep the 2D
        // parametrization; the general approximation is used then.
        Ok(self.error_3d <= self.tol)
    }

    // The iso curve on the parameter range of the 2D ends: trimmed, or (on an
    // infinite surface) on a trimmed surface; None when the range misses the surface
    // or is too short.
    fn iso_curve(
        &self,
        mut surf: Handle<dyn GeomSurface>,
        is_u: bool,
        param: f64,
        [f2d, l2d]: [Point2; 2],
    ) -> Result<Option<Handle<dyn GeomCurve>>, Failure> {
        let (u1, u2, v1, v2) = surf.get().ok_or_else(|| null_table("Surface"))?.bounds();
        let (ends, bounds) = if is_u {
            ([f2d.y(), l2d.y()], [v1, v2])
        } else {
            ([f2d.x(), l2d.x()], [u1, u2])
        };
        let (mut p1, mut p2) = (std_min(ends[0], ends[1]), std_max(ends[0], ends[1]));
        if p2 < bounds[0] - self.tol || p1 > bounds[1] + self.tol {
            return Ok(None);
        }
        let to_trim: bool =
            !(precision::is_infinite(bounds[0]) || precision::is_infinite(bounds[1]));
        if to_trim {
            p1 = std_max(p1, bounds[0]);
            p2 = std_min(p2, bounds[1]);
        }
        if (p2 - p1).abs() < precision::p_confusion() {
            return Ok(None);
        }
        if !to_trim {
            let (u, v) = if is_u {
                ((u1, u2), (p1, p2))
            } else {
                ((p1, p2), (v1, v2))
            };
            let trimmed = GeomRectangularTrimmedSurface::new(&surf, u, v, (true, true))?;
            surf = Handle::new(trimmed).cast::<dyn GeomSurface>();
        }
        let surface = surf.get().ok_or_else(|| null_table("Surface"))?;
        let iso = if is_u {
            surface.u_iso(param)?
        } else {
            surface.v_iso(param)?
        };
        if !to_trim {
            return Ok(Some(iso));
        }
        let trimmed = GeomTrimmedCurve::new(&iso, p1, p2, true, true)?;
        Ok(Some(Handle::new(trimmed).cast::<dyn GeomCurve>()))
    }

    // The largest distance between the 3D curve and the curve on surface at 24
    // parameters of [myFirst, myLast].
    fn iso_deviation(&self, c2d: &dyn Adaptor2dCurve2d) -> Result<f64, Failure> {
        let curve = self.curve_3d.get().ok_or_else(|| null_table("myCurve3d"))?;
        let surf = self.surf.get().ok_or_else(|| null_table("mySurf"))?;
        let mut error: f64 = 0.0;
        for index in 0..=DEVIATION_SAMPLES {
            let par: f64 = self.first
                + ((self.last - self.first) * f64::from(index)) / f64::from(DEVIATION_SAMPLES);
            let p2d: Point2 = c2d.value(par)?;
            let p_c3d = curve.value(par)?;
            let p_c2d = surf.value(p2d.x(), p2d.y())?;
            error = std_max(p_c3d.square_distance(&p_c2d), error);
        }
        Ok(error.sqrt())
    }
}
