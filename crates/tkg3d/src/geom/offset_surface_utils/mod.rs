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
// Rust translation of ModelingData/TKG3d/Geom/Geom_OffsetSurfaceUtils.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_OffsetSurfaceUtils`: offset point and derivatives from basis derivatives,
//! with the fallbacks at singular points (osculating surfaces, higher-order
//! normals through CSLib, derivative replacement and point shifting).
//!
//! The native templates take the basis surface and an optional osculating-surface
//! query; here the basis is any [`BasisSurfaceEvaluator`] (a Geom surface or a
//! GeomAdaptor_Surface, as natively) and the query an [`OsculatingQuery`]. The `evaluate_*` functions return `Ok(None)` where the
//! native ones return false.
mod derivatives;
mod evaluate_high;
mod evaluate_low;
pub use derivatives::{DerivativeOrders, compute_derivatives};
pub use evaluate_high::{
    evaluate_d2, evaluate_d2_from, evaluate_d3, evaluate_dn, evaluate_dn_from,
};
pub use evaluate_low::{evaluate_d0, evaluate_d0_from, evaluate_d1, evaluate_d1_from};

use crate::geom::errors;
use crate::geom::{BasisSurfaceEvaluator, GeomSurface};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::gp::{Point3, Vector3};

/// Magnitude below which a first derivative (or the normal) counts as null
/// (`THE_D1_MAGNITUDE_TOL`).
pub const THE_D1_MAGNITUDE_TOL: f64 = 1.0e-9;

/// The osculating-surface flags of one evaluation (`OsculatingInfo`);
/// `OsculatingInfo::default().sign() == 1.0`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OsculatingInfo {
    /// Osculating along U.
    pub along_u: bool,
    /// Osculating along V.
    pub along_v: bool,
    /// Whether the normal must be reversed.
    pub is_opposite: bool,
}
impl OsculatingInfo {
    /// -1 when osculating and opposite, 1 otherwise (`Sign`); `info.sign()`.
    pub fn sign(&self) -> f64 {
        if (self.along_u || self.along_v) && self.is_opposite {
            -1.0
        } else {
            1.0
        }
    }
    /// Whether osculating in either direction (`HasOsculating`); `info.has_osculating()`.
    pub fn has_osculating(&self) -> bool {
        self.along_u || self.along_v
    }
}

/// The queries of `Geom_OsculatingSurface` that the offset evaluation needs: whether
/// the point lies on a degenerate boundary, the osculating surface there and whether
/// its normal is opposite. Like the native members, a query that does not apply
/// leaves `is_opposite` and `surface` untouched; `osc.u_osculating_surface(u, v, &mut t, &mut l)?`.
pub trait OsculatingQuery {
    /// `UOsculatingSurface(U, V, T, L)`.
    fn u_osculating_surface(
        &self,
        u: f64,
        v: f64,
        is_opposite: &mut bool,
        surface: &mut Handle<dyn GeomSurface>,
    ) -> Result<bool, Failure>;
    /// `VOsculatingSurface(U, V, T, L)`.
    fn v_osculating_surface(
        &self,
        u: f64,
        v: f64,
        is_opposite: &mut bool,
        surface: &mut Handle<dyn GeomSurface>,
    ) -> Result<bool, Failure>;
}

/// The two osculating queries of the evaluators, U first then V, sharing the
/// opposite flag and the surface as natively; `query_osculating(osc, u, v)?`.
pub fn query_osculating(
    query: Option<&dyn OsculatingQuery>,
    u: f64,
    v: f64,
) -> Result<(OsculatingInfo, Handle<dyn GeomSurface>), Failure> {
    let mut info = OsculatingInfo::default();
    let mut surface: Handle<dyn GeomSurface> = Handle::default();
    if let Some(query) = query {
        info.along_u = query.u_osculating_surface(u, v, &mut info.is_opposite, &mut surface)?;
        info.along_v = query.v_osculating_surface(u, v, &mut info.is_opposite, &mut surface)?;
    }
    Ok((info, surface))
}

