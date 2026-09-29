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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_SweepApproximation.cxx (Eval, D0, D1, D2, Approx_SweepApproximation_Eval)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The evaluation of the sweep function for AdvApprox: the weights, the transformed
//! 2D poles and the weighted, translated 3D poles (or their derivatives), cached for
//! the last parameter and order.
use crate::approx::sweep_function::{SweepD1, SweepD2, SweepFunction};
use crate::failure_capture::FailureCapture;
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adv_approx::AdvApproxEvaluatorFunction;
use rustycad_tkmath::gp::{GeneralTransform2, Point2, Point3, Vector2, Vector3, Xy};

/// The evaluation state of `Approx_SweepApproximation`: the subspace counts, the 2D
/// affinities, the translation of rational sections, the section arrays and the
/// control variables (`myOrder`, `myParam`, `first`, `last`).
pub(super) struct SweepCache {
    pub(super) func: Handle<dyn SweepFunction>,
    pub(super) num_1d: i32,
    pub(super) num_2d: i32,
    pub(super) num_3d: i32,
    pub(super) a_affin: Array1<'static, GeneralTransform2>,
    pub(super) translation: Vector3,
    pub(super) poles: Array1<'static, Point3>,
    pub(super) d_poles: Array1<'static, Vector3>,
    pub(super) d2_poles: Array1<'static, Vector3>,
    pub(super) weights: Array1<'static, f64>,
    pub(super) d_weights: Array1<'static, f64>,
    pub(super) d2_weights: Array1<'static, f64>,
    pub(super) poles2d: Array1<'static, Point2>,
    pub(super) d_poles2d: Array1<'static, Vector2>,
    pub(super) d2_poles2d: Array1<'static, Vector2>,
    pub(super) order: i32,
    pub(super) param: f64,
    pub(super) first: f64,
    pub(super) last: f64,
}

fn func_of(func: &Handle<dyn SweepFunction>) -> Result<&dyn SweepFunction, Failure> {
    func.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "Approx_SweepApproximation: null sweep function",
        )
    })
}

// The native `+=`/`*=` on gp_XYZ, component by component.
fn weighted(p: Point3, t: Vector3, w: f64) -> Point3 {
    Point3::new(
        (p.x() - t.x()) * w,
        (p.y() - t.y()) * w,
        (p.z() - t.z()) * w,
    )
}

fn translated(p: Point3, t: Vector3) -> Point3 {
    Point3::new(p.x() - t.x(), p.y() - t.y(), p.z() - t.z())
}

fn as_vector(p: Point3) -> Vector3 {
    Vector3::new(p.x(), p.y(), p.z())
}

fn scaled(p: Point3, w: f64) -> Point3 {
    Point3::new(p.x() * w, p.y() * w, p.z() * w)
}

impl SweepCache {
    /// An empty cache before Perform (no subspace; the counts are uninitialized
    /// natively).
    pub(super) fn new(func: Handle<dyn SweepFunction>) -> Self {
        Self {
            func,
            num_1d: 0,
            num_2d: 0,
            num_3d: 0,
            a_affin: Array1::default(),
            translation: Vector3::default(),
            poles: Array1::default(),
            d_poles: Array1::default(),
            d2_poles: Array1::default(),
            weights: Array1::default(),
            d_weights: Array1::default(),
            d2_weights: Array1::default(),
            poles2d: Array1::default(),
            d_poles2d: Array1::default(),
            d2_poles2d: Array1::default(),
            order: -1,
            param: 0.0,
            first: 1.0e100,
            last: -1.0e100,
        }
    }

    /// `Eval(Parameter, DerivativeRequest, First, Last, Result)`: the error code (0,
    /// 1 when the function fails, 2 for an order beyond 2).
    pub(super) fn eval(
        &mut self,
        parameter: f64,
        request: i32,
        [first, last]: [f64; 2],
        result: &mut [f64],
    ) -> Result<i32, Failure> {
        let ok: bool = match request {
            0 => self.d0(parameter, [first, last], result)?,
            1 => self.d1(parameter, [first, last], result)?,
            2 => self.d2(parameter, [first, last], result)?,
            _ => return Ok(2),
        };
        Ok(i32::from(!ok))
    }

