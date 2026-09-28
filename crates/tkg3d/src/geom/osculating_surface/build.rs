// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG3d/Geom/Geom_OsculatingSurface.cxx (buildOsculatingSurface) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! One osculating B-spline patch: the polynomial of the span at the degenerate
//! boundary, reduced by one degree across it (the derivative of the collapsing
//! direction), converted back to poles.
use crate::geom::errors;
use crate::geom::{GeomBSplineSurface, GeomSurface};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bsplslib::{self, CacheSource};
use rustycad_tkmath::convert::{GridPolynomialToPoles, PolynomialGrid};
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::plib;

fn array1<T: Default + Clone>(count: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(1, count).map_err(errors::array_bounds)
}

fn array2<T: Default + Clone>(rows: i32, cols: i32) -> Result<Array2<'static, T>, Failure> {
    Array2::with_bounds(1, rows, 1, cols).map_err(errors::array_bounds)
}

/// The direction of the degenerate boundary.
#[derive(Clone, Copy)]
pub(super) struct Along {
    pub(super) u: bool,
    pub(super) v: bool,
}

// The Taylor cache of the span starting at knots (`u_knot`, `v_knot`), moved to the
// far end of the span across the boundary when `param` lies in its second half.
fn span_cache(
    surface: &GeomBSplineSurface,
    param: f64,
    knots: (i32, i32),
    along: Along,
) -> Result<(Array2<'static, Point3>, bool, bool), Failure> {
    let (u_deg, v_deg) = (surface.u_degree(), surface.v_degree());
    let mut cache_poles = array2::<Point3>(u_deg.max(v_deg) + 1, u_deg.min(v_deg) + 1)?;
    let (mut u_param, mut v_param) = (surface.u_knot(knots.0)?, surface.v_knot(knots.1)?);
    let u_span: f64 = surface.u_knot(knots.0 + 1)? - u_param;
    let v_span: f64 = surface.v_knot(knots.1 + 1)? - v_param;
    let is_v_neg: bool = param > v_param + v_span / 2.;
    let is_u_neg: bool = param > u_param + u_span / 2.;
    if along.u && is_v_neg {
        v_param += v_span;
    }
    if along.v && is_u_neg {
        u_param += u_span;
    }
    let (flat_u, flat_v, poles) = (
        surface.u_knot_sequence(),
        surface.v_knot_sequence(),
        surface.poles(),
    );
    let source = CacheSource {
        u: u_param,
        v: v_param,
        u_span_domain: u_span,
        v_span_domain: v_span,
        u_periodic: surface.is_u_periodic(),
        v_periodic: surface.is_v_periodic(),
        u_degree: u_deg,
        v_degree: v_deg,
        u_index: 0,
        v_index: 0,
        u_flat_knots: &flat_u,
        v_flat_knots: &flat_v,
    };
    bsplslib::build_cache(&source, &poles, None, &mut cache_poles, None)?;
    Ok((cache_poles, is_u_neg, is_v_neg))
}

// The coefficients of the reduced patch, laid out as the native loops do.
fn reduced_coefficients(
    cache: &Array2<'static, Point3>,
    degrees: (i32, i32),
    along: Along,
    negated: (bool, bool),
) -> Result<(Array1<'static, f64>, (i32, i32)), Failure> {
    let (u_deg, v_deg) = degrees;
    let (u_poles, v_poles) = (u_deg as usize + 1, v_deg as usize + 1);
    // The osculating coefficient counts: the along-U layout is overwritten by the
    // along-V one when both are set, as natively.
    let mut counts = (0, 0);
    if along.u {
        counts = (u_deg + 1, v_deg);
    }
    if along.v {
        counts = (u_deg, v_deg + 1);
    }
    let nb_coef: i32 = counts.0 * counts.1 * 3;
    let mut osc = array2::<Point3>(counts.0, counts.1)?;
    let transposed: bool = u_deg <= v_deg;
    let (rows, cols) = if along.u {
        (u_poles, v_poles - 1)
    } else {
        (u_poles - 1, v_poles)
    };
    for n in 0..rows {
        for m in 0..cols {
            let (a, b) = if along.u { (n, m + 1) } else { (n + 1, m) };
            let (r, c) = if transposed { (b, a) } else { (a, b) };
            osc[(n as i32 + 1, m as i32 + 1)] = cache[(r as i32 + 1, c as i32 + 1)];
        }
    }
    if along.u && negated.1 {
        plib::v_trimming(-1., 0., &mut osc, None)?;
    }
    if !along.u && negated.0 {
        plib::u_trimming(-1., 0., &mut osc, None)?;
    }
    let mut coefficients = array1::<f64>(nb_coef)?;
    let mut index: i32 = 1;
    for n in 0..rows {
        for m in 0..cols {
            let pole: Point3 = osc[(n as i32 + 1, m as i32 + 1)];
            for value in [pole.x(), pole.y(), pole.z()] {
                coefficients[index] = value;
                index += 1;
            }
        }
    }
    Ok((coefficients, counts))
}

/// `buildOsculatingSurface`: `None` when a degree is too low or the patch is empty.
pub(super) fn build_osculating_surface(
    param: f64,
    knots: (i32, i32),
    surface: &GeomBSplineSurface,
    along: Along,
) -> Result<Option<Handle<GeomBSplineSurface>>, Failure> {
    let (u_deg, v_deg) = (surface.u_degree(), surface.v_degree());
    if (along.u && v_deg <= 1) || (along.v && u_deg <= 1) {
        return Ok(None);
    }
    // No coefficient without a degenerate direction: the native aNbCoef == 0 return.
    if !along.u && !along.v {
        return Ok(None);
    }
    let (cache, is_u_neg, is_v_neg) = span_cache(surface, param, knots, along)?;
    let (coefficients, counts) =
        reduced_coefficients(&cache, (u_deg, v_deg), along, (is_u_neg, is_v_neg))?;
    let mut num_coeff = Array2::<i32>::with_bounds(1, 1, 1, 2).map_err(errors::array_bounds)?;
    num_coeff[(1, 1)] = counts.0;
    num_coeff[(1, 2)] = counts.1;
    let mut poly_u = array1::<f64>(2)?;
    let mut poly_v = array1::<f64>(2)?;
    let mut true_u = array1::<f64>(2)?;
    let mut true_v = array1::<f64>(2)?;
    for i in 0..2 {
        poly_u[i + 1] = f64::from(i);
        poly_v[i + 1] = f64::from(i);
        true_u[i + 1] = surface.u_knot(knots.0 + i)?;
        true_v[i + 1] = surface.v_knot(knots.1 + i)?;
    }
    let grid = PolynomialGrid {
        nb_u_surfaces: 1,
        nb_v_surfaces: 1,
        u_continuity: -1,
        v_continuity: -1,
        max_u_degree: if along.v { u_deg - 1 } else { u_deg },
        max_v_degree: if along.u { v_deg - 1 } else { v_deg },
        num_coeff_per_surface: &num_coeff,
        coefficients: &coefficients,
        polynomial_u_intervals: &poly_u,
        polynomial_v_intervals: &poly_v,
        true_u_intervals: &true_u,
        true_v_intervals: &true_v,
    };
    let data = GridPolynomialToPoles::from_grid(&grid)?;
    let reduced = GeomBSplineSurface::new(
        data.poles()?,
        (data.u_knots()?, data.u_multiplicities()?),
        (data.v_knots()?, data.v_multiplicities()?),
        (data.u_degree()?, data.v_degree()?),
        (false, false),
    )?;
    Ok(Some(Handle::new(reduced)))
}
