// Created on: 1995-01-27
// Created by: Jacques GOUSSARD
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
// Rust translation of the approximated curves of the walking case of
// GeomInt_IntSS::MakeCurve (ModelingAlgorithms/TKGeomAlgo/GeomInt/GeomInt_IntSS_1.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The B-spline curves of an approximated walking line: the 3d curve (in the plane
//! from the 2d poles when a surface is a plane) and the 2d curves, with their tangents
//! fixed at both ends and the 2d curves moved into the U period of the surface.
use super::bspline::adjust_u_periodic;
use super::{GeomIntIntSS, GeomIntIntSSApprox};
use crate::geom_int::instances::GeomIntWLApprox;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, arrays};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBoundedCurve, GeomCurve, GeomSurface};
use rustycad_tkgeombase::app_par_curves::{MultiBSpCurve, MultiCurveEval};
use rustycad_tkgeombase::geom_lib::{Check2dBSplineCurve, CheckBSplineCurve};
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Plane3, Point2, Point3};

/// The knots, the multiplicities and the degree of the approximated curve.
struct Spline {
    knots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
    degree: i32,
    nb_poles: i32,
}

impl Spline {
    fn of(mbspc: &MultiBSpCurve) -> Result<Self, Failure> {
        Ok(Self {
            knots: mbspc.knots()?,
            mults: mbspc.multiplicities()?,
            degree: MultiCurveEval::degree(mbspc)?,
            nb_poles: mbspc.nb_poles(),
        })
    }

    /// The 3d B-spline of `poles`, its tangents fixed at both ends.
    fn curve3d(
        &self,
        poles: &Array1<'_, Point3>,
        (tol, ang): (f64, f64),
    ) -> Result<Handle<GeomBSplineCurve>, Failure> {
        let bs = Handle::new(GeomBSplineCurve::new(
            poles,
            &self.knots,
            &self.mults,
            self.degree,
            false,
        )?);
        let mut check = CheckBSplineCurve::new(&bs, tol, ang)?;
        check.fix_tangent(true, true)?;
        Ok(bs)
    }

    /// The 2d B-spline of `poles`, its tangents fixed at both ends and moved into the
    /// U period of `surface`.
    fn curve2d(
        &self,
        poles: &Array1<'_, Point2>,
        (tol, ang): (f64, f64),
        surface: &Handle<dyn GeomSurface>,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let bs = Handle::new(Geom2dBSplineCurve::new(
            poles,
            &self.knots,
            &self.mults,
            self.degree,
            false,
        )?);
        let mut check = Check2dBSplineCurve::new(&bs, tol, ang)?;
        check.fix_tangent(true, true)?;
        let c2d: Handle<dyn Geom2dCurve> = bs.cast();
        adjust_u_periodic(surface, &c2d)?;
        Ok(c2d)
    }

    /// The 2d poles of the curve `index`.
    fn poles2d(
        &self,
        mbspc: &MultiBSpCurve,
        index: i32,
    ) -> Result<Array1<'static, Point2>, Failure> {
        let mut poles: Array1<'static, Point2> = arrays::one_based(self.nb_poles)?;
        mbspc.curve_2d(index, &mut poles)?;
        Ok(poles)
    }
}

impl GeomIntIntSS {
    /// The curves of each multi-curve of a successful approximation.
    pub(super) fn approximated_part_curves(
        &mut self,
        theapp3d: &GeomIntWLApprox,
        (typs1, typs2): (SurfaceType, SurfaceType),
        approx: GeomIntIntSSApprox,
    ) -> Result<(), Failure> {
        for j in 1..=theapp3d.nb_multi_curves() {
            let mbspc: &MultiBSpCurve = theapp3d.value(j);
            if typs1 == SurfaceType::Plane {
                self.plane_curves(mbspc, 1, approx)?;
            } else if typs2 == SurfaceType::Plane {
                self.plane_curves(mbspc, 2, approx)?;
            } else {
                self.space_curves(mbspc, approx)?;
            }
        }
        Ok(())
    }

    /// The tangent check tolerances.
    fn check_tolerances(&self) -> (f64, f64) {
        (self.my_tol_check, self.my_tol_ang_check)
    }

    /// A surface of the intersection (its Geom surface).
    fn geom_surface(&self, k: i32) -> Handle<dyn GeomSurface> {
        if k == 1 {
            self.my_hs1.surface()
        } else {
            self.my_hs2.surface()
        }
    }

