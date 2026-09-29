// Created on: 1997-08-22
// Created by: Sergey SOKOLOV
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
// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_ApproxSurface.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomConvert_ApproxSurface`: the polynomial B-spline approximation of a surface
//! by AdvApp2Var, cut preferably at its C2, then C3, breaks.
use crate::adv_app2var::{
    AdvApp2VarApproxAFunc2Var, AdvApp2VarEvaluation, AdvApp2VarEvaluatorFunc2Var,
};
use crate::array_access::{grid, put};
use crate::failure_capture::FailureCapture;
use crate::native_failure::failure;
use crate::native_text::stream_real;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::adv_approx::AdvApproxPrefAndRec;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::geom_abs::{IsoType, Shape};
use rustycad_tkmath::gp::Vector3;
use std::cell::RefCell;
use std::fmt;

/// The result of one surface approximation: the B-spline, whether it meets the
/// tolerance, whether there is one at all, and the greatest error.
///
/// `let approx = ApproxSurface::from_surface(&s, 1e-4, (Shape::C1, Shape::C1), (14, 14), 16, 1)?;`
#[derive(Clone)]
pub struct ApproxSurface {
    my_bspl_surf: Handle<GeomBSplineSurface>,
    my_is_done: bool,
    my_has_result: bool,
    my_max_error: f64,
}

impl ApproxSurface {
    /// Approximates `surf` through a GeomAdaptor_Surface within `tol3d`, with the U
    /// and V continuities (C0 to C2), maximum degrees, at most `max_segments`
    /// patches and the AdvApp2Var precision code (`GeomConvert_ApproxSurface(Surf,
    /// Tol3d, UContinuity, VContinuity, MaxDegU, MaxDegV, MaxSegments, PrecisCode)`);
    /// AdvApp2Var raises Standard_ConstructionError for a continuity above C2;
    /// `ApproxSurface::from_surface(&s, 1e-4, (Shape::C1, Shape::C1), (14, 14), 16, 1)?`.
    pub fn from_surface(
        surf: &Handle<dyn GeomSurface>,
        tol3d: f64,
        continuities: (Shape, Shape),
        max_degrees: (i32, i32),
        max_segments: i32,
        precis_code: i32,
    ) -> Result<Self, Failure> {
        let adaptor: Handle<dyn Adaptor3dSurface> =
            Handle::new(GeomAdaptorSurface::with_surface(surf)?).cast::<dyn Adaptor3dSurface>();
        Self::from_adaptor(
            &adaptor,
            tol3d,
            continuities,
            max_degrees,
            max_segments,
            precis_code,
        )
    }

    /// Approximates the adapted `surf` (`GeomConvert_ApproxSurface(const
    /// occ::handle<Adaptor3d_Surface>&, ...)`); an exception of the adaptor propagates;
    /// `ApproxSurface::from_adaptor(&adaptor, 1e-4, (Shape::C1, Shape::C1), (14, 14), 16, 1)?`.
    pub fn from_adaptor(
        surf: &Handle<dyn Adaptor3dSurface>,
        tol3d: f64,
        continuities: (Shape, Shape),
        max_degrees: (i32, i32),
        max_segments: i32,
        precis_code: i32,
    ) -> Result<Self, Failure> {
        let adaptor: &dyn Adaptor3dSurface = surf.get().ok_or_else(|| {
            failure(
                "Standard_NullObject",
                "GeomConvert_ApproxSurface: null adaptor; expected a surface",
            )
        })?;
        let domain = [
            adaptor.first_u_parameter()?,
            adaptor.last_u_parameter()?,
            adaptor.first_v_parameter()?,
            adaptor.last_v_parameter()?,
        ];
        let tolerances = Tolerances::new(tol3d)?;
        let u_dec = AdvApproxPrefAndRec::new(
            &u_cuts(adaptor, Shape::C2)?,
            &u_cuts(adaptor, Shape::C3)?,
            5.0,
        )?;
        let v_dec = AdvApproxPrefAndRec::new(
            &v_cuts(adaptor, Shape::C2)?,
            &v_cuts(adaptor, Shape::C3)?,
            5.0,
        )?;
        let ev = SurfaceEvaluator {
            my_adaptor: RefCell::new(surf.clone()),
            capture: RefCell::default(),
        };
        let approx = AdvApp2VarApproxAFunc2Var::new(
            0,
            0,
            1,
            Some(&tolerances.nul1),
            Some(&tolerances.nul1),
            Some(&tolerances.eps_3d),
            Some(&tolerances.nul2),
            Some(&tolerances.nul2),
            Some(&tolerances.eps_fr),
            domain[0],
            domain[1],
            domain[2],
            domain[3],
            IsoType::IsoV,
            continuities.0,
            continuities.1,
            precis_code,
            max_degrees.0,
            max_degrees.1,
            max_segments,
            &ev,
            &u_dec,
            &v_dec,
        );
        ev.capture.borrow_mut().take()?;
        let approx = approx?;
        Ok(Self {
            my_max_error: approx.max_error(3, 1)?,
            my_bspl_surf: approx.surface(1)?,
            my_is_done: approx.is_done(),
            my_has_result: approx.has_result(),
        })
    }

