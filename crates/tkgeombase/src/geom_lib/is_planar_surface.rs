// Created on: 1998-11-23
// Created by: Philippe MANGIN
// Copyright (c) 1998-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib_IsPlanarSurface.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLib_IsPlanarSurface`: whether a surface is planar within a tolerance, and
//! its plane.
use super::inertia::{InertiaAxes, inertia};
use super::live;
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface, GeomSurfaceD1};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkmath::geom_abs::{CurveType, Shape, SurfaceType};
use rustycad_tkmath::gp::{self, Axis3, Direction3, Plane3, Point3, Vector3};
use std::f64::consts::PI;

/// The planarity of a surface and, when planar, its plane.
///
/// `let test = IsPlanarSurface::new(&surface, 1e-7)?; if test.is_planar() { let plane = test.plan()?; }`
#[derive(Clone, Copy, Debug)]
pub struct IsPlanarSurface {
    my_plan: Plane3,
    is_plan: bool,
}

impl IsPlanarSurface {
    /// Tests `s` within `tol` (`GeomLib_IsPlanarSurface(S, Tol = 1e-7)`): a plane is
    /// planar and quadrics are not; a revolution whose normal (at the middle, or 10%
    /// further when a derivative vanishes) is within 0.1 rad of its axis, or an
    /// extrusion whose normal is normal to its direction within `tol / (Vmax -
    /// Vmin)`, is tested along an isoline (Umin, resp. the middle V); any other
    /// surface is sampled on `8 + 3 NbIntervals(CN)` points per direction and fitted
    /// by its inertia axes; `IsPlanarSurface::new(&surface, 1e-7)?`.
    pub fn new(s: &Handle<dyn GeomSurface>, tol: f64) -> Result<Self, Failure> {
        let adaptor = GeomAdaptorSurface::with_surface(s)?;
        let surface: &dyn GeomSurface = live(s, "GeomLib_IsPlanarSurface")?;
        let mut test = Self {
            my_plan: Plane3::default(),
            is_plan: false,
        };
        match adaptor.get_type()? {
            SurfaceType::Plane => {
                test.is_plan = true;
                test.my_plan = adaptor.plane()?;
            }
            SurfaceType::Cylinder
            | SurfaceType::Cone
            | SurfaceType::Sphere
            | SurfaceType::Torus => {}
            SurfaceType::SurfaceOfRevolution => test.revolution(surface, &adaptor, tol)?,
            SurfaceType::SurfaceOfExtrusion => test.extrusion(surface, &adaptor, tol)?,
            _ => test.sampled(surface, &adaptor, tol)?,
        }
        Ok(test)
    }

    // The revolution: planar when its normal is along the axis and its Umin isoline
    // lies in the plane through the middle point.
    fn revolution(
        &mut self,
        s: &dyn GeomSurface,
        adaptor: &GeomAdaptorSurface,
        tol: f64,
    ) -> Result<(), Failure> {
        let mut dir: Direction3 = adaptor.axe_of_revolution()?.direction();
        let (umin, _, _, _) = s.bounds();
        let d1: GeomSurfaceD1 = middle_d1(s, |d| d.magnitude() <= gp::resolution())?;
        let dn: Vector3 = d1.d1u.crossed(d1.d1v);
        let mut essai: bool = true;
        if dn.magnitude() > 1.0e-7 {
            let mut angle: f64 = dir.angle(&Direction3::from_vector(dn)?);
            if angle > PI / 2.0 {
                angle = PI - angle;
                dir.reverse();
            }
            essai = angle < 0.1;
        }
        if !essai {
            self.is_plan = false;
            return Ok(());
        }
        let mut axe = Axis3::from_normal(d1.point, dir)?;
        axe.set_x_direction(Direction3::from_vector(d1.d1u)?)?;
        self.my_plan.set_position(axe);
        self.my_plan.set_location(d1.point);
        self.is_plan = controle_curve(&s.u_iso(umin)?, &self.my_plan, tol)?;
        Ok(())
    }

    // The extrusion: planar when its normal is normal to the direction within
    // tol / (Vmax - Vmin) and its middle V isoline lies in the plane.
    fn extrusion(
        &mut self,
        s: &dyn GeomSurface,
        adaptor: &GeomAdaptorSurface,
        tol: f64,
    ) -> Result<(), Failure> {
        let (_, _, vmin, vmax) = s.bounds();
        let d1: GeomSurfaceD1 = middle_d1(s, |d| d.magnitude() <= gp::resolution())?;
        let mut dn: Vector3 = d1.d1u.crossed(d1.d1v);
        let norm: f64 = dn.magnitude();
        let mut essai: bool = false;
        if norm > 1.0e-15 {
            dn /= norm;
            let angmax: f64 = tol / (vmax - vmin);
            let d: Direction3 = Direction3::from_vector(dn)?;
            essai = d.is_normal(adaptor.direction()?, angmax);
        }
        if !essai {
            self.is_plan = false;
            return Ok(());
        }
        let axe = Axis3::new(
            d1.point,
            Direction3::from_vector(dn)?,
            Direction3::from_vector(d1.d1u)?,
        )?;
        self.my_plan.set_position(axe);
        self.my_plan.set_location(d1.point);
        self.is_plan = controle_curve(&s.v_iso((vmin + vmax) / 2.0)?, &self.my_plan, tol)?;
        Ok(())
    }

