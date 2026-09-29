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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_CurveOnSurface.cxx (Perform)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_CurveOnSurface::Perform`: the evaluator, tolerances and cutting of the
//! requested curves, then AdvApprox.
use super::evaluator::CurveAndPcurveEvaluator;
use super::{CurveOnSurface, CurveOnSurfaceOptions};
use crate::approx::curve_2d::poles_from_two_1d;
use crate::approx::curve_3d::null_table;
use crate::curve_approx_eval::{Curve2dEvaluator, Curve3dEvaluator, cut_points_3d};
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adaptor3d::h_surface_tool::is_surf_g1;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::adv_approx::{
    AdvApproxApproxAFunction, AdvApproxCutting, AdvApproxDichoCutting, AdvApproxEvaluatorFunction,
    AdvApproxPrefAndRec,
};
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point3;

/// The evaluator Perform selects (`EvalPtr`).
enum Evaluator {
    Curve3d(Curve3dEvaluator),
    Curve2d(Curve2dEvaluator),
    Both(CurveAndPcurveEvaluator),
}

impl Evaluator {
    fn as_dyn(&mut self) -> &mut dyn AdvApproxEvaluatorFunction {
        match self {
            Self::Curve3d(e) => e,
            Self::Curve2d(e) => e,
            Self::Both(e) => e,
        }
    }

    fn take_failure(&mut self) -> Result<(), Failure> {
        match self {
            Self::Curve3d(e) => e.take_failure(),
            Self::Curve2d(e) => e.take_failure(),
            Self::Both(e) => e.take_failure(),
        }
    }
}

/// The tolerances of the 1D (u, v) and 3D subspaces.
struct Tolerances {
    one_d: Option<Array1<'static, f64>>,
    three_d: Option<Array1<'static, f64>>,
}

/// `std::min(a, b)`: `b` only when `b < a`.
fn std_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

impl CurveOnSurface {
    /// `Perform(theMaxSegments, theMaxDegree, theContinuity, theOnly3d, theOnly2d)`:
    /// Standard_ConstructionError when both `only_3d` and `only_2d` are set; the
    /// continuity is reduced to C1/C2 (G1 to C1, G2 and beyond to C2);
    /// `approx.perform(options)?`.
    pub fn perform(&mut self, options: CurveOnSurfaceOptions) -> Result<(), Failure> {
        (self.is_done, self.has_result) = (false, false);
        (self.error_2d_u, self.error_2d_v, self.error_3d) = (0.0, 0.0, 0.0);
        if options.only_3d && options.only_2d {
            return Err(failure(
                "Standard_ConstructionError",
                "Approx_CurveOnSurface::Perform: only3d and only2d are both set",
            ));
        }
        let continuity: Shape = match options.continuity {
            Shape::G1 => Shape::C1,
            Shape::G2 => Shape::C2,
            shape if shape > Shape::C2 => Shape::C2,
            shape => shape,
        };
        let c2d: &dyn Adaptor2dCurve2d = self.c2d.get().ok_or_else(|| null_table("myC2D"))?;
        let trimmed: Handle<dyn Adaptor2dCurve2d> =
            c2d.trim(self.first, self.last, precision::p_confusion())?;
        if options.only_3d && self.iso_line_curve(&trimmed)? {
            (self.is_done, self.has_result) = (true, true);
            return Ok(());
        }
        let hcons: Handle<dyn Adaptor3dCurve> =
            Handle::new(Adaptor3dCurveOnSurface::with_curve(&trimmed, &self.surf)?)
                .cast::<dyn Adaptor3dCurve>();
        let mut evaluator: Evaluator = self.evaluator(&hcons, &trimmed, options);
        let tolerances: Tolerances = self.tolerances(options)?;
        let cut_tool: Box<dyn AdvApproxCutting> = self.cut_tool(&hcons, continuity)?;
        let num = [
            if options.only_3d { 0 } else { 2 },
            0,
            if options.only_2d { 0 } else { 1 },
        ];
        let approx = AdvApproxApproxAFunction::with_cutting(
            num[0],
            num[1],
            num[2],
            tolerances.one_d.as_ref(),
            None,
            tolerances.three_d.as_ref(),
            self.first,
            self.last,
            continuity,
            options.max_degree,
            options.max_segments,
            evaluator.as_dyn(),
            cut_tool.as_ref(),
        );
        evaluator.take_failure()?;
        self.store(&approx?, options)
    }

    fn evaluator(
        &self,
        hcons: &Handle<dyn Adaptor3dCurve>,
        trimmed: &Handle<dyn Adaptor2dCurve2d>,
        options: CurveOnSurfaceOptions,
    ) -> Evaluator {
        let range: [f64; 2] = [self.first, self.last];
        if options.only_3d {
            return Evaluator::Curve3d(Curve3dEvaluator::new(
                hcons.clone(),
                range,
                "Approx_CurveOnSurface_Eval3d",
            ));
        }
        if options.only_2d {
            return Evaluator::Curve2d(Curve2dEvaluator::new(
                trimmed.clone(),
                range,
                false,
                "Approx_CurveOnSurface_Eval2d",
            ));
        }
        Evaluator::Both(CurveAndPcurveEvaluator::new(
            hcons.clone(),
            trimmed.clone(),
            range,
        ))
    }

    // The u and v tolerances (half the resolutions, raised along a C0 direction that
    // is not G1), and the 3D tolerance (half the tolerance).
    fn tolerances(&self, options: CurveOnSurfaceOptions) -> Result<Tolerances, Failure> {
        let mut result = Tolerances {
            one_d: None,
            three_d: None,
        };
        let surf: &dyn Adaptor3dSurface = self.surf.get().ok_or_else(|| null_table("mySurf"))?;
        if !options.only_3d {
            let mut tol_u: f64 = surf.u_resolution(self.tol)? / 2.;
            let mut tol_v: f64 = surf.v_resolution(self.tol)? / 2.;
            if surf.u_continuity()? == Shape::C0 {
                tol_u = self.c0_tolerance(tol_u, true)?;
            }
            if surf.v_continuity()? == Shape::C0 {
                tol_v = self.c0_tolerance(tol_v, false)?;
            }
            let mut one_d: Array1<'static, f64> = one_based(2)?;
            (one_d[1], one_d[2]) = (tol_u, tol_v);
            result.one_d = Some(one_d);
        }
        if !options.only_2d {
            let mut three_d: Array1<'static, f64> = one_based(1)?;
            three_d.init(self.tol / 2.);
            result.three_d = Some(three_d);
        }
        Ok(result)
    }

    // A C0 direction: 1e3 times the tolerance when not G1 at the angular precision,
    // then 1e2 times when not G1 at the confusion precision, each capped at 1e-3.
    fn c0_tolerance(&self, mut tol: f64, along_u: bool) -> Result<f64, Failure> {
        if !is_surf_g1(&self.surf, along_u, precision::angular())? {
            tol = std_min(1.0e-3, 1.0e3 * tol);
        }
        if !is_surf_g1(&self.surf, along_u, precision::confusion())? {
            tol = std_min(1.0e-3, 1.0e2 * tol);
        }
        Ok(tol)
    }

    // Dichotomic cutting when the curve and the surface are continuous enough, else
    // preferred cuts at the C1/C2 (for C1) or C2/C3 breaks of the curve on surface.
    fn cut_tool(
        &self,
        hcons: &Handle<dyn Adaptor3dCurve>,
        continuity: Shape,
    ) -> Result<Box<dyn AdvApproxCutting>, Failure> {
        let c2d: &dyn Adaptor2dCurve2d = self.c2d.get().ok_or_else(|| null_table("myC2D"))?;
        let surf: &dyn Adaptor3dSurface = self.surf.get().ok_or_else(|| null_table("mySurf"))?;
        if continuity <= c2d.continuity()?
            && continuity <= surf.u_continuity()?
            && continuity <= surf.v_continuity()?
        {
            return Ok(Box::new(AdvApproxDichoCutting::new()));
        }
        let hcons: &dyn Adaptor3dCurve = hcons.get().ok_or_else(|| null_table("HCOnS"))?;
        let (recommended, preferred) = if continuity == Shape::C1 {
            (Shape::C1, Shape::C2)
        } else {
            (Shape::C2, Shape::C3)
        };
        Ok(Box::new(AdvApproxPrefAndRec::new(
            &cut_points_3d(hcons, recommended)?,
            &cut_points_3d(hcons, preferred)?,
            5.0,
        )?))
    }

    fn store(
        &mut self,
        approx: &AdvApproxApproxAFunction,
        options: CurveOnSurfaceOptions,
    ) -> Result<(), Failure> {
        self.is_done = approx.is_done();
        self.has_result = approx.has_result();
        if !self.has_result {
            return Ok(());
        }
        let knots = approx.knots().ok_or_else(|| null_table("Knots"))?;
        let mults = approx
            .multiplicities()
            .ok_or_else(|| null_table("Multiplicities"))?;
        let degree: i32 = approx.degree();
        if !options.only_2d {
            let mut poles: Array1<'static, Point3> = one_based(approx.nb_poles()?)?;
            approx.poles_of(1, &mut poles)?;
            self.curve_3d =
                Handle::new(GeomBSplineCurve::new(&poles, knots, mults, degree, false)?);
            self.error_3d = approx.max_error(3, 1)?;
        }
        if !options.only_3d {
            let poles = poles_from_two_1d(approx, [1, 2])?;
            self.curve_2d = Handle::new(Geom2dBSplineCurve::new(
                &poles, knots, mults, degree, false,
            )?);
            self.error_2d_u = approx.max_error(1, 1)?;
            self.error_2d_v = approx.max_error(1, 2)?;
        }
        Ok(())
    }
}
