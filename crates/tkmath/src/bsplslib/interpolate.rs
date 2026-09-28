// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/BSplSLib/BSplSLib_EvaluatorFunction.hxx and BSplSLib.cxx (Interpolate, FunctionMultiply) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Surface interpolation by two passes of curve interpolation (iso U, then iso V),
//! and products of a surface with a scalar function.
use super::eval::homogeneous_d0;
use super::prepare::SurfaceSpan;
use crate::bsplclib::{build_schoenberg_points, interpolate_flat};
use crate::gp::Point3;
use crate::plib::access::{grid_cell, grid_value};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

/// A scalar function a(u, v) (`BSplSLib_EvaluatorFunction`).
pub trait EvaluatorFunction {
    /// Evaluates derivative `derivative_request` at (u, v), setting `error_code`
    /// non-zero on failure; `function.evaluate(0, 0.5, 0.5, &mut value, &mut code)`.
    fn evaluate(
        &self,
        derivative_request: i32,
        u: f64,
        v: f64,
        result: &mut f64,
        error_code: &mut i32,
    );
}

/// Interpolation data shared by both passes.
pub struct InterpolationGrid<'r, 'a> {
    /// U degree.
    pub u_degree: i32,
    /// V degree.
    pub v_degree: i32,
    /// U flat knots.
    pub u_flat_knots: &'r Array1<'a, f64>,
    /// V flat knots.
    pub v_flat_knots: &'r Array1<'a, f64>,
    /// U parameters.
    pub u_parameters: &'r Array1<'a, f64>,
    /// V parameters.
    pub v_parameters: &'r Array1<'a, f64>,
}

fn one_based(count: i32, width: i32) -> Result<Vec<f64>, Failure> {
    usize::try_from(i64::from(count) * i64::from(width))
        .map(|size| vec![0.0; size])
        .map_err(|_| {
            Failure::out_of_range(&format!("BSplSLib::Interpolate: {count} x {width} values"))
        })
}

fn contact_orders(count: i32) -> Result<Array1<'static, i32>, Failure> {
    let mut orders: Array1<'static, i32> = Array1::with_bounds(1, count)
        .map_err(|error| Failure::out_of_range(&format!("BSplSLib::Interpolate: {error}")))?;
    orders.init(0);
    Ok(orders)
}

// Interpolates rows of `width`-value poles laid out per (U index, V index) with
// `dim` values each; `values(jj, ii)` reads the data point (U jj, V ii).
fn interpolate_grid(
    grid: &InterpolationGrid<'_, '_>,
    dim: usize,
    values: impl Fn(i32, i32) -> Result<Vec<f64>, Failure>,
    inversion_problem: &mut i32,
) -> Result<Option<Vec<f64>>, Failure> {
    let (u_length, v_length) = (
        grid.u_parameters.len() as i32,
        grid.v_parameters.len() as i32,
    );
    // Iso U: one row per V parameter holding the U data points.
    let width: usize = dim * u_length as usize;
    let mut points: Vec<f64> = one_based(v_length, width as i32)?;
    for ii in 1..=v_length {
        for jj in 1..=u_length {
            let start: usize = (ii - 1) as usize * width + (jj - 1) as usize * dim;
            points[start..start + dim].copy_from_slice(&values(jj, ii)?);
        }
    }
    interpolate_flat(
        grid.v_degree,
        grid.v_flat_knots,
        grid.v_parameters,
        &contact_orders(v_length)?,
        width as i32,
        &mut points,
        inversion_problem,
    )?;
    if *inversion_problem != 0 {
        return Ok(None);
    }
    // Iso V: one row per U parameter holding the V results.
    let width2: usize = dim * v_length as usize;
    let mut iso: Vec<f64> = one_based(u_length, width2 as i32)?;
    for ii in 0..u_length as usize {
        for jj in 0..v_length as usize {
            let (target, source) = (ii * width2 + jj * dim, jj * width + ii * dim);
            iso[target..target + dim].copy_from_slice(&points[source..source + dim]);
        }
    }
    interpolate_flat(
        grid.u_degree,
        grid.u_flat_knots,
        grid.u_parameters,
        &contact_orders(u_length)?,
        width2 as i32,
        &mut iso,
        inversion_problem,
    )?;
    Ok(Some(iso))
}

