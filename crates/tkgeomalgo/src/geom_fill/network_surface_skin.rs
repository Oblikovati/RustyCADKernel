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
// Rust translation of GeomFill_NetworkSurface.cxx (makeInterpolationBasis,
// interpolatePoles, poleWeight, SkinningBasis, makeProfileSkin, makeGuideSkin and the
// reference surface of makeNetworkSurface) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The profile skin, the guide skin and the reference surface of a network.
use super::gordon_params::null_curve;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBSplineSurface};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::bsplslib::{self, InterpolationGrid};
use rustycad_tkmath::gp::{self, Point3, Xyz};

const MEMBER: &str = "GeomFill_NetworkSurface";

// A 1-based native array of `n` default values.
pub(crate) fn array<T: Default>(n: usize) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(1, n as i32).map_err(|e| e.into_failure(MEMBER))
}

// A native grid `1..=rows x 1..=cols`.
pub(crate) fn grid<T: Default + Clone>(
    rows: i32,
    cols: i32,
) -> Result<Array2<'static, T>, Failure> {
    Array2::with_bounds(1, rows, 1, cols).map_err(|e| e.into_failure(MEMBER))
}

// `At(i)`: the value at the 0-based offset `i` from the lower bound.
pub(crate) fn at<T: Copy>(values: &Array1<'_, T>, i: usize) -> T {
    values[values.lower() + i as i32]
}

/// A grid of poles with its weights.
pub(crate) type WeightedGrid = (Array2<'static, Point3>, Array2<'static, f64>);

/// `SkinningBasis`: the interpolation basis of one family (`makeInterpolationBasis`):
/// degree min(3, n - 1), end knots of multiplicity degree + 1 and the averages of
/// `degree` consecutive interior parameters as simple knots.
pub(crate) struct SkinningBasis {
    pub(crate) degree: i32,
    pub(crate) knots: Array1<'static, f64>,
    pub(crate) mults: Array1<'static, i32>,
    pub(crate) flat_knots: Array1<'static, f64>,
}

impl SkinningBasis {
    /// `SkinningBasis::Init(Parameters)`.
    pub(crate) fn init(the_parameters: &Array1<'_, f64>) -> Result<Self, Failure> {
        let a_nb_params: usize = the_parameters.len();
        let the_degree: i32 = 3.min(a_nb_params as i32 - 1);
        let a_nb_internal_knots: usize = a_nb_params - the_degree as usize - 1;
        let mut knots: Array1<'static, f64> = array(a_nb_internal_knots + 2)?;
        let mut mults: Array1<'static, i32> = array(a_nb_internal_knots + 2)?;
        knots[1] = at(the_parameters, 0);
        mults[1] = the_degree + 1;
        for a_knot_idx in 0..a_nb_internal_knots {
            let mut a_sum: f64 = 0.0;
            for a_param_offset in 1..=the_degree as usize {
                a_sum += at(the_parameters, a_knot_idx + a_param_offset);
            }
            knots[a_knot_idx as i32 + 2] = a_sum / f64::from(the_degree);
            mults[a_knot_idx as i32 + 2] = 1;
        }
        let last: i32 = a_nb_internal_knots as i32 + 2;
        knots[last] = at(the_parameters, a_nb_params - 1);
        mults[last] = the_degree + 1;
        let mut flat_knots: Array1<'static, f64> = array(the_degree as usize + a_nb_params + 1)?;
        let mut a_flat_idx: i32 = 1;
        for k in 1..=last {
            for _ in 0..mults[k] {
                flat_knots[a_flat_idx] = knots[k];
                a_flat_idx += 1;
            }
        }
        Ok(Self {
            degree: the_degree,
            knots,
            mults,
            flat_knots,
        })
    }
}

/// `poleWeight`: the weight of a pole of a rational curve, 1 otherwise.
pub(crate) fn pole_weight(the_curve: &GeomBSplineCurve, the_pole_idx: i32) -> Result<f64, Failure> {
    if the_curve.is_rational() {
        return the_curve.weight(the_pole_idx);
    }
    Ok(1.0)
}