/// Whether a coordinate is infinite in the Precision sense (`IsInfiniteCoord`);
/// `is_infinite_coord(Vector3::new(1e101, 0.0, 0.0))`.
pub fn is_infinite_coord(vector: Vector3) -> bool {
    precision::is_infinite(vector.x())
        || precision::is_infinite(vector.y())
        || precision::is_infinite(vector.z())
}

// D1U and D1V divided by their norms when longer than 1, and their cross product.
pub(super) fn bounded_cross_product(d1u: Vector3, d1v: Vector3) -> Vector3 {
    let (mut u, mut v) = (d1u, d1v);
    let (u_norm2, v_norm2) = (u.square_magnitude(), v.square_magnitude());
    if u_norm2 > 1.0 {
        u /= u_norm2.sqrt();
    }
    if v_norm2 > 1.0 {
        v /= v_norm2.sqrt();
    }
    u.crossed(v)
}

/// Whether the normal of the (bounded) tangents is null within `tol` (`IsSingular`);
/// `is_singular(d1u, d1v, THE_D1_MAGNITUDE_TOL)`.
pub fn is_singular(d1u: Vector3, d1v: Vector3, tol: f64) -> bool {
    bounded_cross_product(d1u, d1v).square_magnitude() <= tol * tol
}

/// The unit normal of the (bounded) tangents, `None` when null within `tol`
/// (`ComputeNormal`); `compute_normal(d1u, d1v, THE_D1_MAGNITUDE_TOL)?`.
pub fn compute_normal(d1u: Vector3, d1v: Vector3, tol: f64) -> Result<Option<Vector3>, Failure> {
    let mut normal: Vector3 = bounded_cross_product(d1u, d1v);
    if normal.square_magnitude() <= tol * tol {
        return Ok(None);
    }
    normal.normalize()?;
    Ok(Some(normal))
}

// The components of the U or V derivative of D1U ^ D1V, before projection.
fn raw_normal_derivative(a: Vector3, d1v: Vector3, d1u: Vector3, b: Vector3) -> Vector3 {
    Vector3::new(
        a.y() * d1v.z() + d1u.y() * b.z() - a.z() * d1v.y() - d1u.z() * b.y(),
        -(a.x() * d1v.z() + d1u.x() * b.z() - a.z() * d1v.x() - d1u.z() * b.x()),
        a.x() * d1v.y() + d1u.x() * b.y() - a.y() * d1v.x() - d1u.y() * b.x(),
    )
}

// Remove the normal component and divide by (D1U ^ D1V) . N.
fn project_normal_derivative(mut raw: Vector3, scale: f64, normal: Vector3) -> Vector3 {
    let along: f64 = raw.dot(normal);
    raw -= normal * along;
    raw /= scale;
    raw
}

/// U derivative of the unit normal (`ComputeDNormalU`); `compute_dnormal_u(d1u, d1v, d2u, d2uv, n)`.
pub fn compute_dnormal_u(
    d1u: Vector3,
    d1v: Vector3,
    d2u: Vector3,
    d2uv: Vector3,
    normal: Vector3,
) -> Vector3 {
    let scale: f64 = d1u.crossed(d1v).dot(normal);
    project_normal_derivative(raw_normal_derivative(d2u, d1v, d1u, d2uv), scale, normal)
}

/// V derivative of the unit normal (`ComputeDNormalV`); `compute_dnormal_v(d1u, d1v, d2v, d2uv, n)`.
///
/// The native formula groups the products differently from the U one; the terms
/// are `D2UV x D1V + D1U x D2V` written with their operands swapped where it does.
pub fn compute_dnormal_v(
    d1u: Vector3,
    d1v: Vector3,
    d2v: Vector3,
    d2uv: Vector3,
    normal: Vector3,
) -> Vector3 {
    let scale: f64 = d1u.crossed(d1v).dot(normal);
    let raw = Vector3::new(
        d2uv.y() * d1v.z() + d2v.z() * d1u.y() - d2uv.z() * d1v.y() - d2v.y() * d1u.z(),
        -(d2uv.x() * d1v.z() + d2v.z() * d1u.x() - d2uv.z() * d1v.x() - d2v.x() * d1u.z()),
        d2uv.x() * d1v.y() + d2v.y() * d1u.x() - d2uv.y() * d1v.x() - d2v.x() * d1u.y(),
    );
    project_normal_derivative(raw, scale, normal)
}

