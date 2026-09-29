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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/Law/Law_BSpline.cxx (the evaluators,
// LocateU, MovePointAndTangent and Resolution) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The evaluators of `Law_BSpline`.
use super::{LawBSpline, LawBSplineData};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg2d::geom2d::spline_data::locate_inside;
use rustycad_tkmath::bsplclib::{self, CurveSpan};

impl LawBSplineData {
    /// The evaluation span of `u` from knot hint `index` on the flat knots.
    fn span(&self, u: f64, index: i32) -> CurveSpan<'_, 'static> {
        CurveSpan {
            u,
            index,
            degree: self.deg,
            periodic: self.periodic,
            knots: &self.flatknots,
            mults: None,
        }
    }

    /// The weights passed to BSplCLib: the stored ones of a rational law.
    fn eval_weights(&self) -> Option<&Array1<'static, f64>> {
        self.weights.as_ref().filter(|_| self.rational)
    }

    /// The flat knot index of `U` between knots `from_k1` and `to_k2`, and `U` located
    /// there, as the `Local*` evaluators compute them.
    fn local_span(
        &self,
        member: &str,
        u: f64,
        from_k1: i32,
        to_k2: i32,
    ) -> Result<(i32, f64), Failure> {
        if from_k1 == to_k2 {
            return Err(errors::domain(&format!(
                "Law_BSpline::{member}: FromK1 = ToK2 = {from_k1}; expected distinct knots"
            )));
        }
        let (mut index, mut located): (i32, f64) = (0, u);
        bsplclib::locate_parameter_flat_knots(
            self.deg,
            &self.flatknots,
            u,
            self.periodic,
            from_k1,
            to_k2,
            &mut index,
            &mut located,
        )?;
        let flat: i32 = bsplclib::flat_index(self.deg, index, &self.mults, self.periodic)?;
        Ok((flat, located))
    }

    pub(super) fn d0(&self, u: f64, index: i32) -> Result<f64, Failure> {
        bsplclib::d0_scalar(&self.span(u, index), &self.poles, self.eval_weights())
    }

    pub(super) fn d1(&self, u: f64, index: i32) -> Result<(f64, f64), Failure> {
        bsplclib::d1_scalar(&self.span(u, index), &self.poles, self.eval_weights())
    }

    fn d2(&self, u: f64, index: i32) -> Result<(f64, f64, f64), Failure> {
        bsplclib::d2_scalar(&self.span(u, index), &self.poles, self.eval_weights())
    }

    fn d3(&self, u: f64, index: i32) -> Result<(f64, f64, f64, f64), Failure> {
        bsplclib::d3_scalar(&self.span(u, index), &self.poles, self.eval_weights())
    }

    fn dn(&self, u: f64, n: i32, index: i32) -> Result<f64, Failure> {
        bsplclib::dn_scalar(&self.span(u, index), n, &self.poles, self.eval_weights())
    }

    /// `LocateU`. The two end tests compare the given `U`, not the normalized one, with
    /// the first and last knots, as natively.
    fn locate_u(
        &self,
        u: f64,
        tolerance: f64,
        with_repetition: bool,
    ) -> Result<(i32, i32), Failure> {
        let knots: &Array1<'static, f64> = if with_repetition {
            &self.flatknots
        } else {
            &self.knots
        };
        let new_u: f64 = self.periodic_normalization(u);
        let count: i32 = knots.len() as i32;
        let (first, last) = (knots.as_slice()[0], knots.as_slice()[count as usize - 1]);
        let tol: f64 = tolerance.abs();
        if (u - first).abs() <= tol {
            return Ok((1, 1));
        }
        if (u - last).abs() <= tol {
            return Ok((count, count));
        }
        if new_u < first - tol {
            return Ok((0, 1));
        }
        if new_u > last + tol {
            return Ok((count, count + 1));
        }
        locate_inside(knots, new_u, tol)
    }
}

impl LawBSpline {
    /// The value at `u` (`Value`); `law.value(0.5)?`.
    pub fn value(&self, u: f64) -> Result<f64, Failure> {
        self.d0(u)
    }

    /// The value at `u`, periodic parameters normalized first (`D0`); `law.d0(0.5)?`.
    pub fn d0(&self, u: f64) -> Result<f64, Failure> {
        let data = self.state.read();
        data.d0(data.periodic_normalization(u), 0)
    }

    /// The value and first derivative (`D1`); `law.d1(0.5)?`.
    pub fn d1(&self, u: f64) -> Result<(f64, f64), Failure> {
        let data = self.state.read();
        data.d1(data.periodic_normalization(u), 0)
    }

    /// The value and two derivatives (`D2`); `law.d2(0.5)?`.
    pub fn d2(&self, u: f64) -> Result<(f64, f64, f64), Failure> {
        let data = self.state.read();
        data.d2(data.periodic_normalization(u), 0)
    }

    /// The value and three derivatives (`D3`); `law.d3(0.5)?`.
    pub fn d3(&self, u: f64) -> Result<(f64, f64, f64, f64), Failure> {
        let data = self.state.read();
        data.d3(data.periodic_normalization(u), 0)
    }

