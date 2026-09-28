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
// Rust translation of ModelingData/TKG3d/Geom/Geom_OffsetSurfaceUtils.pxx (EvaluateD2, EvaluateD3, EvaluateDN) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Offset derivatives of order 2, 3 and N: the normal and its derivatives come
//! from CSLib at every point (order 0 when the normal is defined, 3 otherwise).
use super::derivatives::{DerivativeOrders, entry};
use super::evaluate_low::{defined_normal, derivatives_with, singular_normal};
use super::{
    BasisDomain, OsculatingInfo, OsculatingQuery, THE_D1_MAGNITUDE_TOL, is_infinite_coord,
    offset_point, put, query_osculating, shift_point, zero_table,
};
use crate::geom::{GeomSurface, GeomSurfaceD2, GeomSurfaceD3};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::cslib::{self, NormalStatus, SingularNormal};
use rustycad_tkmath::gp::{Point3, Vector3};

// The first-derivative normal status, the extra order and the osculating data
// shared by EvaluateD2, EvaluateD3 and EvaluateDN.
fn osculating_setup(
    uv: (f64, f64),
    d1: (Vector3, Vector3),
    osc: Option<&dyn OsculatingQuery>,
) -> Result<(i32, OsculatingInfo, Handle<dyn GeomSurface>), Failure> {
    let first = cslib::normal_with_magnitude(d1.0, d1.1, THE_D1_MAGNITUDE_TOL)?;
    let defined: bool = first.status == NormalStatus::Defined;
    let max_order: i32 = if defined { 0 } else { 3 };
    if defined {
        return Ok((max_order, OsculatingInfo::default(), Handle::default()));
    }
    let (info, surface) = query_osculating(osc, uv.0, uv.1)?;
    Ok((max_order, info, surface))
}

// The surface table filled from D3 values: (1,0), (0,1), (1,1), (2,0), (0,2),
// (3,0), (2,1), (1,2), (0,3), in the native order.
fn third_order_table(rows: i32, d3: &GeomSurfaceD3) -> Result<Array2<'static, Vector3>, Failure> {
    let mut table = zero_table(rows, rows)?;
    let values = [
        (1, 0, d3.d1u),
        (0, 1, d3.d1v),
        (1, 1, d3.d2uv),
        (2, 0, d3.d2u),
        (0, 2, d3.d2v),
        (3, 0, d3.d3u),
        (2, 1, d3.d3uuv),
        (1, 2, d3.d3uvv),
        (0, 3, d3.d3v),
    ];
    for (i, j, value) in values {
        put(&mut table, i, j, value)?;
    }
    Ok(table)
}

// One pass of EvaluateD2/EvaluateD3: the offset normal data, or None to shift.
struct NormalData {
    point: Point3,
    der_nuv: Array2<'static, Vector3>,
    der_surf: Array2<'static, Vector3>,
    found: SingularNormal,
    scale: f64,
}

fn normal_data(
    uv: (f64, f64),
    basis: &dyn GeomSurface,
    offset: f64,
    osc: Option<&dyn OsculatingQuery>,
    orders: (i32, &GeomSurfaceD3),
    domain: BasisDomain,
) -> Result<Option<NormalData>, Failure> {
    let (extra, d3) = orders;
    let (max_order, info, surface) = osculating_setup(uv, (d3.d1u, d3.d1v), osc)?;
    let mut der_nuv = zero_table(max_order + extra, max_order + extra)?;
    let mut der_surf = third_order_table(max_order + extra + 1, d3)?;
    let orders = DerivativeOrders {
        max_order,
        min_order: 3,
        nu: extra,
        nv: extra,
    };
    if !derivatives_with(
        orders,
        uv,
        basis,
        (&info, &surface),
        (&mut der_nuv, &mut der_surf),
    )? {
        return Ok(None);
    }
    let found: SingularNormal = singular_normal(max_order, &der_nuv, uv, domain)?;
    let Some(normal) = defined_normal(&found) else {
        return Ok(None);
    };
    let scale: f64 = offset * info.sign();
    let mut point: Point3 = d3.point;
    offset_point(&mut point, scale, Vector3::from(normal));
    Ok(Some(NormalData {
        point,
        der_nuv,
        der_surf,
        found,
        scale,
    }))
}