// theValue.SetXYZ(theValue.XYZ() + scale * theNormal.XYZ()).
pub(super) fn offset_point(value: &mut Point3, scale: f64, normal: Vector3) {
    value.set_xyz(value.xyz() + *normal.xyz() * scale);
}

/// Offset point along the regular normal; `value` enters as the basis point
/// (`CalculateD0`); `calculate_d0(&mut p, d1u, d1v, 2.0, 1.0)?`.
pub fn calculate_d0(
    value: &mut Point3,
    d1u: Vector3,
    d1v: Vector3,
    offset: f64,
    sign: f64,
) -> Result<bool, Failure> {
    let Some(normal) = compute_normal(d1u, d1v, THE_D1_MAGNITUDE_TOL)? else {
        return Ok(false);
    };
    offset_point(value, offset * sign, normal);
    Ok(true)
}

/// Offset point and first derivatives at a regular point (`CalculateD1`);
/// `calculate_d1(&mut p, [&mut d1u, &mut d1v], [d2u, d2v, d2uv], 2.0, 1.0)?`.
pub fn calculate_d1(
    value: &mut Point3,
    d1: [&mut Vector3; 2],
    d2: [Vector3; 3],
    offset: f64,
    sign: f64,
) -> Result<bool, Failure> {
    let [d1u, d1v] = d1;
    let [d2u, d2v, d2uv] = d2;
    let Some(normal) = compute_normal(*d1u, *d1v, THE_D1_MAGNITUDE_TOL)? else {
        return Ok(false);
    };
    offset_point(value, offset * sign, normal);
    let n1u: Vector3 = compute_dnormal_u(*d1u, *d1v, d2u, d2uv, normal);
    let n1v: Vector3 = compute_dnormal_v(*d1u, *d1v, d2v, d2uv, normal);
    *d1u += n1u * (offset * sign);
    *d1v += n1v * (offset * sign);
    Ok(true)
}

/// The parametric rectangle and periodicity of the basis, read once per evaluation.
#[derive(Clone, Copy, Debug)]
pub struct BasisDomain {
    /// `UMin`, `UMax`, `VMin`, `VMax`.
    pub bounds: (f64, f64, f64, f64),
    /// `IsUPeriodic`.
    pub u_periodic: bool,
    /// `IsVPeriodic`.
    pub v_periodic: bool,
}
impl BasisDomain {
    /// The domain of `basis` (`Bounds`, `IsUPeriodic`, `IsVPeriodic`, which an
    /// adaptor basis can raise from); `BasisDomain::of(&*surface)?`.
    pub fn of<B: BasisSurfaceEvaluator + ?Sized>(basis: &B) -> Result<Self, Failure> {
        Ok(Self {
            bounds: basis.basis_bounds()?,
            u_periodic: basis.basis_u_periodic()?,
            v_periodic: basis.basis_v_periodic()?,
        })
    }
}

