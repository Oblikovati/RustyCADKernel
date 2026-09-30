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
// Rust translation of GeomFill_NetworkSurface.cxx (denominatorValue,
// DenominatorProductEvaluator, makeTripleProductBasis, multiplyByDenominators,
// makeCorrectedProfileSkin) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corrected profile skin: the pole sum of the aligned surfaces, exactly through
//! common denominators when one of them is rational.
use super::network_surface::NetworkSurfaceStatus;
use super::network_surface_skin::point_of;
use super::network_surface_skin::{WeightedGrid, grid};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomBSplineSurface;
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::bsplslib::{
    self, EvaluatorFunction, InterpolationKnots, MultiplySource, SurfaceSpan,
};
use rustycad_tkmath::gp::Point3;
use std::cell::RefCell;

/// The knots, multiplicities and weights of a surface, read once.
struct SurfaceData {
    poles: Array2<'static, Point3>,
    weights: Array2<'static, f64>,
    u_knots: Array1<'static, f64>,
    v_knots: Array1<'static, f64>,
    u_mults: Array1<'static, i32>,
    v_mults: Array1<'static, i32>,
    u_degree: i32,
    v_degree: i32,
}

impl SurfaceData {
    fn of(s: &GeomBSplineSurface) -> Self {
        Self {
            poles: s.poles(),
            weights: s.weights_array(|w| w.clone()),
            u_knots: s.u_knots(),
            v_knots: s.v_knots(),
            u_mults: s.u_multiplicities(),
            v_mults: s.v_multiplicities(),
            u_degree: s.u_degree(),
            v_degree: s.v_degree(),
        }
    }
    // The span of BSplSLib::HomogeneousD0 at (u, v): no index hints, rational in both
    // directions, not periodic.
    fn span(&self, u: f64, v: f64) -> SurfaceSpan<'_, 'static> {
        SurfaceSpan {
            u,
            v,
            u_index: 0,
            v_index: 0,
            u_knots: &self.u_knots,
            v_knots: &self.v_knots,
            u_mults: Some(&self.u_mults),
            v_mults: Some(&self.v_mults),
            u_degree: self.u_degree,
            v_degree: self.v_degree,
            u_rat: true,
            v_rat: true,
            u_per: false,
            v_per: false,
        }
    }
    /// `denominatorValue`: the homogeneous weight at (u, v).
    fn denominator_value(&self, u: f64, v: f64) -> Result<f64, Failure> {
        let (w, _) = bsplslib::homogeneous_d0(&self.span(u, v), &self.poles, Some(&self.weights))?;
        Ok(w)
    }
}

/// `DenominatorProductEvaluator`: the product of the denominators of two surfaces; an
/// exception of the evaluation is kept (it unwinds through BSplSLib::FunctionMultiply
/// natively) and reported as a failed evaluation.
struct DenominatorProductEvaluator<'d> {
    my_first: &'d SurfaceData,
    my_second: &'d SurfaceData,
    capture: RefCell<FailureCapture>,
}

impl EvaluatorFunction for DenominatorProductEvaluator<'_> {
    fn evaluate(
        &self,
        derivative_request: i32,
        u: f64,
        v: f64,
        result: &mut f64,
        error_code: &mut i32,
    ) {
        if derivative_request != 0 {
            *error_code = 1;
            return;
        }
        let product = self
            .my_first
            .denominator_value(u, v)
            .and_then(|first| Ok(first * self.my_second.denominator_value(u, v)?));
        match self.capture.borrow_mut().keep(product) {
            Some(value) => {
                *result = value;
                *error_code = 0;
            }
            None => *error_code = 1,
        }
    }
}

/// The product basis of one direction (`makeTripleProductBasis`): degree 3 d, end
/// knots of multiplicity 3 d + 1, interior ones min(3 d, 2 d + m); None when the degree
/// exceeds Geom_BSplineSurface::MaxDegree().
struct ProductBasis {
    degree: i32,
    knots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
    flat_knots: Array1<'static, f64>,
}

fn make_triple_product_basis(
    the_knots: &Array1<'_, f64>,
    the_mults: &Array1<'_, i32>,
    the_degree: i32,
) -> Result<Option<ProductBasis>, Failure> {
    let degree: i32 = 3 * the_degree;
    if degree > GeomBSplineSurface::max_degree() {
        return Ok(None);
    }
    let n: i32 = the_knots.len() as i32;
    let mut knots: Array1<'static, f64> = super::network_surface_skin::array(n as usize)?;
    let mut mults: Array1<'static, i32> = super::network_surface_skin::array(n as usize)?;
    let mut a_flat_length: i32 = 0;
    for k in 0..n {
        knots[k + 1] = the_knots[the_knots.lower() + k];
        let m: i32 = if k == 0 || k + 1 == n {
            degree + 1
        } else {
            degree.min(2 * the_degree + the_mults[the_mults.lower() + k])
        };
        mults[k + 1] = m;
        a_flat_length += m;
    }
    let mut flat_knots: Array1<'static, f64> =
        super::network_surface_skin::array(a_flat_length as usize)?;
    bsplclib::knot_sequence(&knots, &mults, &mut flat_knots, false)?;
    Ok(Some(ProductBasis {
        degree,
        knots,
        mults,
        flat_knots,
    }))
}