impl NormalData {
    // CSLib::DNNormal(nu, nv, DerNUV, OrderU, OrderV) * scale.
    fn term(&self, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        Ok(cslib::dn_normal(
            nu,
            nv,
            &self.der_nuv,
            self.found.order_u,
            self.found.order_v,
        )? * self.scale)
    }
    // DerSurf(i, j) + DNNormal(i, j) * scale, for the first derivatives.
    fn first(&self, i: i32, j: i32) -> Result<Vector3, Failure> {
        Ok(entry(&self.der_surf, i, j)? + self.term(i, j)?)
    }
}

/// Offset point and derivatives up to order 2 from the basis D3 at `(u, v)`
/// (`EvaluateD2`); `evaluate_d2((u, v), &*basis, 2.0, None)?`.
pub fn evaluate_d2(
    uv: (f64, f64),
    basis: &dyn GeomSurface,
    offset: f64,
    osc: Option<&dyn OsculatingQuery>,
) -> Result<Option<GeomSurfaceD2>, Failure> {
    let d3: GeomSurfaceD3 = basis.eval_d3(uv.0, uv.1)?;
    evaluate_d2_from(uv, basis, offset, osc, d3)
}

/// `EvaluateD2` with the basis derivatives up to order 3 already evaluated;
/// `evaluate_d2_from((u, v), &*basis, 2.0, None, d3)?`.
pub fn evaluate_d2_from(
    start: (f64, f64),
    basis: &dyn GeomSurface,
    offset: f64,
    osc: Option<&dyn OsculatingQuery>,
    first: GeomSurfaceD3,
) -> Result<Option<GeomSurfaceD2>, Failure> {
    let domain = BasisDomain::of(basis);
    let mut uv: (f64, f64) = start;
    let mut current: GeomSurfaceD3 = first;
    let mut is_first_iteration: bool = true;
    loop {
        if !is_first_iteration {
            current = basis.eval_d3(uv.0, uv.1)?;
        }
        is_first_iteration = false;
        if is_infinite_coord(current.d1u) || is_infinite_coord(current.d1v) {
            return Ok(None);
        }
        if let Some(data) = normal_data(uv, basis, offset, osc, (2, &current), domain)? {
            return second_order(&data, uv, basis).map(Some);
        }
        if !shift_point(start, &mut uv, domain, current.d1u, current.d1v) {
            return Ok(None);
        }
    }
}

// The D1 and D2 of the offset at a point whose normal is defined.
fn second_order(
    data: &NormalData,
    uv: (f64, f64),
    basis: &dyn GeomSurface,
) -> Result<GeomSurfaceD2, Failure> {
    let (u, v) = uv;
    let (d1u, d1v) = (data.first(1, 0)?, data.first(0, 1)?);
    let dn20: Vector3 = basis.eval_dn(u, v, 2, 0)?;
    let dn02: Vector3 = basis.eval_dn(u, v, 0, 2)?;
    let dn11: Vector3 = basis.eval_dn(u, v, 1, 1)?;
    Ok(GeomSurfaceD2 {
        point: data.point,
        d1u,
        d1v,
        d2u: dn20 + data.term(2, 0)?,
        d2v: dn02 + data.term(0, 2)?,
        d2uv: dn11 + data.term(1, 1)?,
    })
}

/// Offset point and derivatives up to order 3 at `(u, v)` (`EvaluateD3`); unlike
/// the lower orders it evaluates the basis D3 at every pass, the first included;
/// `evaluate_d3((u, v), &*basis, 2.0, None)?`.
pub fn evaluate_d3(
    start: (f64, f64),
    basis: &dyn GeomSurface,
    offset: f64,
    osc: Option<&dyn OsculatingQuery>,
) -> Result<Option<GeomSurfaceD3>, Failure> {
    let domain = BasisDomain::of(basis);
    let mut uv: (f64, f64) = start;
    loop {
        let current: GeomSurfaceD3 = basis.eval_d3(uv.0, uv.1)?;
        if is_infinite_coord(current.d1u) || is_infinite_coord(current.d1v) {
            return Ok(None);
        }
        if let Some(data) = normal_data(uv, basis, offset, osc, (3, &current), domain)? {
            return third_order(&data, uv, basis).map(Some);
        }
        if !shift_point(start, &mut uv, domain, current.d1u, current.d1v) {
            return Ok(None);
        }
    }
}