/// Move `(u, v)` from the start point towards the middle of the domain by twice
/// the distance already travelled (at least PConfusion); false when the step
/// would pass the middle (`ShiftPoint`); `shift_point((u0, v0), &mut uv, domain, d1u, d1v)`.
pub fn shift_point(
    start: (f64, f64),
    uv: &mut (f64, f64),
    domain: BasisDomain,
    d1u: Vector3,
    d1v: Vector3,
) -> bool {
    let (u_min, u_max, v_min, v_max) = domain.bounds;
    let tol2: f64 = THE_D1_MAGNITUDE_TOL * THE_D1_MAGNITUDE_TOL;
    let is_u_singular: bool = d1u.square_magnitude() < tol2;
    let is_v_singular: bool = d1v.square_magnitude() < tol2;
    let dir_u: f64 = if domain.u_periodic || (is_u_singular && !is_v_singular) {
        0.
    } else {
        0.5 * (u_min + u_max) - start.0
    };
    let dir_v: f64 = if domain.v_periodic || (is_v_singular && !is_u_singular) {
        0.
    } else {
        0.5 * (v_min + v_max) - start.1
    };
    let dist: f64 = (dir_u * dir_u + dir_v * dir_v).sqrt();
    let (du, dv) = (uv.0 - start.0, uv.1 - start.1);
    let mut step: f64 = std_max(2. * (du * du + dv * dv).sqrt(), precision::p_confusion());
    if step >= dist {
        return false;
    }
    step /= dist;
    uv.0 += dir_u * step;
    uv.1 += dir_v * step;
    true
}

/// Replace a null first derivative by the one at a nearby parameter, when exactly
/// one of D1U and D1V is null (`ReplaceDerivative`);
/// `replace_derivative(uv, domain, [&mut du, &mut dv], tol2, &*basis)?`.
pub fn replace_derivative<B: BasisSurfaceEvaluator + ?Sized>(
    uv: (f64, f64),
    bounds: (f64, f64, f64, f64),
    derivatives: [&mut Vector3; 2],
    square_tol: f64,
    basis: &B,
) -> Result<bool, Failure> {
    let [du, dv] = derivatives;
    let replace_du: bool = du.square_magnitude() < square_tol;
    let replace_dv: bool = dv.square_magnitude() < square_tol;
    if replace_du == replace_dv {
        return Ok(false);
    }
    let (u_min, u_max, v_min, v_max) = bounds;
    let step: f64 = if replace_dv {
        let step: f64 = precision::confusion() * du.magnitude();
        if step > u_max - u_min {
            (u_max - u_min) / 100.
        } else {
            step
        }
    } else {
        let step: f64 = precision::confusion() * dv.magnitude();
        if step > v_max - v_min {
            (v_max - v_min) / 100.
        } else {
            step
        }
    };
    for step_sign in [-1.0, 1.0] {
        let (mut u, mut v) = uv;
        if replace_dv {
            u = uv.0 + step_sign * step;
            if u < u_min || u > u_max {
                continue;
            }
        } else {
            v = uv.1 + step_sign * step;
            if v < v_min || v > v_max {
                continue;
            }
        }
        let result = basis.basis_d1(u, v)?;
        let mut replaced: bool = false;
        if replace_du && result.d1u.square_magnitude() > square_tol {
            *du = result.d1u;
            replaced = true;
        }
        if replace_dv && result.d1v.square_magnitude() > square_tol {
            *dv = result.d1v;
            replaced = true;
        }
        if replaced {
            return Ok(true);
        }
    }
    Ok(false)
}

/// A zero-filled derivative table with bounds `0..=rows` x `0..=cols`.
pub(crate) fn zero_table(rows: i32, cols: i32) -> Result<Array2<'static, Vector3>, Failure> {
    Array2::with_bounds(0, rows, 0, cols).map_err(|error| {
        errors::out_of_range(&format!(
            "Geom_OffsetSurfaceUtils: derivative table 0..{rows} x 0..{cols}: {error}"
        ))
    })
}

/// `table(i, j)` with the native bound check; `at(&table, 1, 0)?`.
pub(crate) fn at(table: &Array2<'_, Vector3>, i: i32, j: i32) -> Result<Vector3, Failure> {
    table.value(i, j).copied().map_err(errors::array_bounds)
}

/// `table.SetValue(i, j, value)` with the native bound check; `put(&mut t, 1, 0, v)?`.
pub(crate) fn put(
    table: &mut Array2<'_, Vector3>,
    i: i32,
    j: i32,
    value: Vector3,
) -> Result<(), Failure> {
    table.set_value(i, j, value).map_err(errors::array_bounds)
}
