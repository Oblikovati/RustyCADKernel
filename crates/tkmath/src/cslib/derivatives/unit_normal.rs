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

//! CSLib::DNNormal, the Leibniz recursion for derivatives of the unit normal.
use super::derivative_at;
use crate::cslib::binomial::plib_bin;
use crate::gp::Vector3;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Failure;

/// The native aDerVecNor / aTabScal / aTabNorm tables, indexed `0..=order` twice.
///
/// Every entry the recursion reads is written earlier, so the zero start values
/// (native doubles are uninitialized) are never observed.
struct NormalTables {
    side: usize,
    vectors: Vec<Vector3>,
    scalars: Vec<f64>,
    norms: Vec<f64>,
}

impl NormalTables {
    fn new(order: i32) -> Self {
        let side: usize = order as usize + 1;
        Self {
            side,
            vectors: vec![Vector3::new(0.0, 0.0, 0.0); side * side],
            scalars: vec![0.0; side * side],
            norms: vec![0.0; side * side],
        }
    }
    fn at(&self, p: i32, q: i32) -> usize {
        p as usize * self.side + q as usize
    }
    fn vector(&self, p: i32, q: i32) -> Vector3 {
        self.vectors[self.at(p, q)]
    }
    fn scalar(&self, p: i32, q: i32) -> f64 {
        self.scalars[self.at(p, q)]
    }
    fn norm(&self, p: i32, q: i32) -> f64 {
        self.norms[self.at(p, q)]
    }
}

/// Input derivatives of `N = D1U ^ D1V` and the reference offsets of DNNormal.
struct RawNormal<'s, 'a> {
    derivatives: &'s Array2<'a, Vector3>,
    iduref: i32,
    idvref: i32,
}

impl RawNormal<'_, '_> {
    fn at(&self, p: i32, q: i32) -> Result<Vector3, Failure> {
        derivative_at(self.derivatives, p + self.iduref, q + self.idvref)
    }
    // PLib::Bin(p + Iduref, i + Iduref), the U-direction coefficient.
    fn bin_u(&self, p: i32, i: i32) -> Result<f64, Failure> {
        plib_bin(p + self.iduref, i + self.iduref)
    }
    // PLib::Bin(q + Idvref, j + Idvref), the V-direction coefficient.
    fn bin_v(&self, q: i32, j: i32) -> Result<f64, Failure> {
        plib_bin(q + self.idvref, j + self.idvref)
    }
}

/// CSLib::DNNormal(Nu, Nv, DerNUV, Iduref, Idvref): derivative of the unit normal.
///
/// `der_nuv(i, j)` holds `d^(i+j)(D1U ^ D1V) / (du^i dv^j)` for
/// `i = iduref..=nu+iduref`, `j = idvref..=nv+idvref`. A null reference
/// derivative fails like gp_Vec::Normalized; negative total orders are rejected
/// as the native final table read would be.
/// `dn_normal(1, 0, &der_nuv, 0, 0)?`.
pub fn dn_normal(
    nu: i32,
    nv: i32,
    der_nuv: &Array2<'_, Vector3>,
    iduref: i32,
    idvref: i32,
) -> Result<Vector3, Failure> {
    let raw: RawNormal<'_, '_> = RawNormal {
        derivatives: der_nuv,
        iduref,
        idvref,
    };
    let order: i32 = nu + nv;
    let reference: Vector3 = raw.at(0, 0)?;
    let unit: Vector3 = reference.normalized()?;
    if nu < 0 || nv < 0 {
        return Err(negative_order(nu, nv));
    }
    let mut tables: NormalTables = NormalTables::new(order);
    let origin: usize = tables.at(0, 0);
    tables.vectors[origin] = unit;
    tables.norms[origin] = reference.dot(unit);
    for total in 1..=order {
        for p in 0..=total {
            let q: i32 = total - p;
            if p > nu || q > nv {
                continue;
            }
            fill_order(&mut tables, &raw, p, q)?;
        }
    }
    Ok(tables.vector(nu, nv))
}

fn negative_order(nu: i32, nv: i32) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("CSLib::DNNormal order ({nu}, {nv}): expected nu >= 0 and nv >= 0"),
        "",
    )
}