    /// The curves when the surface `k` is a plane: the 3d curve from the 2d poles on the
    /// plane, the 2d curve on the plane from them, the other 2d curve when asked.
    fn plane_curves(
        &mut self,
        mbspc: &MultiBSpCurve,
        k: i32,
        approx: GeomIntIntSSApprox,
    ) -> Result<(), Failure> {
        let spline = Spline::of(mbspc)?;
        let tols = self.check_tolerances();
        let (on_plane_wanted, other_wanted) = if k == 1 {
            (approx.approx_s1, approx.approx_s2)
        } else {
            (approx.approx_s2, approx.approx_s1)
        };
        // With the plane second, its curve is the second one when S1 is approximated.
        let plane_index: i32 = if k == 2 && approx.approx_s1 { 2 } else { 1 };
        let tpoles2d = spline.poles2d(mbspc, plane_index)?;
        let pln: Plane3 = if k == 1 {
            self.my_hs1.plane()?
        } else {
            self.my_hs2.plane()?
        };
        let mut tpoles: Array1<'static, Point3> = arrays::one_based(spline.nb_poles)?;
        for ik in 1..=spline.nb_poles {
            let p: Point2 = arrays::at(&tpoles2d, ik)?;
            arrays::put(&mut tpoles, ik, elslib::value(p.x(), p.y(), &pln))?;
        }
        let bs = spline.curve3d(&tpoles, tols)?;
        self.sline.push(bs.cast());
        let on_plane: Handle<dyn Geom2dCurve> = if on_plane_wanted {
            spline.curve2d(&tpoles2d, tols, &self.geom_surface(k))?
        } else {
            Handle::default()
        };
        let other_k: i32 = 3 - k;
        let other: Handle<dyn Geom2dCurve> = if other_wanted {
            let other_index: i32 = if k == 1 { 2 } else { 1 };
            let poles = spline.poles2d(mbspc, other_index)?;
            spline.curve2d(&poles, tols, &self.geom_surface(other_k))?
        } else {
            Handle::default()
        };
        if k == 1 {
            self.sline_s1.push(on_plane);
            self.sline_s2.push(other);
        } else {
            self.sline_s2.push(on_plane);
            self.sline_s1.push(other);
        }
        Ok(())
    }

    /// The curves when no surface is a plane: the 3d curve (closed when its ends
    /// coincide, dropped when it is a segment of two coincident poles) and the 2d curves
    /// when asked.
    fn space_curves(
        &mut self,
        mbspc: &MultiBSpCurve,
        approx: GeomIntIntSSApprox,
    ) -> Result<(), Failure> {
        let spline = Spline::of(mbspc)?;
        let tols = self.check_tolerances();
        let mut tpoles: Array1<'static, Point3> = arrays::one_based(spline.nb_poles)?;
        mbspc.curve_3d(1, &mut tpoles)?;
        let bs = spline.curve3d(&tpoles, tols)?;
        // Check IsClosed().
        let start: Point3 = bs.start_point()?;
        let end: Point3 = bs.end_point()?;
        let dist: f64 = rustycad_tkg2d::geom2d::native::std_max(
            start.xyz().square_modulus(),
            end.xyz().square_modulus(),
        );
        let eps: f64 = epsilon(dist);
        if start.square_distance(&end) < 2.0 * eps {
            // Avoid creating B-splines containing two coincident poles only.
            if spline.degree == 1 && spline.nb_poles == 2 {
                return Ok(());
            }
            if !bs.is_closed()? && !bs.is_periodic() {
                // Force Closed().
                let pm = Point3::from((bs.pole(1)?.xyz() + bs.pole(bs.nb_poles())?.xyz()) / 2.0);
                bs.set_pole(1, pm)?;
                bs.set_pole(bs.nb_poles(), pm)?;
            }
        }
        self.sline.push(bs.cast());
        let c1: Handle<dyn Geom2dCurve> = if approx.approx_s1 {
            spline.curve2d(&spline.poles2d(mbspc, 2)?, tols, &self.geom_surface(1))?
        } else {
            Handle::default()
        };
        self.sline_s1.push(c1);
        let c2: Handle<dyn Geom2dCurve> = if approx.approx_s2 {
            let index: i32 = if approx.approx_s1 { 3 } else { 2 };
            spline.curve2d(&spline.poles2d(mbspc, index)?, tols, &self.geom_surface(2))?
        } else {
            Handle::default()
        };
        self.sline_s2.push(c2);
        Ok(())
    }
}
