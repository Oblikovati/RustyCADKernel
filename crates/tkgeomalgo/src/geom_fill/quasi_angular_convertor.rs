// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_QuasiAngularConvertor.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_QuasiAngularConvertor`: a circular arc as the seven rational poles of a
//! degree 6 curve close to the angular parameterisation, and their derivatives.
use super::arc_section::{ArcPolesD1, ArcPolesD2, ArcSectionD0, ArcSectionD1, ArcSectionD2};
use super::convertor_basis::{arc_frame, basis_matrix};
use super::frame_math::point_from_xyz;
use super::quasi_angular_terms::{
    QuasiTerms, first_order_b, first_order_terms, first_order_terms_d2, frame_d1, frame_d2,
    second_order_b, second_order_terms, value_b,
};
use super::sweep_failures::matrix_with_bounds;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::put;
use rustycad_tkmath::gp::{Matrix3, Point3, Vector3, Xyz};
use rustycad_tkmath::math::{Matrix, Vector};
use std::sync::OnceLock;

/// The number of poles (`Ordre`).
const ORDRE: i32 = 7;

/// `GeomFill_QuasiAngularConvertor`: converts an arc into rational poles once
/// `Init` has loaded the basis. `Section` keeps its work vectors in the object (the
/// native members `Px`, `Py`, `W`, `Vx`, `Vy`, `Vw`), so it takes `&mut self`.
///
/// ```ignore
/// let mut convertor = GeomFillQuasiAngularConvertor::new()?;
/// convertor.init()?;
/// convertor.section(&arc, &mut poles, &mut weights)?;
/// ```
#[derive(Clone)]
pub struct GeomFillQuasiAngularConvertor {
    myinit: bool,
    b: Matrix<'static>,
    px: Vector<'static>,
    py: Vector<'static>,
    w: Vector<'static>,
    vx: Vector<'static>,
    vy: Vector<'static>,
    vw: Vector<'static>,
}

/// The monomial basis, computed once per process as the native function-local
/// static.
fn the_basis() -> Result<Matrix<'static>, Failure> {
    static THE_BASIS: OnceLock<Matrix<'static>> = OnceLock::new();
    if let Some(matrix) = THE_BASIS.get() {
        return Ok(matrix.clone());
    }
    let basis = basis_matrix(ORDRE)?;
    Ok(THE_BASIS.get_or_init(|| basis).clone())
}

/// `V1` rotated by half the arc and the frame `M(V1, Dir ^ V1)`.
fn rotated_frame(arc: &ArcSectionD0) -> Result<(Vector3, Matrix3), Failure> {
    let mut rot = Matrix3::default();
    rot.set_rotation(*arc.dir.xyz(), arc.angle / 2.0)?;
    let mut aux = *Vector3::between(arc.center, arc.first_pnt).xyz();
    aux *= rot;
    let v1 = Vector3::from(aux);
    Ok((v1, arc_frame(v1, arc.dir ^ v1)))
}

fn vector7() -> Result<Vector<'static>, Failure> {
    Vector::with_bounds(1, ORDRE)
}

impl GeomFillQuasiAngularConvertor {
    /// `GeomFill_QuasiAngularConvertor()`: not initialized; the matrix and vectors
    /// start at zero (natively they are left uninitialized).
    pub fn new() -> Result<Self, Failure> {
        Ok(Self {
            myinit: false,
            b: matrix_with_bounds(1, ORDRE, 1, ORDRE)?,
            px: vector7()?,
            py: vector7()?,
            w: vector7()?,
            vx: vector7()?,
            vy: vector7()?,
            vw: vector7()?,
        })
    }

    /// `Initialized()`; `convertor.initialized()`.
    pub fn initialized(&self) -> bool {
        self.myinit
    }

    /// `Init()`: loads the basis and the constant terms `Vx(1) = 1`, `Vy(2) = 2`,
    /// `Vw(1) = 1`; `convertor.init()?`.
    pub fn init(&mut self) -> Result<(), Failure> {
        if self.myinit {
            return Ok(());
        }
        self.b = the_basis()?;
        self.vx.init(0.0);
        self.vx[1] = 1.0;
        self.vy.init(0.0);
        self.vy[2] = 2.0;
        self.vw.init(0.0);
        self.vw[1] = 1.0;
        self.myinit = true;
        Ok(())
    }

    /// Stores the odd and even value terms and multiplies them by the basis.
    fn store_value_terms(&mut self, terms: &QuasiTerms) -> Result<(), Failure> {
        for (index, value) in [(3, terms.x[0]), (5, terms.x[1]), (7, terms.x[2])] {
            self.vx[index] = value;
        }
        for (index, value) in [(2, terms.y[0]), (4, terms.y[1]), (6, terms.y[2])] {
            self.vy[index] = value;
        }
        for (index, value) in [(3, terms.w[0]), (5, terms.w[1]), (7, terms.w[2])] {
            self.vw[index] = value;
        }
        self.px.set_matrix_vector(&self.b, &self.vx)?;
        self.py.set_matrix_vector(&self.b, &self.vy)?;
        self.w.set_matrix_vector(&self.b, &self.vw)
    }