// The derivatives up to order 3 of the offset at a point whose normal is defined;
// all basis DN are evaluated before the normal terms are added, as natively.
fn third_order(
    data: &NormalData,
    uv: (f64, f64),
    basis: &dyn GeomSurface,
) -> Result<GeomSurfaceD3, Failure> {
    let (u, v) = uv;
    let (d1u, d1v) = (data.first(1, 0)?, data.first(0, 1)?);
    let orders: [(i32, i32); 7] = [(2, 0), (0, 2), (1, 1), (3, 0), (0, 3), (2, 1), (1, 2)];
    let mut basis_dn: [Vector3; 7] = [Vector3::default(); 7];
    for (slot, (nu, nv)) in basis_dn.iter_mut().zip(orders) {
        *slot = basis.eval_dn(u, v, nu, nv)?;
    }
    let mut summed: [Vector3; 7] = basis_dn;
    for (slot, (nu, nv)) in summed.iter_mut().zip(orders) {
        *slot += data.term(nu, nv)?;
    }
    let [d2u, d2v, d2uv, d3u, d3v, d3uuv, d3uvv] = summed;
    Ok(GeomSurfaceD3 {
        point: data.point,
        d1u,
        d1v,
        d2u,
        d2v,
        d2uv,
        d3u,
        d3v,
        d3uuv,
        d3uvv,
    })
}

/// Offset derivative of orders `(nu, nv)` at `(u, v)` from the basis D1
/// (`EvaluateDN`); `evaluate_dn((u, v), (1, 1), &*basis, 2.0, None)?`.
pub fn evaluate_dn(
    uv: (f64, f64),
    orders: (i32, i32),
    basis: &dyn GeomSurface,
    offset: f64,
    osc: Option<&dyn OsculatingQuery>,
) -> Result<Option<Vector3>, Failure> {
    let d1 = basis.eval_d1(uv.0, uv.1)?;
    evaluate_dn_from(uv, orders, basis, offset, osc, (d1.d1u, d1.d1v))
}

/// `EvaluateDN` with the basis first derivatives already evaluated;
/// `evaluate_dn_from((u, v), (1, 1), &*basis, 2.0, None, (d1u, d1v))?`.
pub fn evaluate_dn_from(
    start: (f64, f64),
    orders: (i32, i32),
    basis: &dyn GeomSurface,
    offset: f64,
    osc: Option<&dyn OsculatingQuery>,
    first: (Vector3, Vector3),
) -> Result<Option<Vector3>, Failure> {
    let domain = BasisDomain::of(basis);
    let mut uv: (f64, f64) = start;
    let (mut d1u, mut d1v) = first;
    let mut is_first_iteration: bool = true;
    loop {
        if !is_first_iteration {
            let r = basis.eval_d1(uv.0, uv.1)?;
            (d1u, d1v) = (r.d1u, r.d1v);
        }
        is_first_iteration = false;
        if is_infinite_coord(d1u) || is_infinite_coord(d1v) {
            return Ok(None);
        }
        if let Some(result) = attempt_dn(uv, orders, basis, (offset, osc), (d1u, d1v), domain)? {
            return Ok(Some(result));
        }
        if !shift_point(start, &mut uv, domain, d1u, d1v) {
            return Ok(None);
        }
    }
}

// One pass of the EvaluateDN loop body; None to shift the point.
fn attempt_dn(
    uv: (f64, f64),
    orders: (i32, i32),
    basis: &dyn GeomSurface,
    offset_osc: (f64, Option<&dyn OsculatingQuery>),
    d1: (Vector3, Vector3),
    domain: BasisDomain,
) -> Result<Option<Vector3>, Failure> {
    let (offset, osc) = offset_osc;
    let (nu, nv) = orders;
    let (max_order, info, surface) = osculating_setup(uv, d1, osc)?;
    let mut der_nuv = zero_table(max_order + nu, max_order + nv)?;
    let mut der_surf = zero_table(max_order + nu + 1, max_order + nv + 1)?;
    put(&mut der_surf, 1, 0, d1.0)?;
    put(&mut der_surf, 0, 1, d1.1)?;
    let request = DerivativeOrders {
        max_order,
        min_order: 1,
        nu,
        nv,
    };
    if !derivatives_with(
        request,
        uv,
        basis,
        (&info, &surface),
        (&mut der_nuv, &mut der_surf),
    )? {
        return Ok(None);
    }
    let found: SingularNormal = singular_normal(max_order, &der_nuv, uv, domain)?;
    if defined_normal(&found).is_none() {
        return Ok(None);
    }
    let scale: f64 = offset * info.sign();
    let basis_dn: Vector3 = basis.eval_dn(uv.0, uv.1, nu, nv)?;
    let normal_dn: Vector3 = cslib::dn_normal(nu, nv, &der_nuv, found.order_u, found.order_v)?;
    Ok(Some(basis_dn + normal_dn * scale))
}
