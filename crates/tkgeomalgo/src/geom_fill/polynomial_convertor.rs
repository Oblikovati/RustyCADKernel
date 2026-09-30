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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_PolynomialConvertor.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_PolynomialConvertor`: a circular arc as the eight poles of a
//! polynomial (non-rational) Bezier curve of degree 7, and their derivatives.
use super::arc_section::{ArcSectionD0, ArcSectionD1, ArcSectionD2};
use super::convertor_basis::{arc_frame, basis_matrix, math_vector, pole_xyz};
use super::frame_math::point_from_xyz;
use super::sweep_failures::matrix_with_bounds;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::put;
use rustycad_tkmath::gp::{Matrix3, Point3, Vector3, Xyz};
use rustycad_tkmath::math::{Matrix, Vector};
use rustycad_tkmath::plib;
use std::sync::OnceLock;

/// The polynomial degree plus one (`Ordre`).
const ORDRE: i32 = 8;

/// `GeomFill_PolynomialConvertor`: converts an arc into polynomial poles once
/// `Init` has computed the conversion matrix.
///
/// ```ignore
/// let mut convertor = GeomFillPolynomialConvertor::new()?;
/// convertor.init()?;
/// convertor.section(&arc, &mut poles)?;
/// ```
#[derive(Clone)]
pub struct GeomFillPolynomialConvertor {
    myinit: bool,
    bh: Matrix<'static>,
}

/// The Hermite basis of the arc terms `B * H^T`, computed once per process as
/// the native function-local static.
fn bh_matrix() -> Result<Matrix<'static>, Failure> {
    static THE_BH_MATRIX: OnceLock<Matrix<'static>> = OnceLock::new();
    if let Some(matrix) = THE_BH_MATRIX.get() {
        return Ok(matrix.clone());
    }
    let b = basis_matrix(ORDRE)?;
    let mut h = matrix_with_bounds(1, ORDRE, 1, ORDRE)?;
    plib::hermite_coefficients(-1.0, 1.0, ORDRE / 2 - 1, ORDRE / 2 - 1, &mut h)?;
    h.transpose()?;
    let bh = b.multiplied_matrix(&h)?;
    Ok(THE_BH_MATRIX.get_or_init(|| bh).clone())
}

/// The value terms `Vx`, `Vy` of the arc of half angle `beta`.
fn value_terms(beta: f64, cos_b: f64, sin_b: f64) -> [[f64; 8]; 2] {
    let (beta2, beta3) = (beta * beta, beta * (beta * beta));
    [
        [
            1.0,
            0.0,
            -beta2,
            0.0,
            cos_b,
            -beta * sin_b,
            -beta2 * cos_b,
            beta3 * sin_b,
        ],
        [
            0.0,
            beta,
            0.0,
            -beta3,
            sin_b,
            beta * cos_b,
            -beta2 * sin_b,
            -beta3 * cos_b,
        ],
    ]
}

/// The first derivative terms `DVx`, `DVy` (`bprim = DAngle / 2`).
fn first_terms(beta: f64, bprim: f64, cos_b: f64, sin_b: f64) -> [[f64; 8]; 2] {
    let (b_bprim, b2_bprim) = (bprim * beta, bprim * (beta * beta));
    [
        [
            0.0,
            0.0,
            -2.0 * b_bprim,
            0.0,
            -2.0 * bprim * sin_b,
            -bprim * sin_b - 2.0 * b_bprim * cos_b,
            2.0 * b_bprim * (-cos_b + beta * sin_b),
            b2_bprim * (3.0 * sin_b + 2.0 * beta * cos_b),
        ],
        [
            0.0,
            bprim,
            0.0,
            -3.0 * b2_bprim,
            2.0 * bprim * cos_b,
            bprim * cos_b - 2.0 * b_bprim * sin_b,
            -2.0 * b_bprim * (sin_b + beta * cos_b),
            b2_bprim * (2.0 * beta * sin_b - 3.0 * cos_b),
        ],
    ]
}

