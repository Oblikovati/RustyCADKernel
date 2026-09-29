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
// Rust translation of the pole cases of
// ProjLib_ComputeApproxOnPolarSurface::ProjectUsingInitialCurve2d
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The pole cases of `ProjectUsingInitialCurve2d`: a Bezier or B-spline curve on a
//! plane gives the plane parameters of its poles. The bilinear-patch case (a
//! degree-1 2x2 Bezier or B-spline surface) tests the static `MaxDegree()` of the
//! surface class (25), so it never applies natively; it is kept as written.
use super::ComputeApproxOnPolarSurface;
use super::project::Tolerances;
use crate::extrema::GenLocateExtPS;
use crate::native_failure::failure;
use crate::proj_lib::new_array;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomBezierCurve, GeomBezierSurface, GeomCurve,
};
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Plane3, Point2, Point3, Vector3};
use std::f64::consts::PI;

/// The pole data of a Bezier or B-spline curve: the poles, the weights of a
/// rational curve, knots, multiplicities, degree and periodicity.
struct PoleCurve {
    poles: Array1<'static, Point3>,
    weights: Option<Array1<'static, f64>>,
    knots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
    degree: i32,
    periodic: bool,
}

fn null_curve() -> Failure {
    failure(
        "Standard_NullObject",
        "ProjLib_ComputeApproxOnPolarSurface: null spline curve",
    )
}

impl PoleCurve {
    fn of_bspline(curve: &dyn Adaptor3dCurve) -> Result<Self, Failure> {
        let source: Handle<GeomBSplineCurve> = curve.bspline()?;
        let bsc: &GeomBSplineCurve = source.get().ok_or_else(null_curve)?;
        Ok(Self {
            poles: bsc.poles(),
            weights: bsc.is_rational().then(|| bsc.weights_array(|w| w.clone())),
            knots: bsc.knots(),
            mults: bsc.multiplicities(),
            degree: bsc.degree(),
            periodic: bsc.is_periodic(),
        })
    }
    // A Bezier curve as one B-spline span: knots 0 and 1 of multiplicity NbPoles.
    fn of_bezier(curve: &dyn Adaptor3dCurve) -> Result<Self, Failure> {
        let source: Handle<GeomBezierCurve> = curve.bezier()?;
        let bc: &GeomBezierCurve = source.get().ok_or_else(null_curve)?;
        let mut knots: Array1<'static, f64> = new_array(2)?;
        (knots[1], knots[2]) = (0.0, 1.0);
        let mut mults: Array1<'static, i32> = new_array(2)?;
        (mults[1], mults[2]) = (bc.nb_poles(), bc.nb_poles());
        Ok(Self {
            poles: bc.poles(),
            weights: bc.is_rational().then(|| bc.weights_array(|w| w.clone())),
            knots,
            mults,
            degree: bc.degree(),
            periodic: bc.is_periodic(),
        })
    }
    fn of_kind(curve: &dyn Adaptor3dCurve, kind: CurveType) -> Result<Option<Self>, Failure> {
        match kind {
            CurveType::BSplineCurve => Ok(Some(Self::of_bspline(curve)?)),
            CurveType::BezierCurve => Ok(Some(Self::of_bezier(curve)?)),
            _ => Ok(None),
        }
    }
    // The 2D B-spline with `poles2d` and the curve's weights, knots and degree.
    fn with_poles(
        &self,
        poles2d: &Array1<'_, Point2>,
    ) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        let spline = match &self.weights {
            Some(weights) => Geom2dBSplineCurve::with_weights(
                poles2d,
                weights,
                &self.knots,
                &self.mults,
                self.degree,
                self.periodic,
            )?,
            None => Geom2dBSplineCurve::new(
                poles2d,
                &self.knots,
                &self.mults,
                self.degree,
                self.periodic,
            )?,
        };
        Ok(Handle::new(spline))
    }
}

/// A Bezier or B-spline curve on a plane: the plane parameters of its poles (the
/// adaptor's pole count); None for another curve kind.
pub(super) fn poles_on_plane(
    polar: &mut ComputeApproxOnPolarSurface,
    curve: &dyn Adaptor3dCurve,
    plane: &Plane3,
    kind: CurveType,
) -> Result<Option<Handle<Geom2dBSplineCurve>>, Failure> {
    let Some(pole_curve) = PoleCurve::of_kind(curve, kind)? else {
        return Ok(None);
    };
    polar.tol_reached = precision::confusion();
    let nb_poles: i32 = curve.nb_poles()?;
    let mut poles2d: Array1<'static, Point2> = new_array(nb_poles)?;
    for i in 1..=nb_poles {
        let (s, t) = elslib::parameters(plane, pole_curve.poles[i]);
        poles2d[i] = Point2::new(s, t);
    }
    Ok(Some(pole_curve.with_poles(&poles2d)?))
}