    /// The derivative of order `n` at `u` (`DN`); unlike `D0` to `D3`, `u` is not
    /// normalized on a periodic law, as natively. `law.dn(0.5, 2)?`.
    pub fn dn(&self, u: f64, n: i32) -> Result<f64, Failure> {
        self.state.read().dn(u, n, 0)
    }

    /// The value on the span between knots `from_k1` and `to_k2` (`LocalValue`);
    /// Standard_DomainError when `from_k1 == to_k2`. `law.local_value(1.0, 1, 2)?`.
    pub fn local_value(&self, u: f64, from_k1: i32, to_k2: i32) -> Result<f64, Failure> {
        self.local_d0(u, from_k1, to_k2)
    }

    /// The value on a span (`LocalD0`); `law.local_d0(1.0, 1, 2)?`.
    pub fn local_d0(&self, u: f64, from_k1: i32, to_k2: i32) -> Result<f64, Failure> {
        let data = self.state.read();
        let (index, located) = data.local_span("LocalValue", u, from_k1, to_k2)?;
        data.d0(located, index)
    }

    /// The value and first derivative on a span (`LocalD1`); `law.local_d1(1.0, 1, 2)?`.
    pub fn local_d1(&self, u: f64, from_k1: i32, to_k2: i32) -> Result<(f64, f64), Failure> {
        let data = self.state.read();
        let (index, located) = data.local_span("LocalD1", u, from_k1, to_k2)?;
        data.d1(located, index)
    }

    /// The value and two derivatives on a span (`LocalD2`); `law.local_d2(1.0, 1, 2)?`.
    pub fn local_d2(&self, u: f64, from_k1: i32, to_k2: i32) -> Result<(f64, f64, f64), Failure> {
        let data = self.state.read();
        let (index, located) = data.local_span("LocalD2", u, from_k1, to_k2)?;
        data.d2(located, index)
    }

    /// The value and three derivatives on a span (`LocalD3`); `law.local_d3(1.0, 1, 2)?`.
    pub fn local_d3(
        &self,
        u: f64,
        from_k1: i32,
        to_k2: i32,
    ) -> Result<(f64, f64, f64, f64), Failure> {
        let data = self.state.read();
        let (index, located) = data.local_span("LocalD3", u, from_k1, to_k2)?;
        data.d3(located, index)
    }

    /// The derivative of order `n` on a span (`LocalDN`; its native message names
    /// LocalD3). `law.local_dn(1.0, 1, 2, 2)?`.
    pub fn local_dn(&self, u: f64, from_k1: i32, to_k2: i32, n: i32) -> Result<f64, Failure> {
        let data = self.state.read();
        let (index, located) = data.local_span("LocalD3", u, from_k1, to_k2)?;
        data.dn(located, n, index)
    }

    /// The knot interval `(I1, I2)` of `u` within `parametric_tolerance` (`LocateU`;
    /// the native default of `WithKnotRepetition` is false): equal indices on a knot,
    /// `(0, 1)` before the first and `(N, N + 1)` after the last.
    /// `law.locate_u(0.5, 1e-9, false)?`.
    pub fn locate_u(
        &self,
        u: f64,
        parametric_tolerance: f64,
        with_knot_repetition: bool,
    ) -> Result<(i32, i32), Failure> {
        self.state
            .read()
            .locate_u(u, parametric_tolerance, with_knot_repetition)
    }

    /// Moves the poles so that the law passes through `p` with derivative `tangent` at
    /// `u` (`MovePointAndTangent`); returns the BSplCLib error status and keeps the
    /// poles unchanged unless it is 0. `law.move_point_and_tangent(0.5, 1.0, 0.0, 1e-7, 1, 1)?`.
    pub fn move_point_and_tangent(
        &self,
        u: f64,
        p: f64,
        tangent: f64,
        tolerance: f64,
        starting_condition: i32,
        ending_condition: i32,
    ) -> Result<i32, Failure> {
        let mut data = self.state.write();
        let (value, derivative) = data.d1(data.periodic_normalization(u), 0)?;
        let delta: f64 = p - value;
        let delta_derivative: f64 = tangent - derivative;
        let mut new_poles: Vec<f64> = vec![0.0; data.poles.len()];
        let mut error_status: i32 = 0;
        bsplclib::move_point_and_tangent_flat(
            u,
            1,
            &[delta],
            &[delta_derivative],
            tolerance,
            data.deg,
            starting_condition,
            ending_condition,
            data.poles.as_slice(),
            data.eval_weights(),
            &data.flatknots,
            &mut new_poles,
            &mut error_status,
        )?;
        if error_status == 0 {
            data.poles.as_mut_slice().copy_from_slice(&new_poles);
        }
        Ok(error_status)
    }

    /// The parametric tolerance matching `tolerance_3d` (`Resolution`);
    /// `law.resolution(1e-7)?`.
    pub fn resolution(&self, tolerance_3d: f64) -> Result<f64, Failure> {
        let data = self.state.read();
        bsplclib::resolution_flat(
            data.poles.as_slice(),
            1,
            data.poles.len() as i32,
            data.eval_weights(),
            &data.flatknots,
            data.deg,
            tolerance_3d,
        )
    }
}
