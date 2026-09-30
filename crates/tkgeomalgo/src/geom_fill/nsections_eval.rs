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
// Rust translation of D0, D1, D2 and the static ResultEval of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_NSections.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The evaluators of `GeomFill_NSections`: the V iso curve of the surface and the
//! V derivatives of its poles, from a B-spline evaluation of the flattened
//! (homogeneous) pole rows.
use super::nsections::GeomFillNSections;
use super::section_law::{SectionD1, SectionD2};
use super::sweep_failures::{live, sweep_failure};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomGeometry, GeomSurface};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{Point3, Vector3, Xyz};

fn index_failure(array: &str, index: i32, e: impl std::fmt::Display) -> Failure {
    sweep_failure(
        "Standard_OutOfRange",
        &format!(
            "GeomFill_NSections: {array}({index}): {e}; expected an index in the array bounds"
        ),
    )
}

/// The homogeneous pole rows of the surface, V row after V row.
fn flat_poles(surf: &GeomBSplineSurface, rational: bool) -> Result<Vec<f64>, Failure> {
    let mut s_poles = Vec::new();
    for jj in 1..=surf.nb_v_poles() {
        for ii in 1..=surf.nb_u_poles() {
            let p = surf.pole(ii, jj)?;
            if rational {
                let w = surf.weight(ii, jj)?;
                s_poles.extend_from_slice(&[p.x() * w, p.y() * w, p.z() * w, w]);
            } else {
                s_poles.extend_from_slice(&[p.x(), p.y(), p.z()]);
            }
        }
    }
    Ok(s_poles)
}

/// The static `ResultEval(surf, V, deriv, Result)`: the `deriv`-th V derivative of
/// the flattened pole rows (`BSplCLib::Eval`, extrapolated with the degree).
fn result_eval(surf: &GeomBSplineSurface, v: f64, deriv: i32) -> Result<Vec<f64>, Failure> {
    let rational = surf.is_v_rational();
    let gap = if rational { 4 } else { 3 };
    let cdeg = surf.v_degree();
    let cdim = surf.nb_u_poles() * gap;
    let f_knots = surf.v_knot_sequence();
    let s_poles = flat_poles(surf, rational)?;
    let mut eval_bs = vec![0.0; (cdim * (deriv + 1)) as usize];
    let extrap_mode = [cdeg, cdeg];
    bsplclib::eval_extrapolated(
        v,
        false,
        deriv,
        &extrap_mode,
        cdeg,
        &f_knots,
        cdim,
        &s_poles,
        &mut eval_bs,
    )?;
    let start = (deriv * cdim) as usize;
    Ok(eval_bs[start..start + cdim as usize].to_vec())
}

/// The surface the derivatives are evaluated on: a non-periodic copy of a V
/// periodic surface.
fn derivative_surface(
    surface: &Handle<GeomBSplineSurface>,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let s = live(surface, "GeomFill_NSections::D1", "mySurface")?;
    if !s.is_v_periodic() {
        return Ok(surface.clone());
    }
    let surf_deper: Handle<GeomBSplineSurface> = s.copy()?.cast();
    live(&surf_deper, "GeomFill_NSections::D1", "surf_deper")?.set_v_not_periodic()?;
    Ok(surf_deper)
}

fn result_at(result: &[f64], index: usize) -> Result<f64, Failure> {
    result
        .get(index)
        .copied()
        .ok_or_else(|| index_failure("Result", index as i32 + 1, "outside the evaluation"))
}

fn xyz_at(result: &[f64], indice: usize) -> Result<Xyz, Failure> {
    Ok(Xyz::new(
        result_at(result, indice)?,
        result_at(result, indice + 1)?,
        result_at(result, indice + 2)?,
    ))
}

fn value<T: Copy>(array: &Array1<'_, T>, name: &str, ii: i32) -> Result<T, Failure> {
    array
        .value(ii)
        .copied()
        .map_err(|e| index_failure(name, ii, e))
}

fn set<T>(array: &mut Array1<'_, T>, name: &str, ii: i32, v: T) -> Result<(), Failure> {
    array
        .set_value(ii, v)
        .map_err(|e| index_failure(name, ii, e))
}

/// The poles, weights and first derivatives of a section, read by the second
/// derivatives.
struct FirstOrder<'a, 'b> {
    poles: &'a Array1<'b, Point3>,
    d_poles: &'a Array1<'b, Vector3>,
    weights: &'a Array1<'b, f64>,
    d_weights: &'a Array1<'b, f64>,
}

