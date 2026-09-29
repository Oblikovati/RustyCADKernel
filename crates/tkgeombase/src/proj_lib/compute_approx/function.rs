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
// Rust translation of Function_Value, Function_D1, Function_ComputeStep and
// ProjLib_Function (ProjLib_ComputeApprox.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The projected curve of ProjLib_ComputeApprox as a continuous 2D function: the
//! ElSLib parameters of each curve point, moved into the bounds that
//! `Function_SetUVBounds` chose across the seams.
use crate::app_cont::{Function, FunctionBase};
use crate::failure_capture::FailureCapture;
use crate::gcpnts::AbscissaPoint;
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::real::{epsilon, real_to_int};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::GeomSurfaceD1;
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use std::cell::RefCell;
use std::f64::consts::{FRAC_PI_4, PI};

/// The parameter window of the projected curve and whether it crosses the U or V
/// seam (`myU1`, `myU2`, `myV1`, `myV2`, `UCouture`, `VCouture`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct UvBounds {
    pub(super) u1: f64,
    pub(super) u2: f64,
    pub(super) v1: f64,
    pub(super) v2: f64,
    pub(super) u_couture: bool,
    pub(super) v_couture: bool,
}

/// The adapted curve and surface of one projection.
#[derive(Clone, Copy)]
pub(super) struct Projection<'a> {
    pub(super) curve: &'a dyn Adaptor3dCurve,
    pub(super) surface: &'a dyn Adaptor3dSurface,
}

/// The ElSLib parameters of `point` on an elementary adapted surface;
/// Standard_NoSuchObject(`member`) for any other surface.
pub(in crate::proj_lib) fn elementary_parameters(
    surface: &dyn Adaptor3dSurface,
    point: Point3,
    member: &str,
) -> Result<(f64, f64), Failure> {
    match surface.get_type()? {
        SurfaceType::Plane => Ok(elslib::parameters(&surface.plane()?, point)),
        SurfaceType::Cylinder => Ok(elslib::parameters(&surface.cylinder()?, point)),
        SurfaceType::Cone => Ok(elslib::parameters(&surface.cone()?, point)),
        SurfaceType::Sphere => Ok(elslib::parameters(&surface.sphere()?, point)),
        SurfaceType::Torus => Ok(elslib::parameters(&surface.torus()?, point)),
        _ => Err(failure("Standard_NoSuchObject", member)),
    }
}

impl Projection<'_> {
    /// `Function_Value(U, ...)`: the surface parameters of `C(u)` in the window.
    pub(super) fn value(&self, u: f64, b: &UvBounds) -> Result<Point2, Failure> {
        let p3d: Point3 = self.curve.value(u)?;
        let (mut s, mut t) =
            elementary_parameters(self.surface, p3d, "ProjLib_ComputeApprox::Value")?;
        if b.u_couture && (s < b.u1 || s > b.u2) {
            s = elclib::in_period(s, b.u1, b.u2);
        }
        if b.v_couture {
            if self.surface.get_type()? == SurfaceType::Sphere {
                if (s - b.u1).abs() > PI {
                    t = PI - t;
                    s = PI + s;
                }
                // Natively `S > U1 || S < U2`, which almost always holds.
                if s > b.u1 || s < b.u2 {
                    s = elclib::in_period(s, b.u1, b.u2);
                }
            }
            if t < b.v1 || t > b.v2 {
                t = elclib::in_period(t, b.v1, b.v2);
            }
        }
        Ok(Point2::new(s, t))
    }

    /// `Function_D1(U, P, D, ...)`: the point and `(C'.Su / |Su|^2, C'.Sv / |Sv|^2)`;
    /// None when a surface derivative is (almost) null.
    pub(super) fn d1(&self, u: f64, b: &UvBounds) -> Result<Option<(Point2, Vector2)>, Failure> {
        let p: Point2 = self.value(u, b)?;
        let (_, tangent) = self.curve.d1(u)?;
        let s: GeomSurfaceD1 = self.surface.d1(p.x(), p.y())?;
        let (mut du, mut dv) = (tangent.dot(s.d1u), tangent.dot(s.d1v));
        let (nu, nv) = (s.d1u.square_magnitude(), s.d1v.square_magnitude());
        let eps: f64 = epsilon(1.);
        if nu < eps || nv < eps {
            return Ok(None);
        }
        du /= nu;
        dv /= nv;
        Ok(Some((p, Vector2::new(du, dv))))
    }
}

