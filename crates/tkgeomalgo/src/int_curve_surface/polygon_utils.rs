// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of IntCurveSurface_PolygonUtils.pxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurveSurface_PolygonUtils`: the discretization of a curve into a polygon,
//! generic over the curve tool (docs/geomalgo-design.md section 11).
use super::failures::out_of_bounds;
use super::tools::InterCurveTool;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Direction3, Line3, Point3, Vector3};

/// The sampled points of a polygon with its box and deflection estimate, the
/// outputs of `InitUniform` and `InitWithParams`.
pub struct PolygonSamples {
    /// `thePnts`: the points, `points[i - 1]` for index `i`.
    pub points: Vec<Point3>,
    /// `theBnd`: the box of the points, enlarged by the deflection.
    pub bnd: BoundingBox3,
    /// `theDeflection`: the largest distance of a mid-parameter point to its chord.
    pub deflection: f64,
}

/// `InitUniform`: `nb_pnt_in` points from `binf` to `bsup` by equal steps, the box
/// enlarged by 1.5 times the deflection (by 1e-10 for up to 3 points);
/// `init_uniform::<TheHCurveTool>(&curve, 0.0, 1.0, 5)?`.
pub fn init_uniform<C: InterCurveTool>(
    curve: &C::Curve,
    binf: f64,
    bsup: f64,
    nb_pnt_in: i32,
) -> Result<PolygonSamples, Failure> {
    let du: f64 = (bsup - binf) / f64::from(nb_pnt_in - 1);
    let mut samples = PolygonSamples {
        points: Vec::new(),
        bnd: BoundingBox3::default(),
        deflection: 0.0,
    };
    let mut u: f64 = binf;
    for _ in 1..=nb_pnt_in {
        let p: Point3 = C::d0(curve, u)?;
        samples.bnd.add_point(p);
        samples.points.push(p);
        u += du;
    }
    if nb_pnt_in > 3 {
        let mut u: f64 = binf + du * 0.5;
        let mut mids: Vec<f64> = Vec::new();
        for _ in 1..nb_pnt_in {
            mids.push(u);
            u += du;
        }
        estimate_deflection::<C>(curve, &mids, &mut samples)?;
    } else {
        samples.bnd.enlarge(1e-10);
    }
    Ok(samples)
}

/// `InitWithParams`: the points at the first `nb_pnt_in` parameters of `upars`, the
/// mid parameters of each pair for the deflection; the parameters are copied from
/// index 1 (`theParams`); `init_with_params::<TheHCurveTool>(&curve, &upars, 5)?`.
pub fn init_with_params<C: InterCurveTool>(
    curve: &C::Curve,
    upars: &Array1<'_, f64>,
    nb_pnt_in: i32,
) -> Result<(PolygonSamples, Array1<'static, f64>), Failure> {
    let mut params: Array1<'static, f64> = Array1::with_bounds(1, upars.length())
        .map_err(|e| e.into_failure("NCollection_HArray1"))?;
    let i0: i32 = upars.lower() - 1;
    let mut samples = PolygonSamples {
        points: Vec::new(),
        bnd: BoundingBox3::default(),
        deflection: 0.0,
    };
    for i in 1..=nb_pnt_in {
        let u: f64 = param(upars, i + i0)?;
        params[i] = u;
        let p: Point3 = C::d0(curve, u)?;
        samples.bnd.add_point(p);
        samples.points.push(p);
    }
    if nb_pnt_in > 3 {
        let mut mids: Vec<f64> = Vec::new();
        for i in 1..nb_pnt_in {
            mids.push(0.5 * (param(upars, i0 + i)? + param(upars, i0 + i + 1)?));
        }
        estimate_deflection::<C>(curve, &mids, &mut samples)?;
    } else {
        samples.bnd.enlarge(1e-10);
    }
    Ok((samples, params))
}

/// `upars(index)`: Standard_OutOfRange outside the array.
fn param(upars: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    upars
        .value(index)
        .copied()
        .map_err(|e| e.into_failure("NCollection_Array1::Value"))
}

/// The deflection: the largest distance of the point at `mids[i - 1]` to the line
/// from point `i` toward point `i + 1` (gp_Dir raises Standard_ConstructionError
/// for two equal points); the box is enlarged by 1.5 times it.
fn estimate_deflection<C: InterCurveTool>(
    curve: &C::Curve,
    mids: &[f64],
    samples: &mut PolygonSamples,
) -> Result<(), Failure> {
    samples.deflection = 0.0;
    for (i, &u) in mids.iter().enumerate() {
        let pm: Point3 = C::value(curve, u)?;
        let (p1, p2) = (samples.points[i], samples.points[i + 1]);
        let l = Line3::new(p1, Direction3::from_vector(Vector3::between(p1, p2))?);
        let t: f64 = l.distance(pm);
        if t > samples.deflection {
            samples.deflection = t;
        }
    }
    samples.bnd.enlarge(1.5 * samples.deflection);
    Ok(())
}

/// `ApproxParamOnCurve(Index, ParamOnLine, Binf, Bsup, NbPntIn, Params)`: the curve
/// parameter at `param_on_line` along segment `index`, from the uniform step or the
/// stored parameters (Standard_OutOfRange when they have no such segment). A
/// position outside `[0, 1]` gives `binf + param_on_line * (bsup - binf) / (n - 1)`
/// whatever the segment, and the end of the last point is the end of the last
/// segment; `approx_param_on_curve(2, 0.5, 0.0, 1.0, 5, None)?`.
pub fn approx_param_on_curve(
    index: i32,
    param_on_line: f64,
    binf: f64,
    bsup: f64,
    nb_pnt_in: i32,
    params: Option<&Array1<'_, f64>>,
) -> Result<f64, Failure> {
    if !(0.0..=1.0).contains(&param_on_line) {
        return Ok(binf + (param_on_line * (bsup - binf)) / f64::from(nb_pnt_in - 1));
    }
    let (mut index, mut param_on_line) = (index, param_on_line);
    if index == nb_pnt_in && param_on_line == 0.0 {
        index -= 1;
        param_on_line = 1.0;
    }
    let (du, u): (f64, f64) = match params {
        None => {
            let du: f64 = (bsup - binf) / f64::from(nb_pnt_in - 1);
            (du, binf + du * f64::from(index - 1))
        }
        Some(params) => {
            let at = |i: i32| {
                params
                    .value(i)
                    .copied()
                    .map_err(|_| stored_out_of_range(i, params))
            };
            (at(index + 1)? - at(index)?, at(index)?)
        }
    };
    Ok(u + du * param_on_line)
}

/// The Standard_OutOfRange of `myParams->Value(i)`.
fn stored_out_of_range(index: i32, params: &Array1<'_, f64>) -> Failure {
    out_of_bounds(
        "IntCurveSurface_PolygonUtils::ApproxParamOnCurve",
        index,
        params.lower(),
        params.upper(),
    )
}