// One (p,q) step: n . d(p,q)n, then the derivative of |N|, then d(p,q)n itself.
fn fill_order(
    tables: &mut NormalTables,
    raw: &RawNormal<'_, '_>,
    p: i32,
    q: i32,
) -> Result<(), Failure> {
    let index: usize = tables.at(p, q);
    tables.scalars[index] = unit_dot_derivative(tables, p, q)? / 2.0;
    tables.norms[index] = length_derivative(tables, raw, p, q)?;
    tables.vectors[index] = unit_derivative(tables, raw, p, q)?;
    Ok(())
}

// Native computes n . derivative(p,q) of n with two mirrored summation orders.
fn unit_dot_derivative(tables: &NormalTables, p: i32, q: i32) -> Result<f64, Failure> {
    if p > q {
        return unit_dot_derivative_u_major(tables, p, q);
    }
    unit_dot_derivative_v_major(tables, p, q)
}

fn unit_dot_derivative_u_major(tables: &NormalTables, p: i32, q: i32) -> Result<f64, Failure> {
    let mut scal: f64 = 0.0;
    for j in 1..=q {
        scal -= plib_bin(q, j)? * tables.vector(0, j).dot(tables.vector(p, q - j));
    }
    for j in 0..q {
        scal -= plib_bin(q, j)? * tables.vector(p, j).dot(tables.vector(0, q - j));
    }
    for i in 1..p {
        for j in 0..=q {
            scal -= plib_bin(p, i)?
                * plib_bin(q, j)?
                * tables.vector(i, j).dot(tables.vector(p - i, q - j));
        }
    }
    Ok(scal)
}

fn unit_dot_derivative_v_major(tables: &NormalTables, p: i32, q: i32) -> Result<f64, Failure> {
    let mut scal: f64 = 0.0;
    for i in 1..=p {
        scal -= plib_bin(p, i)? * tables.vector(i, 0).dot(tables.vector(p - i, q));
    }
    for i in 0..p {
        scal -= plib_bin(p, i)? * tables.vector(i, q).dot(tables.vector(p - i, 0));
    }
    for i in 0..=p {
        for j in 1..q {
            scal -= plib_bin(p, i)?
                * plib_bin(q, j)?
                * tables.vector(i, j).dot(tables.vector(p - i, q - j));
        }
    }
    Ok(scal)
}

// Derivative (p,q) of the length of the raw normal.
fn length_derivative(
    tables: &NormalTables,
    raw: &RawNormal<'_, '_>,
    p: i32,
    q: i32,
) -> Result<f64, Failure> {
    let mut dnorm: f64 = raw.at(p, q)?.dot(tables.vector(0, 0));
    for j in 0..q {
        dnorm -= raw.bin_v(q, j)? * tables.norm(p, j) * tables.scalar(0, q - j);
    }
    for i in 0..p {
        for j in 0..=q {
            dnorm -= raw.bin_u(p, i)?
                * raw.bin_v(q, j)?
                * tables.norm(i, j)
                * tables.scalar(p - i, q - j);
        }
    }
    Ok(dnorm)
}

// Derivative (p,q) of the unit normal.
fn unit_derivative(
    tables: &NormalTables,
    raw: &RawNormal<'_, '_>,
    p: i32,
    q: i32,
) -> Result<Vector3, Failure> {
    let mut der_nor: Vector3 = raw.at(p, q)?;
    for j in 1..=q {
        let scale: f64 = raw.bin_u(p, 0)? * raw.bin_v(q, j)? * tables.norm(0, j);
        der_nor = der_nor - tables.vector(p, q - j) * scale;
    }
    for i in 1..=p {
        for j in 0..=q {
            let scale: f64 = raw.bin_u(p, i)? * raw.bin_v(q, j)? * tables.norm(i, j);
            der_nor = der_nor - tables.vector(p - i, q - j) * scale;
        }
    }
    let divisor: f64 = raw.bin_u(p, 0)? * raw.bin_v(q, 0)? * tables.norm(0, 0);
    Ok(der_nor / divisor)
}