    /// The approximating B-spline, null without a result (`Surface()`);
    /// `approx.surface()`.
    pub fn surface(&self) -> Handle<GeomBSplineSurface> {
        self.my_bspl_surf.clone()
    }

    /// Whether the approximation meets the tolerance (`IsDone()`); `approx.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    /// Whether there is a result, within the tolerance or not (`HasResult()`);
    /// `approx.has_result()`.
    pub fn has_result(&self) -> bool {
        self.my_has_result
    }

    /// The greatest 3D error of the approximation (`MaxError()`); `approx.max_error()`.
    pub fn max_error(&self) -> f64 {
        self.my_max_error
    }

    /// Writes the native text: a blank line, "No result" or "Result max error :"
    /// with the error as `%g`, then a blank line (`Dump(o)`); `approx.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl fmt::Write) -> fmt::Result {
        writeln!(stream)?;
        if self.my_has_result {
            writeln!(
                stream,
                "Result max error :{}",
                stream_real(self.my_max_error)
            )?;
        } else {
            writeln!(stream, "No result")?;
        }
        writeln!(stream)
    }
}

// The subspace tolerance tables: zero 1D and 2D tolerances, the 3D tolerance, and
// the same on the four frontiers.
struct Tolerances {
    nul1: Array1<'static, f64>,
    nul2: Array2<'static, f64>,
    eps_3d: Array1<'static, f64>,
    eps_fr: Array2<'static, f64>,
}

impl Tolerances {
    fn new(tol3d: f64) -> Result<Self, Failure> {
        let mut nul1: Array1<'static, f64> = one_based(1)?;
        nul1[1] = 0.0;
        let mut eps_3d: Array1<'static, f64> = one_based(1)?;
        eps_3d[1] = tol3d;
        let (mut nul2, mut eps_fr): (Array2<'static, f64>, Array2<'static, f64>) =
            (grid(1, 4)?, grid(1, 4)?);
        for j in 1..=4 {
            put(&mut nul2, 1, j, 0.0, "nul2")?;
            put(&mut eps_fr, 1, j, tol3d, "epsfr")?;
        }
        Ok(Self {
            nul1,
            nul2,
            eps_3d,
            eps_fr,
        })
    }
}

// NbUIntervals(shape) + 1 break parameters in U.
fn u_cuts(surf: &dyn Adaptor3dSurface, shape: Shape) -> Result<Array1<'static, f64>, Failure> {
    let mut cuts: Array1<'static, f64> = one_based(surf.nb_u_intervals(shape)? + 1)?;
    surf.u_intervals(&mut cuts, shape)?;
    Ok(cuts)
}

// NbVIntervals(shape) + 1 break parameters in V.
fn v_cuts(surf: &dyn Adaptor3dSurface, shape: Shape) -> Result<Array1<'static, f64>, Failure> {
    let mut cuts: Array1<'static, f64> = one_based(surf.nb_v_intervals(shape)? + 1)?;
    surf.v_intervals(&mut cuts, shape)?;
    Ok(cuts)
}