/// `multiplyByDenominators`: the numerator and denominator of `surface` multiplied by
/// the denominators of the two others on the product bases (BSplSLib::FunctionMultiply);
/// None when it reports a failure.
fn multiply_by_denominators(
    [surface, first, second]: [&SurfaceData; 3],
    (u, v): (&ProductBasis, &ProductBasis),
) -> Result<Option<WeightedGrid>, Failure> {
    let (nu, nv) = (
        u.flat_knots.len() as i32 - u.degree - 1,
        v.flat_knots.len() as i32 - v.degree - 1,
    );
    let evaluator = DenominatorProductEvaluator {
        my_first: first,
        my_second: second,
        capture: RefCell::new(FailureCapture::default()),
    };
    let source = MultiplySource {
        surface: surface.span(0.0, 0.0),
        poles: &surface.poles,
        weights: Some(&surface.weights),
    };
    let knots = InterpolationKnots {
        u_flat_knots: &u.flat_knots,
        v_flat_knots: &v.flat_knots,
        u_new_degree: u.degree,
        v_new_degree: v.degree,
    };
    let (mut numerator, mut denominator) = (grid(nu, nv)?, grid(nu, nv)?);
    let mut a_status: i32 = 0;
    let result = bsplslib::function_multiply(
        &evaluator,
        &source,
        &knots,
        &mut numerator,
        &mut denominator,
        &mut a_status,
    );
    evaluator.capture.borrow_mut().take()?;
    result?;
    Ok((a_status == 0).then_some((numerator, denominator)))
}

/// `makeCorrectedProfileSkin`: the poles `profile + guide - reference` of the aligned
/// surfaces; with a rational surface, the three numerators over the common product
/// denominator (which must agree within Precision::Confusion()). Returns the surface or
/// the failing status.
pub(crate) fn make_corrected_profile_skin(
    [profile, guide, reference]: [&GeomBSplineSurface; 3],
) -> Result<Result<Handle<GeomBSplineSurface>, NetworkSurfaceStatus>, Failure> {
    let failed = NetworkSurfaceStatus::RationalConstructionFailed;
    let s = [profile, guide, reference];
    if s.iter()
        .any(|x| x.nb_u_poles() != profile.nb_u_poles() || x.nb_v_poles() != profile.nb_v_poles())
    {
        return Ok(Err(failed));
    }
    if !s.iter().any(|x| x.is_u_rational() || x.is_v_rational()) {
        return Ok(Ok(polynomial_sum(s)?));
    }
    let data = [
        SurfaceData::of(profile),
        SurfaceData::of(guide),
        SurfaceData::of(reference),
    ];
    let (Some(u), Some(v)) = (
        make_triple_product_basis(&data[0].u_knots, &data[0].u_mults, data[0].u_degree)?,
        make_triple_product_basis(&data[0].v_knots, &data[0].v_mults, data[0].v_degree)?,
    ) else {
        return Ok(Err(NetworkSurfaceStatus::RationalDegreeOverflow));
    };
    let (nu, nv) = (
        u.flat_knots.len() as i32 - u.degree - 1,
        v.flat_knots.len() as i32 - v.degree - 1,
    );
    if nu < 2 || nv < 2 {
        return Ok(Err(failed));
    }
    let orders = [
        [&data[0], &data[1], &data[2]],
        [&data[1], &data[0], &data[2]],
        [&data[2], &data[0], &data[1]],
    ];
    let mut products = Vec::with_capacity(3);
    for order in orders {
        let Some(product) = multiply_by_denominators(order, (&u, &v))? else {
            return Ok(Err(failed));
        };
        products.push(product);
    }
    Ok(rational_sum(&products, (&u, &v))?.ok_or(failed))
}

// The polynomial case: `profile + guide - reference` pole by pole, on the profile
// basis.
fn polynomial_sum(
    [profile, guide, reference]: [&GeomBSplineSurface; 3],
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let (p, g, r) = (profile.poles(), guide.poles(), reference.poles());
    let mut result: Array2<'static, Point3> = p.clone();
    for ((pole, gp), rp) in result.iter_mut().zip(g.iter()).zip(r.iter()) {
        *pole = point_of(pole.xyz() + gp.xyz() - rp.xyz());
    }
    let surface = GeomBSplineSurface::new(
        &result,
        (&profile.u_knots(), &profile.u_multiplicities()),
        (&profile.v_knots(), &profile.v_multiplicities()),
        (profile.u_degree(), profile.v_degree()),
        (false, false),
    )?;
    Ok(Handle::new(surface))
}

// The rational case: the numerators summed over the common denominator; None when a
// denominator is within Precision::Confusion() or the three disagree.
fn rational_sum(
    products: &[WeightedGrid],
    (u, v): (&ProductBasis, &ProductBasis),
) -> Result<Option<Handle<GeomBSplineSurface>>, Failure> {
    let [(pn, pd), (gn, gd), (rn, rd)] = products else {
        return Ok(None);
    };
    let (mut poles, mut weights) = (pn.clone(), pd.clone());
    let cells = poles.iter_mut().zip(weights.iter_mut());
    for (i, (pole, weight)) in cells.enumerate() {
        let (a_weight, g_w, r_w) = (pd.as_slice()[i], gd.as_slice()[i], rd.as_slice()[i]);
        if a_weight <= precision::confusion()
            || (g_w - a_weight).abs() > precision::confusion()
            || (r_w - a_weight).abs() > precision::confusion()
        {
            return Ok(None);
        }
        let a_numerator = pn.as_slice()[i].xyz() + gn.as_slice()[i].xyz() - rn.as_slice()[i].xyz();
        *weight = a_weight;
        *pole = point_of(a_numerator / a_weight);
    }
    let surface = GeomBSplineSurface::with_weights(
        &poles,
        &weights,
        (&u.knots, &u.mults),
        (&v.knots, &v.mults),
        (u.degree, v.degree),
        (false, false),
    )?;
    Ok(Some(Handle::new(surface)))
}