/// `interpolatePoles`: BSplCLib::Interpolate of weighted poles in place; false on an
/// inversion problem.
fn interpolate_poles(
    basis: &SkinningBasis,
    the_parameters: &Array1<'_, f64>,
    the_contact_orders: &Array1<'_, i32>,
    (the_poles, the_weights): (&mut Array1<'_, Point3>, &mut Array1<'_, f64>),
) -> Result<bool, Failure> {
    let mut an_inversion_problem: i32 = 0;
    bsplclib::interpolate_rational(
        basis.degree,
        &basis.flat_knots,
        the_parameters,
        the_contact_orders,
        the_poles,
        the_weights,
        &mut an_inversion_problem,
    )?;
    Ok(an_inversion_problem == 0)
}

/// One direction of skinning: the family, its parameters and the basis across it.
pub(crate) struct SkinFamily<'s> {
    pub(crate) curves: &'s [Handle<GeomBSplineCurve>],
    pub(crate) parameters: &'s Array1<'s, f64>,
    pub(crate) basis: &'s SkinningBasis,
}

// The poles interpolated across the family, pole row `k` of the base curve by row:
// `(poles, weights)` indexed (pole of the base curve, curve), or None on an inversion
// problem.
fn skin_poles(family: &SkinFamily<'_>) -> Result<Option<WeightedGrid>, Failure> {
    let a_base = family.curves[0].get().ok_or_else(null_curve)?;
    let (nb_poles, nb_curves) = (a_base.nb_poles(), family.curves.len());
    let mut a_poles: Array2<'static, Point3> = grid(nb_poles, nb_curves as i32)?;
    let mut a_weights: Array2<'static, f64> = grid(nb_poles, nb_curves as i32)?;
    let mut a_contact_orders: Array1<'static, i32> = array(family.parameters.len())?;
    a_contact_orders.init(0);
    for a_pole_idx in 1..=nb_poles {
        let (mut a_column, mut a_weights_column): (Array1<'static, Point3>, Array1<'static, f64>) =
            (array(nb_curves)?, array(nb_curves)?);
        for (i, curve) in family.curves.iter().enumerate() {
            let curve = curve.get().ok_or_else(null_curve)?;
            a_column[i as i32 + 1] = curve.pole(a_pole_idx)?;
            a_weights_column[i as i32 + 1] = pole_weight(curve, a_pole_idx)?;
        }
        let columns = (&mut a_column, &mut a_weights_column);
        if !interpolate_poles(family.basis, family.parameters, &a_contact_orders, columns)? {
            return Ok(None);
        }
        for i in 1..=nb_curves as i32 {
            a_poles[(a_pole_idx, i)] = a_column[i];
            a_weights[(a_pole_idx, i)] = a_weights_column[i];
        }
    }
    Ok(Some((a_poles, a_weights)))
}

/// `makeProfileSkin`: the surface through the profiles (U along them) whose V direction
/// interpolates their poles on the profile parameters; None on an inversion problem.
pub(crate) fn make_profile_skin(
    family: &SkinFamily<'_>,
) -> Result<Option<Handle<GeomBSplineSurface>>, Failure> {
    let Some((a_poles, a_weights)) = skin_poles(family)? else {
        return Ok(None);
    };
    let a_base = family.curves[0].get().ok_or_else(null_curve)?;
    let surface = GeomBSplineSurface::with_weights(
        &a_poles,
        &a_weights,
        (&a_base.knots(), &a_base.multiplicities()),
        (&family.basis.knots, &family.basis.mults),
        (a_base.degree(), family.basis.degree),
        (false, false),
    )?;
    Ok(Some(Handle::new(surface)))
}