    /// `Section(FirstPnt, Center, Dir, Angle, Poles, Weights)`: the seven rational
    /// poles `1..=7` of the arc; `convertor.section(&arc, &mut poles, &mut weights)?`.
    pub fn section(
        &mut self,
        arc: &ArcSectionD0,
        poles: &mut Array1<'_, Point3>,
        weights: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let (_, m) = rotated_frame(arc)?;
        let beta = arc.angle / 4.0;
        self.store_value_terms(&QuasiTerms::value(beta, value_b(beta)))?;
        for ii in 1..=ORDRE {
            let wi = self.w[ii];
            let mut pnt = Xyz::new(self.px[ii] / wi, self.py[ii] / wi, 0.0);
            pnt *= m;
            pnt += arc.center.xyz();
            put(poles, ii, point_from_xyz(pnt))?;
            put(weights, ii, wi)?;
        }
        Ok(())
    }

    /// `Section(...)` with the first derivatives: the poles, weights and their
    /// derivatives; `convertor.section_d1(&arc, out)?`.
    pub fn section_d1(
        &mut self,
        arc: &ArcSectionD1,
        out: ArcPolesD1<'_, '_>,
    ) -> Result<(), Failure> {
        let (m, m_prim) = frame_d1(arc)?;
        let (beta, betaprim) = (arc.arc.angle / 4.0, arc.d_angle / 4.0);
        let (b, bpr) = first_order_b(beta, betaprim);
        let value = QuasiTerms::value(beta, b);
        self.store_value_terms(&value)?;
        let dp = first_order_terms(&value, betaprim, bpr).products(&self.b)?;
        for ii in 1..=ORDRE {
            let wi = self.w[ii];
            let mut p = Xyz::new(self.px[ii] / wi, self.py[ii] / wi, 0.0);
            let mut d = Xyz::new(dp[0][ii] / wi, dp[1][ii] / wi, 0.0);
            d -= (dp[2][ii] / wi) * p;
            put(out.poles, ii, point_from_xyz(m * p + arc.arc.center.xyz()))?;
            p *= m_prim;
            d *= m;
            let mut aux = Xyz::default();
            aux.set_linear_form_offset(1.0, p, 1.0, d, *arc.d_center.xyz());
            put(out.d_poles, ii, Vector3::from(aux))?;
            put(out.weights, ii, wi)?;
            put(out.d_weights, ii, dp[2][ii])?;
        }
        Ok(())
    }

    /// `P`, `DP`, `D2P` of pole `ii` divided by its weight, in the native order.
    fn rational_terms(&self, ii: i32, dp: &[Vector<'static>; 3], d2p: &[Vector<'static>; 3]) -> [Xyz; 3] {
        let (wi, dwi) = (self.w[ii], dp[2][ii]);
        let p = Xyz::new(self.px[ii] / wi, self.py[ii] / wi, 0.0);
        let mut d = Xyz::new(dp[0][ii] / wi, dp[1][ii] / wi, 0.0);
        let mut d2 = Xyz::new(d2p[0][ii] / wi, d2p[1][ii] / wi, 0.0);
        d2 -= 2.0 * (dwi / wi) * d;
        d2 += (2.0 * (dwi / wi).powi(2) - d2p[2][ii] / wi) * p;
        d -= (dp[2][ii] / wi) * p;
        [p, d, d2]
    }

    /// `Section(...)` with the second derivatives;
    /// `convertor.section_d2(&arc, out)?`.
    pub fn section_d2(
        &mut self,
        arc: &ArcSectionD2,
        out: ArcPolesD2<'_, '_>,
    ) -> Result<(), Failure> {
        let [m, m_prim, m_secn] = frame_d2(arc)?;
        let d1 = &arc.d1;
        let primes = [d1.d_angle / 4.0, arc.d2_angle / 4.0];
        let beta = d1.arc.angle / 4.0;
        let (b, bpr, bsc) = second_order_b(beta, primes[0], primes[1]);
        let value = QuasiTerms::value(beta, b);
        self.store_value_terms(&value)?;
        let dp = first_order_terms_d2(&value, primes[0], bpr).products(&self.b)?;
        let d2p = second_order_terms(&value, primes, [bpr, bsc]).products(&self.b)?;
        let ArcPolesD2 { d1: out1, d2_poles, d2_weights } = out;
        for ii in 1..=ORDRE {
            let [mut p, mut d, mut d2] = self.rational_terms(ii, &dp, &d2p);
            put(out1.poles, ii, point_from_xyz(m * p + d1.arc.center.xyz()))?;
            let mut auxyz = Xyz::default();
            auxyz.set_linear_form_offset(1.0, m_prim * p, 1.0, m * d, *d1.d_center.xyz());
            put(out1.d_poles, ii, Vector3::from(auxyz))?;
            p *= m_secn;
            d *= m_prim;
            d2 *= m;
            auxyz.set_linear_form_three_offset([1.0, 2.0, 1.0], [p, d, d2], *arc.d2_center.xyz());
            put(d2_poles, ii, Vector3::from(auxyz))?;
            put(out1.weights, ii, self.w[ii])?;
            put(out1.d_weights, ii, dp[2][ii])?;
            put(d2_weights, ii, d2p[2][ii])?;
        }
        Ok(())
    }
}