/// Replaces the data points `poles(i, j)` (at `(UParameters(i), VParameters(j))`) and
/// their weights by the poles and weights of the interpolating surface, homogeneously
/// (`BSplSLib::Interpolate` with Weights). Poles and weights are read and written from
/// index (1, 1); `bsplslib::interpolate_rational(&grid, &mut poles, &mut weights, &mut problem)?`.
pub fn interpolate_rational(
    grid: &InterpolationGrid<'_, '_>,
    poles: &mut Array2<'_, Point3>,
    weights: &mut Array2<'_, f64>,
    inversion_problem: &mut i32,
) -> Result<(), Failure> {
    let read = |jj: i32, ii: i32| -> Result<Vec<f64>, Failure> {
        let p: Point3 = grid_value(poles, jj, ii)?;
        Ok(vec![p.x(), p.y(), p.z(), grid_value(weights, jj, ii)?])
    };
    let Some(iso) = interpolate_grid(grid, 4, read, inversion_problem)? else {
        return Ok(());
    };
    let v_length: usize = grid.v_parameters.len();
    for ii in 0..grid.u_parameters.len() {
        for jj in 0..v_length {
            let at: usize = (ii * v_length + jj) * 4;
            let (row, col) = (ii as i32 + 1, jj as i32 + 1);
            *grid_cell(poles, row, col)? = Point3::new(iso[at], iso[at + 1], iso[at + 2]);
            *grid_cell(weights, row, col)? = iso[at + 3];
        }
    }
    Ok(())
}

/// Replaces the data points by the poles of the interpolating surface
/// (`BSplSLib::Interpolate`); `bsplslib::interpolate(&grid, &mut poles, &mut problem)?`.
pub fn interpolate(
    grid: &InterpolationGrid<'_, '_>,
    poles: &mut Array2<'_, Point3>,
    inversion_problem: &mut i32,
) -> Result<(), Failure> {
    let read = |jj: i32, ii: i32| -> Result<Vec<f64>, Failure> {
        let p: Point3 = grid_value(poles, jj, ii)?;
        Ok(vec![p.x(), p.y(), p.z()])
    };
    let Some(iso) = interpolate_grid(grid, 3, read, inversion_problem)? else {
        return Ok(());
    };
    let v_length: usize = grid.v_parameters.len();
    for ii in 0..grid.u_parameters.len() {
        for jj in 0..v_length {
            let at: usize = (ii * v_length + jj) * 3;
            *grid_cell(poles, ii as i32 + 1, jj as i32 + 1)? =
                Point3::new(iso[at], iso[at + 1], iso[at + 2]);
        }
    }
    Ok(())
}

/// The B-spline surface multiplied by `function`.
pub struct MultiplySource<'r, 'a> {
    /// The surface to multiply; its parameters and index hints are ignored.
    pub surface: SurfaceSpan<'r, 'a>,
    /// Poles of the surface.
    pub poles: &'r Array2<'a, Point3>,
    /// Weights of the surface; required, as natively the evaluation reads them always.
    pub weights: Option<&'r Array2<'a, f64>>,
}