/// `GeomConvert_ApproxSurface_Eval`: the derivatives of the adapted surface on an
/// iso, the adaptor re-trimmed to the patch at every call (its mutable member).
///
/// The native switch writes nothing for the orders it does not handle: order 3 with
/// UOrder 0 or 3 (D3 is computed but only D3UUV and D3UVV are copied), and any order
/// above 4, so the kernel's result buffer keeps its previous values; the port leaves
/// `result` untouched in the same cases.
struct SurfaceEvaluator {
    my_adaptor: RefCell<Handle<dyn Adaptor3dSurface>>,
    // An adaptor exception unwinds natively; it is carried to the end of AdvApp2Var.
    capture: RefCell<FailureCapture>,
}

impl AdvApp2VarEvaluatorFunc2Var for SurfaceEvaluator {
    fn evaluate(
        &self,
        request: &AdvApp2VarEvaluation<'_>,
        result: &mut [f64],
        error_code: &mut i32,
    ) {
        *error_code = 0;
        // Dimension incorrecte: the code is set and the evaluation goes on.
        if request.dimension != 3 {
            *error_code = 1;
        }
        let filled: Result<(), Failure> = self
            .retrim(request)
            .and_then(|()| self.fill(request, result));
        if self.capture.borrow_mut().keep(filled).is_none() {
            *error_code = 1;
        }
    }
}

impl SurfaceEvaluator {
    // myAdaptor = myAdaptor->UTrim(...)->VTrim(...), with Precision::PConfusion().
    fn retrim(&self, request: &AdvApp2VarEvaluation<'_>) -> Result<(), Failure> {
        let mut adaptor = self.my_adaptor.borrow_mut();
        let [u0, u1] = request.u_start_end;
        let trimmed = live(&adaptor)?.u_trim(u0, u1, precision::p_confusion())?;
        let [v0, v1] = request.v_start_end;
        *adaptor = live(&trimmed)?.v_trim(v0, v1, precision::p_confusion())?;
        Ok(())
    }

    // The derivative (UOrder, VOrder) at each parameter of the iso, dimension 3.
    fn fill(&self, request: &AdvApp2VarEvaluation<'_>, result: &mut [f64]) -> Result<(), Failure> {
        let adaptor = self.my_adaptor.borrow();
        let surface: &dyn Adaptor3dSurface = live(&adaptor)?;
        let dimension: usize = request.dimension.max(0) as usize;
        for jpar in 0..request.nb_params.max(0) as usize {
            let t: f64 = request.parameters[jpar];
            let (upar, vpar) = if request.favor_iso == 1 {
                (request.const_param, t)
            } else {
                (t, request.const_param)
            };
            if let Some(vect) = derivative(surface, upar, vpar, request.u_order, request.v_order)? {
                let offset: usize = jpar * dimension;
                (result[offset], result[offset + 1], result[offset + 2]) =
                    (vect.x(), vect.y(), vect.z());
            }
        }
        Ok(())
    }
}

// The derivative the native switch copies for Order = UOrder + VOrder; None where it
// copies nothing.
fn derivative(
    surface: &dyn Adaptor3dSurface,
    u: f64,
    v: f64,
    u_order: i32,
    v_order: i32,
) -> Result<Option<Vector3>, Failure> {
    let value = match u_order + v_order {
        0 => Some(Vector3::from(surface.value(u, v)?.xyz())),
        1 => {
            let d1 = surface.d1(u, v)?;
            Some(if u_order == 1 { d1.d1u } else { d1.d1v })
        }
        2 => {
            let d2 = surface.d2(u, v)?;
            match u_order {
                2 => Some(d2.d2u),
                1 => Some(d2.d2uv),
                0 => Some(d2.d2v),
                _ => None,
            }
        }
        3 => {
            let d3 = surface.d3(u, v)?;
            match u_order {
                2 => Some(d3.d3uuv),
                1 => Some(d3.d3uvv),
                _ => None,
            }
        }
        4 => Some(surface.dn(u, v, u_order, v_order)?),
        _ => None,
    };
    Ok(value)
}

fn live(adaptor: &Handle<dyn Adaptor3dSurface>) -> Result<&dyn Adaptor3dSurface, Failure> {
    adaptor.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "GeomConvert_ApproxSurface_Eval: null adaptor",
        )
    })
}
