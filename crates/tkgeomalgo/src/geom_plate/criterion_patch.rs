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
// Rust translation of the patch reading shared by GeomPlate_PlateG0Criterion::Value and
// GeomPlate_PlateG1Criterion::Value (ModelingAlgorithms/TKGeomAlgo/GeomPlate) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The polynomial of an AdvApp2Var patch as the two plate criteria read it.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkgeombase::adv_app2var::{AdvApp2VarContext, AdvApp2VarPatch};
use rustycad_tkmath::gp::Xy;
use rustycad_tkmath::plib;

/// The coefficients of a patch, packed `NbCoeffInU x NbCoeffInV x 3`, and its domain.
pub(super) struct PatchPolynomial {
    coeffs: Vec<f64>,
    nb_coeff: [i32; 2],
    u_int: [f64; 2],
    v_int: [f64; 2],
}

/// `adrCoeff[pos]`: the canonical coefficient `pos` (0-based) of the patch; reading
/// past the array is undefined natively and Standard_OutOfRange here.
fn coefficient(array: &Array1<'_, f64>, pos: i32) -> Result<f64, Failure> {
    let index: i32 = array.lower() + pos;
    array.value(index).copied().map_err(|_| {
        errors::out_of_range(&format!(
            "GeomPlate criterion: coefficient {pos} outside the {} of the patch",
            array.length()
        ))
    })
}

impl PatchPolynomial {
    /// Reads the patch coefficients as `Value` does: row `k1` of the U coefficients
    /// starts at `3 * VLimit * (k1 - 1)` (the "JAG 99.04.29" fix). Standard_NullObject
    /// for a patch without coefficients (a null dereference natively).
    pub(super) fn read(p: &AdvApp2VarPatch, c: &AdvApp2VarContext) -> Result<Self, Failure> {
        let array = p.coefficients(1, c)?.ok_or_else(|| {
            errors::null_object("GeomPlate criterion: the patch has no coefficients; expected an approximated patch")
        })?;
        let nb_coeff: [i32; 2] = [p.nb_coeff_in_u(), p.nb_coeff_in_v()];
        let mut coeffs: Vec<f64> =
            Vec::with_capacity((3 * nb_coeff[0] * nb_coeff[1]).max(0) as usize);
        for k1 in 1..=nb_coeff[0] {
            let mut pos: i32 = 3 * c.v_limit() * (k1 - 1);
            for _k2 in 1..=nb_coeff[1] {
                for offset in 0..3 {
                    coeffs.push(coefficient(&array, pos + offset)?);
                }
                pos += 3;
            }
        }
        Ok(Self {
            coeffs,
            nb_coeff,
            u_int: [p.u0(), p.u1()],
            v_int: [p.v0(), p.v1()],
        })
    }

    /// Whether `p2d` is strictly inside the patch domain.
    pub(super) fn contains(&self, p2d: Xy) -> bool {
        let (u, v) = (self.u_int, self.v_int);
        u[0] < p2d.x() && p2d.x() < u[1] && v[0] < p2d.y() && p2d.y() < v[1]
    }

    /// `p2d` mapped onto `(-1, 1)` ("u,v recadres sur (-1,1)").
    pub(super) fn reduced(&self, p2d: Xy) -> (f64, f64) {
        let (u, v) = (self.u_int, self.v_int);
        let up: f64 = (2.0 * p2d.x() - u[0] - u[1]) / (u[1] - u[0]);
        let vp: f64 = (2.0 * p2d.y() - v[0] - v[1]) / (v[1] - v[0]);
        (up, vp)
    }

    /// The derivative `(du, dv)` of the polynomial at the reduced parameters
    /// (`PLib::EvalPoly2Var` of dimension 3).
    pub(super) fn eval(&self, (up, vp): (f64, f64), du: i32, dv: i32) -> Result<[f64; 3], Failure> {
        let mut digit: [f64; 3] = [0.0; 3];
        let (nu, nv) = (self.nb_coeff[0] - 1, self.nb_coeff[1] - 1);
        plib::eval_poly_2var(up, vp, du, dv, nu, nv, 3, &self.coeffs, &mut digit)?;
        Ok(digit)
    }
}