    // Any other surface: a grid of points fitted by their inertia axes.
    fn sampled(
        &mut self,
        s: &dyn GeomSurface,
        adaptor: &GeomAdaptorSurface,
        tol: f64,
    ) -> Result<(), Failure> {
        let nb_u: i32 = 8 + 3 * adaptor.nb_u_intervals(Shape::CN)?;
        let nb_v: i32 = 8 + 3 * adaptor.nb_v_intervals(Shape::CN)?;
        let (umin, umax, vmin, vmax) = s.bounds();
        let du: f64 = (umax - umin) / f64::from(nb_u - 1);
        let dv: f64 = (vmax - vmin) / f64::from(nb_v - 1);
        let mut pnts: Array1<'static, Point3> = one_based(nb_u * nb_v)?;
        let mut kk: i32 = 1;
        for ii in 0..nb_u {
            let u: f64 = umin + du * f64::from(ii);
            for jj in 0..nb_v {
                let v: f64 = vmin + dv * f64::from(jj);
                pnts[kk] = s.d0(u, v)?;
                kk += 1;
            }
        }
        self.is_plan = controle_points(&pnts, tol, s, &mut self.my_plan)?;
        Ok(())
    }

    /// Whether the surface is planar (`IsPlanar()`); `test.is_planar()`.
    pub fn is_planar(&self) -> bool {
        self.is_plan
    }

    /// The plane; StdFail_NotDone when the surface is not planar (`Plan()`);
    /// `test.plan()?`.
    pub fn plan(&self) -> Result<Plane3, Failure> {
        if !self.is_plan {
            return Err(failure("StdFail_NotDone", " GeomLib_IsPlanarSurface"));
        }
        Ok(self.my_plan)
    }
}

// D1 at the middle of the bounds, or 10% of the range further when a derivative
// is degenerate.
fn middle_d1(
    s: &dyn GeomSurface,
    degenerate: impl Fn(Vector3) -> bool,
) -> Result<GeomSurfaceD1, Failure> {
    let (umin, umax, vmin, vmax) = s.bounds();
    let d1: GeomSurfaceD1 = s.d1((umin + umax) / 2.0, (vmin + vmax) / 2.0)?;
    if !(degenerate(d1.d1u) || degenerate(d1.d1v)) {
        return Ok(d1);
    }
    let new_u: f64 = (umin + umax) / 2.0 + (umax - umin) * 0.1;
    let new_v: f64 = (vmin + vmax) / 2.0 + (vmax - vmin) * 0.1;
    s.d1(new_u, new_v)
}

// `Controle(Poles, Tol, S, Plan)`: the points are planar when their smallest
// dispersion is below `tol` and the middle one exceeds Precision::Confusion(); the
// plane is then oriented along the surface derivatives at the middle.
fn controle_points(
    poles: &Array1<'static, Point3>,
    tol: f64,
    s: &dyn GeomSurface,
    plan: &mut Plane3,
) -> Result<bool, Failure> {
    let axes: InertiaAxes = inertia(poles)?;
    if !(axes.z_gap < tol && axes.y_gap > precision::confusion()) {
        return Ok(false);
    }
    let (umin, umax, vmin, vmax) = s.bounds();
    let d1: GeomSurfaceD1 = s.d1((umin + umax) / 2.0, (vmin + vmax) / 2.0)?;
    if !(d1.d1u.square_magnitude() > gp::resolution()
        && d1.d1v.square_magnitude() > gp::resolution())
    {
        return Ok(false);
    }
    let (mut dx, mut dy): (Direction3, Direction3) = (axes.x_dir, axes.y_dir);
    let mut du: Direction3 = Direction3::from_vector(d1.d1u)?;
    let mut angle1: f64 = du.angle(&dx);
    let mut angle2: f64 = du.angle(&dy);
    if angle1 > PI / 2.0 {
        angle1 = PI - angle1;
    }
    if angle2 > PI / 2.0 {
        angle2 = PI - angle2;
    }
    if angle2 < angle1 {
        du = dy;
        dy = dx;
        dx = du;
    }
    if dx.angle(&Direction3::from_vector(d1.d1u)?) > PI / 2.0 {
        dx.reverse();
    }
    if dy.angle(&Direction3::from_vector(d1.d1v)?) > PI / 2.0 {
        dy.reverse();
    }
    let axe = Axis3::new(axes.bary, dx.crossed(dy)?, dx)?;
    plan.set_position(axe);
    plan.set_location(axes.bary);
    Ok(true)
}

// `Controle(C, Plan, Tol)`: every sample of the curve (2 for a line, 3 for a circle,
// 5 for a conic, one per pole for a Bezier or B-spline, 8 + 3 NbIntervals(CN)
// otherwise) is within `tol` of the plane.
fn controle_curve(c: &Handle<dyn GeomCurve>, plan: &Plane3, tol: f64) -> Result<bool, Failure> {
    let ac = GeomAdaptorCurve::with_curve(c)?;
    let nb: i32 = match ac.get_type()? {
        CurveType::Line => 2,
        CurveType::Circle => 3,
        CurveType::Ellipse | CurveType::Hyperbola | CurveType::Parabola => 5,
        CurveType::BezierCurve | CurveType::BSplineCurve => ac.nb_poles()?,
        _ => 8 + 3 * ac.nb_intervals(Shape::CN)?,
    };
    let curve: &dyn GeomCurve = live(c, "GeomLib_IsPlanarSurface")?;
    let (f, l): (f64, f64) = (ac.first_parameter()?, ac.last_parameter()?);
    let du: f64 = (l - f) / f64::from(nb - 1);
    let mut b: bool = true;
    let mut ii: i32 = 1;
    while ii <= nb && b {
        let u: f64 = f64::from(ii - 1) * du + f;
        b = plan.distance(curve.value(u)?) < tol;
        ii += 1;
    }
    Ok(b)
}