/// The second derivative terms `D2Vx`, `D2Vy` (`bsecn = D2Angle / 2`).
fn second_terms(beta: f64, bprim: f64, bsecn: f64, cos_b: f64, sin_b: f64) -> [[f64; 8]; 2] {
    let (beta2, bprim2, b_bsecn) = (beta * beta, bprim * bprim, bsecn * beta);
    let aux7 = 2.0 * (bprim2 + b_bsecn);
    let aux8 = beta * (2.0 * bprim2 + b_bsecn);
    [
        [
            0.0,
            0.0,
            -2.0 * (bprim2 + b_bsecn),
            0.0,
            -2.0 * (bsecn * sin_b + 2.0 * bprim2 * cos_b),
            (4.0 * beta * bprim2 - bsecn) * sin_b - 2.0 * (2.0 * bprim2 + b_bsecn) * cos_b,
            aux7 * (-cos_b + beta * sin_b)
                + 2.0 * beta * bprim2 * (2.0 * beta * cos_b + 3.0 * sin_b),
            aux8 * (3.0 * sin_b + 2.0 * beta * cos_b)
                + 4.0 * beta2 * bprim2 * (2.0 * cos_b - beta * sin_b),
        ],
        [
            0.0,
            bsecn,
            0.0,
            -3.0 * beta * (2.0 * bprim2 + b_bsecn),
            2.0 * (bsecn * cos_b - 2.0 * bprim2 * sin_b),
            (bsecn - 4.0 * beta * bprim2) * cos_b - 2.0 * (b_bsecn + 2.0 * bprim2) * sin_b,
            -aux7 * (sin_b + beta * cos_b)
                - 2.0 * beta * bprim2 * (3.0 * cos_b - 2.0 * beta * sin_b),
            aux8 * (2.0 * beta * sin_b - 3.0 * cos_b)
                + 4.0 * beta2 * bprim2 * (2.0 * sin_b + beta * cos_b),
        ],
    ]
}

/// `MPrim` and `MSecn`: the derivatives of the arc frame `M`.
fn frame_derivatives(arc: &ArcSectionD2, v1: Vector3) -> (Matrix3, Matrix3) {
    let d1 = &arc.d1;
    let v1_prim = d1.d_first_pnt - d1.d_center;
    let v2_prim = (d1.d_dir ^ v1) + (d1.arc.dir ^ v1_prim);
    let v1_secn = arc.d2_first_pnt - arc.d2_center;
    let mut v2_secn = d1.d_dir ^ v1_prim;
    v2_secn *= 2.0;
    v2_secn += (arc.d2_dir ^ v1) + (d1.arc.dir ^ v1_secn);
    (arc_frame(v1_prim, v2_prim), arc_frame(v1_secn, v2_secn))
}

impl GeomFillPolynomialConvertor {
    /// `GeomFill_PolynomialConvertor()`: not initialized; the matrix is zero until
    /// `init` (natively it is left uninitialized).
    pub fn new() -> Result<Self, Failure> {
        Ok(Self {
            myinit: false,
            bh: matrix_with_bounds(1, ORDRE, 1, ORDRE)?,
        })
    }

    /// `Initialized()`: whether `init` ran; `convertor.initialized()`.
    pub fn initialized(&self) -> bool {
        self.myinit
    }

    /// `Init()`: computes the conversion matrix (once per process, then copied);
    /// `convertor.init()?`.
    pub fn init(&mut self) -> Result<(), Failure> {
        if self.myinit {
            return Ok(());
        }
        self.bh = bh_matrix()?;
        self.myinit = true;
        Ok(())
    }

    /// `Section(FirstPnt, Center, Dir, Angle, Poles)`: the poles `1..=8` of the arc;
    /// `convertor.section(&arc, &mut poles)?`.
    pub fn section(
        &self,
        arc: &ArcSectionD0,
        poles: &mut Array1<'_, Point3>,
    ) -> Result<(), Failure> {
        let v1 = Vector3::between(arc.center, arc.first_pnt);
        let m = arc_frame(v1, arc.dir ^ v1);
        let [vx, vy] = value_terms(arc.angle / 2.0, arc.angle.cos(), arc.angle.sin());
        let px = self.bh.multiplied_vector(&math_vector(&vx)?)?;
        let py = self.bh.multiplied_vector(&math_vector(&vy)?)?;
        for ii in 1..=ORDRE {
            let mut pnt = Xyz::new(px[ii], py[ii], 0.0);
            pnt *= m;
            pnt += arc.center.xyz();
            put(poles, ii, point_from_xyz(pnt))?;
        }
        Ok(())
    }