    // `(Param != myParam) || (myOrder < order) || (first != First) || (Last != last)`,
    // after SetInterval on a new interval.
    fn stale(&self, param: f64, order: i32, [first, last]: [f64; 2]) -> Result<bool, Failure> {
        let new_interval: bool = self.first != first || last != self.last;
        if new_interval {
            func_of(&self.func)?.set_interval(first, last)?;
        }
        Ok(param != self.param || self.order < order || new_interval)
    }

    fn update(&mut self, param: f64, order: i32, [first, last]: [f64; 2]) {
        (self.first, self.last, self.order, self.param) = (first, last, order, param);
    }

    fn d0(&mut self, param: f64, range: [f64; 2], result: &mut [f64]) -> Result<bool, Failure> {
        let mut ok: bool = true;
        if self.stale(param, 0, range)? {
            let func = self.func.clone();
            ok = func_of(&func)?.d0(
                param,
                range[0],
                range[1],
                &mut self.poles,
                &mut self.poles2d,
                &mut self.weights,
            )?;
            for ii in 1..=self.num_1d {
                self.poles[ii] = weighted(self.poles[ii], self.translation, self.weights[ii]);
            }
            for ii in 1..=self.num_2d {
                let mut coord: Xy = self.poles2d[ii].xy();
                self.a_affin[ii].transforms(&mut coord);
                self.poles2d[ii] = Point2::new(coord.x(), coord.y());
            }
            self.update(param, 0, range);
        }
        let values: Vec<f64> = self.extract(
            &self.weights,
            &self.poles2d.iter().map(|p| p.xy()).collect::<Vec<_>>(),
            &self.poles.iter().map(|p| as_vector(*p)).collect::<Vec<_>>(),
        );
        result[..values.len()].copy_from_slice(&values);
        Ok(ok)
    }

    fn d1(&mut self, param: f64, range: [f64; 2], result: &mut [f64]) -> Result<bool, Failure> {
        let mut ok: bool = true;
        if self.stale(param, 1, range)? {
            let func = self.func.clone();
            let values = SweepD1 {
                poles: &mut self.poles,
                d_poles: &mut self.d_poles,
                poles2d: &mut self.poles2d,
                d_poles2d: &mut self.d_poles2d,
                weights: &mut self.weights,
                d_weights: &mut self.d_weights,
            };
            ok = func_of(&func)?.d1(param, range[0], range[1], values)?;
            for ii in 1..=self.num_1d {
                self.weight_first_order(ii);
            }
            for ii in 1..=self.num_2d {
                self.d_poles2d[ii] = self.transformed_vector(ii, self.d_poles2d[ii]);
                self.transform_pole2d(ii);
            }
            self.update(param, 1, range);
        }
        let values: Vec<f64> = self.extract(
            &self.d_weights,
            &self.d_poles2d.iter().map(|v| v.xy()).collect::<Vec<_>>(),
            &self.d_poles.iter().copied().collect::<Vec<_>>(),
        );
        result[..values.len()].copy_from_slice(&values);
        Ok(ok)
    }