/// Numerator and denominator of `a(u,v) * S(u,v)` interpolated at the Schoenberg points
/// of new flat knots (`BSplSLib::FunctionMultiply`). Fails with
/// Standard_ConstructionError when the output grids are not
/// `(#UFlat - UNewDegree - 1) x (#VFlat - VNewDegree - 1)` or `a` fails;
/// `bsplslib::function_multiply(&a, &source, &new_knots, &mut numerator, &mut denominator, &mut status)?`.
pub fn function_multiply(
    function: &dyn EvaluatorFunction,
    source: &MultiplySource<'_, '_>,
    new_knots: &InterpolationKnots<'_, '_>,
    new_numerator: &mut Array2<'_, Point3>,
    new_denominator: &mut Array2<'_, f64>,
    status: &mut i32,
) -> Result<(), Failure> {
    let num_u: i32 = new_knots.u_flat_knots.len() as i32 - new_knots.u_new_degree - 1;
    let num_v: i32 = new_knots.v_flat_knots.len() as i32 - new_knots.v_new_degree - 1;
    let mut u_parameters: Array1<'static, f64> = parameters(num_u)?;
    let mut v_parameters: Array1<'static, f64> = parameters(num_v)?;
    let shapes: [usize; 4] = [
        new_numerator.nb_rows(),
        new_numerator.nb_columns(),
        new_denominator.nb_rows(),
        new_denominator.nb_columns(),
    ];
    if shapes
        != [
            num_u as usize,
            num_v as usize,
            num_u as usize,
            num_v as usize,
        ]
    {
        return Err(construction(&format!(
            "output grids {shapes:?}; expected {num_u} x {num_v}"
        )));
    }
    build_schoenberg_points(
        new_knots.u_new_degree,
        new_knots.u_flat_knots,
        &mut u_parameters,
    )?;
    build_schoenberg_points(
        new_knots.v_new_degree,
        new_knots.v_flat_knots,
        &mut v_parameters,
    )?;
    for ii in 1..=num_u {
        for jj in 1..=num_v {
            let (u, v) = (u_parameters[ii], v_parameters[jj]);
            multiply_sample(
                function,
                source,
                (ii, jj),
                (u, v),
                new_numerator,
                new_denominator,
            )?;
        }
    }
    let grid = InterpolationGrid {
        u_degree: new_knots.u_new_degree,
        v_degree: new_knots.v_new_degree,
        u_flat_knots: new_knots.u_flat_knots,
        v_flat_knots: new_knots.v_flat_knots,
        u_parameters: &u_parameters,
        v_parameters: &v_parameters,
    };
    interpolate_rational(&grid, new_numerator, new_denominator, status)
}

/// Flat knots and degrees of the product surface.
pub struct InterpolationKnots<'r, 'a> {
    /// New U flat knots.
    pub u_flat_knots: &'r Array1<'a, f64>,
    /// New V flat knots.
    pub v_flat_knots: &'r Array1<'a, f64>,
    /// New U degree.
    pub u_new_degree: i32,
    /// New V degree.
    pub v_new_degree: i32,
}

fn parameters(count: i32) -> Result<Array1<'static, f64>, Failure> {
    Array1::with_bounds(1, count)
        .map_err(|error| Failure::out_of_range(&format!("BSplSLib::FunctionMultiply: {error}")))
}

fn construction(message: &str) -> Failure {
    Failure::with_type(
        "Standard_ConstructionError",
        &format!("BSplSLib::FunctionMultiply: {message}"),
        "",
    )
}

// Homogeneous surface point scaled by a(u, v), with index hints 0 and both directions
// declared rational and non-periodic, as natively.
fn multiply_sample(
    function: &dyn EvaluatorFunction,
    source: &MultiplySource<'_, '_>,
    (ii, jj): (i32, i32),
    (u, v): (f64, f64),
    new_numerator: &mut Array2<'_, Point3>,
    new_denominator: &mut Array2<'_, f64>,
) -> Result<(), Failure> {
    let span = SurfaceSpan {
        u,
        v,
        u_index: 0,
        v_index: 0,
        u_rat: true,
        v_rat: true,
        u_per: false,
        v_per: false,
        ..source.surface
    };
    let (w, p) = homogeneous_d0(&span, source.poles, source.weights)?;
    *grid_cell(new_denominator, ii, jj)? = w;
    *grid_cell(new_numerator, ii, jj)? = p;
    let (mut result, mut error_code): (f64, i32) = (0.0, 0);
    function.evaluate(0, u, v, &mut result, &mut error_code);
    if error_code != 0 {
        return Err(construction(&format!(
            "the function failed at ({u}, {v}) with code {error_code}"
        )));
    }
    *grid_cell(new_numerator, ii, jj)? =
        Point3::new(p.x() * result, p.y() * result, p.z() * result);
    *grid_cell(new_denominator, ii, jj)? *= result;
    Ok(())
}