/// `Function_ComputeStep(C, R)`: the sampling step of the seam search, about a
/// quarter turn of radius `r` along the curve and at most 0.1;
/// `compute_step(curve, 2.0)?`.
pub(super) fn compute_step(curve: &dyn Adaptor3dCurve, r: f64) -> Result<f64, Failure> {
    let step0: f64 = 0.1;
    let (w1, w2) = (curve.first_parameter()?, curve.last_parameter()?);
    let length: f64 = AbscissaPoint::length(curve)?;
    let mut nbp: i32 = real_to_int(length / (r * FRAC_PI_4)) + 1;
    nbp = nbp.max(3);
    let mut step: f64 = (w2 - w1) / f64::from(nbp - 1);
    if step > step0 {
        step = step0;
        nbp = real_to_int((w2 - w1) / step) + 1;
        nbp = nbp.max(3);
        step = (w2 - w1) / f64::from(nbp - 1);
    }
    Ok(step)
}

/// `ProjLib_Function`: one 2D point, periodic in the surface periods.
pub(super) struct ProjectionFunction<'a> {
    base: FunctionBase,
    projection: Projection<'a>,
    pub(super) bounds: UvBounds,
    periodic: [bool; 2],
    period: [f64; 2],
    failure: RefCell<FailureCapture>,
}

impl<'a> ProjectionFunction<'a> {
    /// `ProjLib_Function(C, S)`: computes the window, then the periods.
    pub(super) fn new(projection: Projection<'a>) -> Result<Self, Failure> {
        let mut base: FunctionBase = FunctionBase::new();
        base.set_number_of_points(0, 1);
        let bounds: UvBounds = super::uv_bounds::set_uv_bounds(&projection)?;
        let surface: &dyn Adaptor3dSurface = projection.surface;
        let periodic: [bool; 2] = [surface.is_u_periodic()?, surface.is_v_periodic()?];
        let u_period: f64 = if periodic[0] { surface.u_period()? } else { 0.0 };
        let v_period: f64 = if periodic[1] { surface.v_period()? } else { 0.0 };
        Ok(Self {
            base,
            projection,
            bounds,
            periodic,
            period: [u_period, v_period],
            failure: RefCell::new(FailureCapture::default()),
        })
    }
    /// `Value(t)`: the 2D point of parameter `t`.
    pub(super) fn point(&self, t: f64) -> Result<Point2, Failure> {
        self.projection.value(t, &self.bounds)
    }
    /// The first failure a callback kept, after the fit.
    pub(super) fn take_failure(&self) -> Result<(), Failure> {
        self.failure.borrow_mut().take()
    }
    fn keep<T>(&self, value: Result<T, Failure>) -> Option<T> {
        self.failure.borrow_mut().keep(value)
    }
}

impl Function for ProjectionFunction<'_> {
    fn base(&self) -> &FunctionBase {
        &self.base
    }
    fn first_parameter(&self) -> f64 {
        self.keep(self.projection.curve.first_parameter()).unwrap_or(f64::NAN)
    }
    fn last_parameter(&self) -> f64 {
        self.keep(self.projection.curve.last_parameter()).unwrap_or(f64::NAN)
    }
    fn value(&self, t: f64, pnt2d: &mut Array1<'_, Point2>, _pnt: &mut Array1<'_, Point3>) -> bool {
        let Some(point) = self.keep(self.point(t)) else {
            return false;
        };
        pnt2d[1] = point;
        true
    }
    // The native D1 stores the derivative even when it fails (a default gp_Vec2d).
    fn d1(&self, t: f64, vec2d: &mut Array1<'_, Vector2>, _vec: &mut Array1<'_, Vector3>) -> bool {
        let Some(derivative) = self.keep(self.projection.d1(t, &self.bounds)) else {
            return false;
        };
        match derivative {
            Some((_, d)) => {
                vec2d[1] = d;
                true
            }
            None => {
                vec2d[1] = Vector2::default();
                false
            }
        }
    }
    fn period_information(&self, dim_idx: i32) -> (bool, f64) {
        let index: usize = (dim_idx - 1) as usize;
        (self.periodic[index], self.period[index])
    }
}

/// A handle's adaptor or Standard_NullObject.
pub(in crate::proj_lib) fn live_curve(
    curve: &Handle<dyn Adaptor3dCurve>,
) -> Result<&dyn Adaptor3dCurve, Failure> {
    curve.get().ok_or_else(|| {
        failure("Standard_NullObject", "ProjLib: null curve adaptor; expected a curve")
    })
}

/// A handle's adaptor or Standard_NullObject.
pub(in crate::proj_lib) fn live_surface(
    surface: &Handle<dyn Adaptor3dSurface>,
) -> Result<&dyn Adaptor3dSurface, Failure> {
    surface.get().ok_or_else(|| {
        failure("Standard_NullObject", "ProjLib: null surface adaptor; expected a surface")
    })
}