    fn d2(&mut self, param: f64, range: [f64; 2], result: &mut [f64]) -> Result<bool, Failure> {
        let mut ok: bool = true;
        if self.stale(param, 2, range)? {
            let func = self.func.clone();
            let values = SweepD2 {
                d1: SweepD1 {
                    poles: &mut self.poles,
                    d_poles: &mut self.d_poles,
                    poles2d: &mut self.poles2d,
                    d_poles2d: &mut self.d_poles2d,
                    weights: &mut self.weights,
                    d_weights: &mut self.d_weights,
                },
                d2_poles: &mut self.d2_poles,
                d2_poles2d: &mut self.d2_poles2d,
                d2_weights: &mut self.d2_weights,
            };
            ok = func_of(&func)?.d2(param, range[0], range[1], values)?;
            for ii in 1..=self.num_1d {
                self.weight_second_order(ii);
            }
            for ii in 1..=self.num_2d {
                self.d2_poles2d[ii] = self.transformed_vector(ii, self.d2_poles2d[ii]);
                self.d_poles2d[ii] = self.transformed_vector(ii, self.d_poles2d[ii]);
                self.transform_pole2d(ii);
            }
            self.update(param, 2, range);
        }
        let values: Vec<f64> = self.extract(
            &self.d2_weights,
            &self.d2_poles2d.iter().map(|v| v.xy()).collect::<Vec<_>>(),
            &self.d2_poles.iter().copied().collect::<Vec<_>>(),
        );
        result[..values.len()].copy_from_slice(&values);
        Ok(ok)
    }

    // The translated pole, its derivative `w DP + w' P`, and the pole times w.
    fn weight_first_order(&mut self, ii: i32) {
        let pole: Point3 = translated(self.poles[ii], self.translation);
        let w: f64 = self.weights[ii];
        let d_pole: Vector3 = self.d_poles[ii] * w + as_vector(pole) * self.d_weights[ii];
        (self.poles[ii], self.d_poles[ii]) = (scaled(pole, w), d_pole);
    }

    // The second derivative `w D2P + 2 w' DP + w'' P`, then the first order.
    fn weight_second_order(&mut self, ii: i32) {
        let pole: Point3 = translated(self.poles[ii], self.translation);
        let w: f64 = self.weights[ii];
        let d2_pole: Vector3 = self.d2_poles[ii] * w
            + self.d_poles[ii] * (2.0 * self.d_weights[ii])
            + as_vector(pole) * self.d2_weights[ii];
        let d_pole: Vector3 = self.d_poles[ii] * w + as_vector(pole) * self.d_weights[ii];
        (self.poles[ii], self.d_poles[ii], self.d2_poles[ii]) = (scaled(pole, w), d_pole, d2_pole);
    }

    fn transformed_vector(&self, ii: i32, v: Vector2) -> Vector2 {
        let mut coord: Xy = v.xy();
        self.a_affin[ii].transforms(&mut coord);
        Vector2::new(coord.x(), coord.y())
    }

    fn transform_pole2d(&mut self, ii: i32) {
        let mut coord: Xy = self.poles2d[ii].xy();
        self.a_affin[ii].transforms(&mut coord);
        self.poles2d[ii] = Point2::new(coord.x(), coord.y());
    }

    // The result layout: the 1D values, then 2 per 2D pole, then 3 per 3D pole.
    fn extract(&self, one_d: &Array1<'static, f64>, two_d: &[Xy], three_d: &[Vector3]) -> Vec<f64> {
        let mut values: Vec<f64> = Vec::new();
        for ii in 1..=self.num_1d {
            values.push(one_d[ii]);
        }
        for ii in 1..=self.num_2d {
            let xy: Xy = two_d[(ii - 1) as usize];
            values.extend([xy.x(), xy.y()]);
        }
        for ii in 1..=self.num_3d {
            let v: Vector3 = three_d[(ii - 1) as usize];
            values.extend([v.x(), v.y(), v.z()]);
        }
        values
    }
}

/// `Approx_SweepApproximation_Eval`: `Tool.Eval(...)`; a function exception unwinds
/// natively and is carried to the end of AdvApprox here.
pub(super) struct SweepEvaluator<'c> {
    pub(super) cache: &'c mut SweepCache,
    pub(super) capture: FailureCapture,
}

impl AdvApproxEvaluatorFunction for SweepEvaluator<'_> {
    fn evaluate(
        &mut self,
        _: i32,
        start_end: [f64; 2],
        parameter: f64,
        request: i32,
        result: &mut [f64],
    ) -> i32 {
        let code = self.cache.eval(parameter, request, start_end, result);
        self.capture.keep(code).unwrap_or(1)
    }
}