/// `makeGuideSkin`: the surface through the guides (V along them) whose U direction
/// interpolates their poles on the guide parameters; None on an inversion problem.
pub(crate) fn make_guide_skin(
    family: &SkinFamily<'_>,
) -> Result<Option<Handle<GeomBSplineSurface>>, Failure> {
    let Some((a_poles_t, a_weights_t)) = skin_poles(family)? else {
        return Ok(None);
    };
    let (nb_poles, nb_curves) = (a_poles_t.nb_rows() as i32, a_poles_t.nb_columns() as i32);
    let mut a_poles: Array2<'static, Point3> = grid(nb_curves, nb_poles)?;
    let mut a_weights: Array2<'static, f64> = grid(nb_curves, nb_poles)?;
    for k in 1..=nb_poles {
        for i in 1..=nb_curves {
            a_poles[(i, k)] = a_poles_t[(k, i)];
            a_weights[(i, k)] = a_weights_t[(k, i)];
        }
    }
    let a_base = family.curves[0].get().ok_or_else(null_curve)?;
    let surface = GeomBSplineSurface::with_weights(
        &a_poles,
        &a_weights,
        (&family.basis.knots, &family.basis.mults),
        (&a_base.knots(), &a_base.multiplicities()),
        (family.basis.degree, a_base.degree()),
        (false, false),
    )?;
    Ok(Some(Handle::new(surface)))
}

/// The reference surface: BSplSLib::Interpolate of the contact grid (homogeneously
/// for a rational one, the weights reset to 1 when exactly one family is rational) on
/// both bases; None on an inversion problem or an interpolated weight within
/// gp::Resolution().
pub(crate) fn make_reference_surface(
    (points, weights): (&Array2<'_, Point3>, &Array2<'_, f64>),
    (u_basis, v_basis): (&SkinningBasis, &SkinningBasis),
    (u_parameters, v_parameters): (&Array1<'_, f64>, &Array1<'_, f64>),
    is_reference_polynomial: bool,
) -> Result<Option<Handle<GeomBSplineSurface>>, Failure> {
    let mut a_reference_poles: Array2<'static, Point3> = owned2(points);
    let mut a_reference_weights: Array2<'static, f64> = owned2(weights);
    if is_reference_polynomial {
        a_reference_weights.init(1.0);
    } else {
        for (pole, weight) in a_reference_poles.iter_mut().zip(a_reference_weights.iter()) {
            *pole = point_of(pole.xyz() * *weight);
        }
    }
    let grid_data = InterpolationGrid {
        u_degree: u_basis.degree,
        v_degree: v_basis.degree,
        u_flat_knots: &u_basis.flat_knots,
        v_flat_knots: &v_basis.flat_knots,
        u_parameters,
        v_parameters,
    };
    let mut an_inversion_problem: i32 = 0;
    bsplslib::interpolate_rational(
        &grid_data,
        &mut a_reference_poles,
        &mut a_reference_weights,
        &mut an_inversion_problem,
    )?;
    if an_inversion_problem != 0 {
        return Ok(None);
    }
    if !is_reference_polynomial {
        for (pole, weight) in a_reference_poles.iter_mut().zip(a_reference_weights.iter()) {
            if *weight <= gp::resolution() {
                return Ok(None);
            }
            *pole = point_of(pole.xyz() / *weight);
        }
    }
    let surface = GeomBSplineSurface::with_weights(
        &a_reference_poles,
        &a_reference_weights,
        (&u_basis.knots, &u_basis.mults),
        (&v_basis.knots, &v_basis.mults),
        (u_basis.degree, v_basis.degree),
        (false, false),
    )?;
    Ok(Some(Handle::new(surface)))
}

// A `'static` copy of a native array with its bounds.
pub(crate) fn owned1<T: Default + Clone>(source: &Array1<'_, T>) -> Array1<'static, T> {
    let mut copy: Array1<'static, T> = Array1::default();
    copy.assign(source);
    copy
}

// A `'static` copy of a native grid with its bounds.
pub(crate) fn owned2<T: Default + Clone>(source: &Array2<'_, T>) -> Array2<'static, T> {
    let mut copy: Array2<'static, T> = Array2::default();
    copy.assign(source);
    copy
}

// `gp_Pnt(theXYZ)`.
pub(crate) fn point_of(xyz: Xyz) -> Point3 {
    Point3::new(xyz.x(), xyz.y(), xyz.z())
}