/// The second derivatives of the poles from the evaluated rows `result` (weighted
/// back by the rational terms); false, after writing the raw value, at a weight
/// below 10 PConfusion().
fn second_order_poles(
    result: &[f64],
    rational: bool,
    first: &FirstOrder<'_, '_>,
    d2_poles: &mut Array1<'_, Vector3>,
    d2_weights: &mut Array1<'_, f64>,
) -> Result<bool, Failure> {
    if !rational {
        d2_weights.init(0.0);
    }
    let gap = if rational { 4 } else { 3 };
    for ii in 1..=first.poles.length() {
        let indice = ((ii - 1) * gap) as usize;
        let mut d2p = xyz_at(result, indice)?;
        if rational {
            let ww = value(first.weights, "Weights", ii)?;
            if ww < eps_w() {
                set(d2_poles, "D2Poles", ii, Vector3::from(d2p))?;
                return Ok(false);
            }
            let d2w = result_at(result, indice + 3)?;
            set(d2_weights, "D2Weights", ii, d2w)?;
            d2p = rational_second(first, ii, d2p, d2w, ww)?;
        }
        set(d2_poles, "D2Poles", ii, Vector3::from(d2p))?;
    }
    Ok(true)
}

/// `(D2P - D2W P - 2 DW DP) / W` of pole `ii`.
fn rational_second(
    first: &FirstOrder<'_, '_>,
    ii: i32,
    d2p: Xyz,
    d2w: f64,
    ww: f64,
) -> Result<Xyz, Failure> {
    let p = value(first.poles, "Poles", ii)?.xyz();
    let dw = value(first.d_weights, "DWeights", ii)?;
    let dp = *value(first.d_poles, "DPoles", ii)?.xyz();
    Ok((d2p - d2w * p - (2.0 * dw) * dp) / ww)
}

/// The weight below which a derivative is not computed (`EpsW`).
fn eps_w() -> f64 {
    10.0 * precision::p_confusion()
}

impl GeomFillNSections {
    /// `D0(V, Poles, Weights)`: the poles and weights of the V iso curve; false
    /// without a surface.
    pub(crate) fn nsections_d0(
        &self,
        v: f64,
        poles: &mut Array1<'_, Point3>,
        weights: &mut Array1<'_, f64>,
    ) -> Result<bool, Failure> {
        let surface = self.surface();
        let Some(s) = surface.get() else {
            return Ok(false);
        };
        let curve = s.v_iso_checked(v, false)?;
        let (c_poles, c_weights) = (curve.poles(), curve.weights_array(Clone::clone));
        for ii in 1..=poles.length() {
            set(poles, "Poles", ii, value(&c_poles, "poles", ii)?)?;
            set(weights, "Weights", ii, value(&c_weights, "weights", ii)?)?;
        }
        Ok(true)
    }

    /// `D1`: with the V derivatives; false without a surface or at a weight below
    /// 10 PConfusion() (the poles before it are written).
    pub(crate) fn nsections_d1(&self, v: f64, out: SectionD1<'_, '_>) -> Result<bool, Failure> {
        let surface = self.surface();
        let Some(s) = surface.get() else {
            return Ok(false);
        };
        if !self.nsections_d0(v, out.poles, out.weights)? {
            return Ok(false);
        }
        let rational = s.is_v_rational();
        let result = result_eval(live(&derivative_surface(&surface)?, "D1", "surf")?, v, 1)?;
        if !rational {
            out.d_weights.init(0.0);
        }
        let gap = if rational { 4 } else { 3 };
        for ii in 1..=out.poles.length() {
            let indice = ((ii - 1) * gap) as usize;
            let mut dp = xyz_at(&result, indice)?;
            if rational {
                let ww = value(out.weights, "Weights", ii)?;
                if ww < eps_w() {
                    set(out.d_poles, "DPoles", ii, Vector3::from(dp))?;
                    return Ok(false);
                }
                let dw = result_at(&result, indice + 3)?;
                set(out.d_weights, "DWeights", ii, dw)?;
                dp = (dp - dw * value(out.poles, "Poles", ii)?.xyz()) / ww;
            }
            set(out.d_poles, "DPoles", ii, Vector3::from(dp))?;
        }
        Ok(true)
    }

    /// `D2`: with the second V derivatives; false as `D1`, or when the V degree is
    /// below 2.
    pub(crate) fn nsections_d2(&self, v: f64, out: SectionD2<'_, '_>) -> Result<bool, Failure> {
        let surface = self.surface();
        let Some(s) = surface.get() else {
            return Ok(false);
        };
        if s.v_degree() < 2 {
            return Ok(false);
        }
        let SectionD2 {
            d1,
            d2_poles,
            d2_weights,
        } = out;
        let (poles, d_poles, weights, d_weights) = (d1.poles, d1.d_poles, d1.weights, d1.d_weights);
        let first = SectionD1 {
            poles: &mut *poles,
            d_poles: &mut *d_poles,
            weights: &mut *weights,
            d_weights: &mut *d_weights,
        };
        if !self.nsections_d1(v, first)? {
            return Ok(false);
        }
        let first = FirstOrder {
            poles,
            d_poles,
            weights,
            d_weights,
        };
        let result = result_eval(live(&derivative_surface(&surface)?, "D2", "surf")?, v, 2)?;
        second_order_poles(&result, s.is_v_rational(), &first, d2_poles, d2_weights)
    }
}