    /// `Section(..., Poles, DPoles)`: the poles and their first derivatives;
    /// `convertor.section_d1(&arc, &mut poles, &mut d_poles)?`.
    pub fn section_d1(
        &self,
        arc: &ArcSectionD1,
        poles: &mut Array1<'_, Point3>,
        d_poles: &mut Array1<'_, Vector3>,
    ) -> Result<(), Failure> {
        let base = &arc.arc;
        let v1 = Vector3::between(base.center, base.first_pnt);
        let m = arc_frame(v1, base.dir ^ v1);
        let v1_prim = arc.d_first_pnt - arc.d_center;
        let m_prim = arc_frame(v1_prim, (arc.d_dir ^ v1) + (base.dir ^ v1_prim));
        let (beta, bprim) = (base.angle / 2.0, arc.d_angle / 2.0);
        let (cos_b, sin_b) = (base.angle.cos(), base.angle.sin());
        let [vx, vy] = value_terms(beta, cos_b, sin_b);
        let [dvx, dvy] = first_terms(beta, bprim, cos_b, sin_b);
        let px = self.bh.multiplied_vector(&math_vector(&vx)?)?;
        let py = self.bh.multiplied_vector(&math_vector(&vy)?)?;
        let dpx = self.bh.multiplied_vector(&math_vector(&dvx)?)?;
        let dpy = self.bh.multiplied_vector(&math_vector(&dvy)?)?;
        for ii in 1..=ORDRE {
            let mut p = Xyz::new(px[ii], py[ii], 0.0);
            put(poles, ii, point_from_xyz(m * p + base.center.xyz()))?;
            p *= m_prim;
            let mut dp = Xyz::new(dpx[ii], dpy[ii], 0.0);
            dp *= m;
            let mut aux = Xyz::default();
            aux.set_linear_form_offset(1.0, p, 1.0, dp, *arc.d_center.xyz());
            put(d_poles, ii, Vector3::from(aux))?;
        }
        Ok(())
    }

    /// `Section(..., Poles, DPoles, D2Poles)`: the poles and two derivatives;
    /// `convertor.section_d2(&arc, &mut poles, &mut d_poles, &mut d2_poles)?`.
    pub fn section_d2(
        &self,
        arc: &ArcSectionD2,
        poles: &mut Array1<'_, Point3>,
        d_poles: &mut Array1<'_, Vector3>,
        d2_poles: &mut Array1<'_, Vector3>,
    ) -> Result<(), Failure> {
        let base = &arc.d1.arc;
        let v1 = Vector3::between(base.center, base.first_pnt);
        let m = arc_frame(v1, base.dir ^ v1);
        let (m_prim, m_secn) = frame_derivatives(arc, v1);
        let (beta, bprim, bsecn) = (base.angle / 2.0, arc.d1.d_angle / 2.0, arc.d2_angle / 2.0);
        let (cos_b, sin_b) = (base.angle.cos(), base.angle.sin());
        let terms = [
            value_terms(beta, cos_b, sin_b),
            first_terms(beta, bprim, cos_b, sin_b),
            second_terms(beta, bprim, bsecn, cos_b, sin_b),
        ];
        let mut products: Vec<Vector<'static>> = Vec::with_capacity(6);
        for term in terms.iter().flatten() {
            products.push(self.bh.multiplied_vector(&math_vector(term)?)?);
        }
        let frames = [m, m_prim, m_secn];
        for ii in 1..=ORDRE {
            let p = pole_xyz(&products, 0, ii);
            let dp = pole_xyz(&products, 2, ii);
            let d2p = pole_xyz(&products, 4, ii);
            put(poles, ii, point_from_xyz(m * p + base.center.xyz()))?;
            let (d1, d2) = second_order_poles(&frames, [p, dp, d2p], arc);
            put(d_poles, ii, d1)?;
            put(d2_poles, ii, d2)?;
        }
        Ok(())
    }
}

/// The first and second derivative of one pole from the three frames and the
/// three planar terms, in the native operation order.
fn second_order_poles(frames: &[Matrix3; 3], terms: [Xyz; 3], arc: &ArcSectionD2) -> (Vector3, Vector3) {
    let [m, m_prim, m_secn] = *frames;
    let [mut p, mut dp, mut d2p] = terms;
    let mut auxyz = Xyz::default();
    auxyz.set_linear_form_offset(1.0, m_prim * p, 1.0, m * dp, *arc.d1.d_center.xyz());
    let d1 = Vector3::from(auxyz);
    p *= m_secn;
    dp *= m_prim;
    d2p *= m;
    auxyz.set_linear_form_three_offset([1.0, 2.0, 1.0], [p, dp, d2p], *arc.d2_center.xyz());
    (d1, Vector3::from(auxyz))
}