/// The four corner poles of a 2x2 surface, when `MaxDegree() == 1`.
fn bilinear_corners(
    surf: &dyn Adaptor3dSurface,
    kind: SurfaceType,
) -> Result<Option<[Point3; 4]>, Failure> {
    let corners =
        |pole: &dyn Fn(i32, i32) -> Result<Point3, Failure>| -> Result<[Point3; 4], Failure> {
            Ok([pole(1, 1)?, pole(1, 2)?, pole(2, 1)?, pole(2, 2)?])
        };
    if kind == SurfaceType::BSplineSurface {
        let source: Handle<GeomBSplineSurface> = surf.bspline()?;
        let bss: &GeomBSplineSurface = source.get().ok_or_else(null_curve)?;
        if GeomBSplineSurface::max_degree() != 1 || bss.nb_u_poles() != 2 || bss.nb_v_poles() != 2 {
            return Ok(None);
        }
        return Ok(Some(corners(&|u, v| bss.pole(u, v))?));
    }
    let source: Handle<GeomBezierSurface> = surf.bezier()?;
    let bs: &GeomBezierSurface = source.get().ok_or_else(null_curve)?;
    if GeomBezierSurface::max_degree() != 1 || bs.nb_u_poles() != 2 || bs.nb_v_poles() != 2 {
        return Ok(None);
    }
    Ok(Some(corners(&|u, v| bs.pole(u, v))?))
}

/// A Bezier or B-spline curve on a bilinear patch whose opposite edges are equal:
/// each pole located on the surface; None when the case does not apply or a pole
/// is not found (the fit follows).
pub(super) fn poles_on_bilinear(
    polar: &mut ComputeApproxOnPolarSurface,
    curve: &dyn Adaptor3dCurve,
    surf: &dyn Adaptor3dSurface,
    kind: SurfaceType,
    tols: &Tolerances,
) -> Result<Option<Handle<Geom2dBSplineCurve>>, Failure> {
    let Some([p11, p12, p21, p22]) = bilinear_corners(surf, kind)? else {
        return Ok(None);
    };
    let (v1, v2) = (Vector3::between(p11, p12), Vector3::between(p21, p22));
    let angular: f64 = tols.tol3d / (p11.distance(&p12) * 180. / PI);
    if !v1.is_equal(v2, tols.tol3d, angular)? {
        return Ok(None);
    }
    let c_type: CurveType = curve.get_type()?;
    let Some(pole_curve) = PoleCurve::of_kind(curve, c_type)? else {
        return Ok(None);
    };
    polar.tol_reached = tols.tol3d;
    // A B-spline starts from the mean of the corner coordinates, a Bezier from 0.5.
    let start: (f64, f64) = if c_type == CurveType::BSplineCurve {
        ((p11.x() + p22.x()) / 2., (p11.y() + p22.y()) / 2.)
    } else {
        (0.5, 0.5)
    };
    let poles2d = locate_poles(
        polar,
        surf,
        curve.nb_poles()?,
        &pole_curve,
        start,
        tols,
        c_type,
    )?;
    let Some(poles2d) = poles2d else {
        return Ok(None);
    };
    Ok(Some(pole_curve.with_poles(&poles2d)?))
}

// Each pole located from `start`, stopping at the first failure. The squared
// distance is truncated to an int before the comparison, as natively. The B-spline
// loop leaves the done flag of the last pole; the Bezier loop clears it after each
// success, so a Bezier never returns here.
fn locate_poles(
    polar: &mut ComputeApproxOnPolarSurface,
    surf: &dyn Adaptor3dSurface,
    nb_poles: i32,
    pole_curve: &PoleCurve,
    start: (f64, f64),
    tols: &Tolerances,
    c_type: CurveType,
) -> Result<Option<Array1<'static, Point2>>, Failure> {
    let mut poles2d: Array1<'static, Point2> = new_array(nb_poles)?;
    for i in 1..=nb_poles {
        if c_type == CurveType::BSplineCurve {
            polar.proj_is_done = false;
        }
        let mut extrloc = GenLocateExtPS::new(surf, tols.tol_u, tols.tol_v);
        extrloc.perform(pole_curve.poles[i], start.0, start.1, false)?;
        if !extrloc.is_done() {
            break;
        }
        let dist2_min: i32 = extrloc.square_distance()? as i32;
        if !(f64::from(dist2_min) < tols.dist_tol3d2) {
            break;
        }
        let (u, v) = extrloc.point()?.parameter();
        poles2d[i] = Point2::new(u, v);
        // The Bezier loop sets the flag and clears it at once.
        polar.proj_is_done = c_type == CurveType::BSplineCurve;
    }
    Ok(polar.proj_is_done.then_some(poles2d))
}
