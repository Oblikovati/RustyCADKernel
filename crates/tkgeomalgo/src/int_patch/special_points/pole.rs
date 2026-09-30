// Copyright (c) 2016 OPEN CASCADE SAS
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
// Rust translation of the ProcessSphere, ProcessCone and GetTangentToIntLineForCone
// members of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_SpecialPoints.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The U parameter of a quadric in its pole (the apex of a cone, a pole of a sphere),
//! from the derivatives of the other surface there.
use super::IntPatchSpecialPoints;
use crate::int_surf::IntSurfPntOn2S;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, real::epsilon};
use rustycad_tkmath::elclib::in_period;
use rustycad_tkmath::gp::{Cone3, Vector2, Vector3, Xyz};
use std::f64::consts::PI;

/// `GetTangent(theConeSemiAngle, theParameter, theResult)`: the tangent of the cone
/// generatrix of the half-angle parameter `w = tan(Un/2)`.
fn get_tangent(cone_semi_angle: f64, parameter: f64) -> Xyz {
    let w2: f64 = parameter * parameter;
    let cos_un: f64 = (1.0 - w2) / (1.0 + w2);
    let sin_un: f64 = 2.0 * parameter / (1.0 + w2);
    let tan_a: f64 = cone_semi_angle.tan();
    Xyz::new(tan_a * cos_un, tan_a * sin_un, 1.0)
}

/// The iso parameter of the quadric point of `pt_iso` (on S2 when reversed).
fn iso_u(pt_iso: &IntSurfPntOn2S, is_reversed: bool) -> f64 {
    if is_reversed {
        pt_iso.parameters_on_s2().0
    } else {
        pt_iso.parameters_on_s1().0
    }
}

/// The U parameter in `[0, 2*PI]` of the unit direction `(cos, sin)`.
fn angle_of(vec_cs: Vector2) -> f64 {
    // Angle in range [0, PI/2]
    let mut uq: f64 = if vec_cs.x().abs() < vec_cs.y().abs() {
        vec_cs.x().abs().acos()
    } else {
        vec_cs.y().abs().asin()
    };
    // Convert angles to the range [0, 2*PI]
    if vec_cs.y() < 0.0 {
        if vec_cs.x() > 0.0 {
            uq = -uq;
        } else {
            uq += PI;
        }
    } else if vec_cs.x() < 0.0 {
        uq = PI - uq;
    }
    uq
}

impl IntPatchSpecialPoints {
    /// `ProcessSphere(thePtIso, theDUofPSurf, theDVofPSurf, theIsReversed, theVquad,
    /// theUquad, theIsIsoChoosen)`: the U parameter of the sphere in the pole of
    /// parameter `v_quad` from the derivatives of the other surface (in the sphere
    /// frame), and whether the iso parameter of `pt_iso` was taken because the U
    /// parameter is undefined there (a tangent plane in the pole, bug #26576). The
    /// normalization raises Standard_ConstructionError on a null direction.
    /// `let (u, iso) = IntPatchSpecialPoints::process_sphere(&p, du, dv, false, v)?;`
    pub fn process_sphere(
        pt_iso: &IntSurfPntOn2S,
        du_of_p_surf: Vector3,
        dv_of_p_surf: Vector3,
        is_reversed: bool,
        v_quad: f64,
    ) -> Result<(f64, bool), Failure> {
        let (du, dv) = (du_of_p_surf, dv_of_p_surf);
        if du.z().abs() < precision::p_confusion() && dv.z().abs() < precision::p_confusion() {
            return Ok((iso_u(pt_iso, is_reversed), true));
        }
        // Vector with {cos(Uq), sin(Uq)} coordinates, normalized.
        let mut v1: Vector2 = if du.z().abs() > dv.z().abs() {
            let dus_dvs: f64 = dv.z() / du.z();
            Vector2::new(du.x() * dus_dvs - dv.x(), du.y() * dus_dvs - dv.y())
        } else {
            let dvs_dus: f64 = du.z() / dv.z();
            Vector2::new(dv.x() * dvs_dus - du.x(), dv.y() * dvs_dus - du.y())
        };
        v1.normalize()?;
        let u_quad: f64 = if v1.x().abs() > v1.y().abs() {
            v1.y().asin().copysign(v_quad)
        } else {
            v1.x().acos().copysign(v_quad)
        };
        Ok((u_quad, false))
    }

    /// `ProcessCone(thePtIso, theDUofPSurf, theDVofPSurf, theCone, theIsReversed,
    /// theUquad, theIsIsoChoosen)`: the U parameter of the cone in its apex, the
    /// direction of the generatrix cut by the plane tangent to the other surface that is
    /// nearest to the iso parameter of `pt_iso`; the iso parameter itself (flag true)
    /// when the tangent plane is undefined or cuts the cone in the apex only. The native
    /// member keeps `theUquad` when no generatrix is chosen; `u_quad` is that input.
    /// `let (u, iso) = IntPatchSpecialPoints::process_cone(&p, du, dv, &cone, false, u)?;`
    pub fn process_cone(
        pt_iso: &IntSurfPntOn2S,
        du_of_p_surf: Vector3,
        dv_of_p_surf: Vector3,
        cone: &Cone3,
        is_reversed: bool,
        u_quad: f64,
    ) -> Result<(f64, bool), Failure> {
        // A plane tangent to 2nd (intersected) surface: its normal.
        let tg_plane_z: Xyz = *du_of_p_surf.crossed(dv_of_p_surf).xyz();
        let sq_mod_tg: f64 = tg_plane_z.square_modulus();
        let mut is_iso = sq_mod_tg < precision::square_confusion();
        let mut tg_iline: [Xyz; 2] = [Xyz::default(); 2];
        let nb_tangent: i32 = if is_iso {
            0
        } else {
            Self::get_tangent_to_int_line_for_cone(
                cone.semi_angle(),
                tg_plane_z / sq_mod_tg.sqrt(),
                &mut tg_iline,
            )
        };
        let mut u_quad: f64 = u_quad;
        if nb_tangent == 0 {
            is_iso = true;
        } else {
            let tangents = &tg_iline[..nb_tangent as usize];
            is_iso = Self::nearest_generatrix(pt_iso, tangents, is_reversed, &mut u_quad);
        }
        if is_iso {
            return Ok((iso_u(pt_iso, is_reversed), true));
        }
        Ok((u_quad, false))
    }

    /// The choice loop of `ProcessCone`: writes the parameter of the generatrix nearest
    /// to the iso parameter; true (the iso is chosen) on a vertical tangent.
    fn nearest_generatrix(
        pt_iso: &IntSurfPntOn2S,
        tangents: &[Xyz],
        is_reversed: bool,
        u_quad: &mut f64,
    ) -> bool {
        let period: f64 = PI + PI;
        let u_iso: f64 = in_period(iso_u(pt_iso, is_reversed), 0.0, period);
        let mut min_delta: f64 = f64::MAX;
        for tangent in tangents {
            // Vector {cos(a), sin(a)}
            let vec_cs = Vector2::new(tangent.x(), tangent.y());
            let sq_mod: f64 = vec_cs.square_magnitude();
            if sq_mod < precision::square_confusion() {
                return true;
            }
            let uq: f64 = in_period(angle_of(vec_cs / sq_mod.sqrt()), 0.0, period);
            // Select the parameter the nearest to the iso one.
            let mut delta: f64 = (uq - u_iso).abs();
            if delta > PI {
                delta = period - delta;
            }
            if delta < min_delta {
                min_delta = delta;
                *u_quad = uq;
            }
        }
        false
    }

    /// `GetTangentToIntLineForCone(theConeSemiAngle, thePlnNormal, theResult)`: the
    /// directions (in the canonical cone frame) of the generatrices cut by the plane of
    /// unit normal `pln_normal` through the apex; returns their number (0, 1 or 2) and
    /// writes them into `result`.
    /// `let n = IntPatchSpecialPoints::get_tangent_to_int_line_for_cone(0.3, normal, &mut out);`
    pub fn get_tangent_to_int_line_for_cone(
        cone_semi_angle: f64,
        pln_normal: Xyz,
        result: &mut [Xyz; 2],
    ) -> i32 {
        let null_tol: f64 = epsilon(1.0);
        let tan_a: f64 = cone_semi_angle.tan();
        let a: f64 = pln_normal.z() / tan_a - pln_normal.x();
        let b: f64 = pln_normal.y();
        let c: f64 = pln_normal.z() / tan_a + pln_normal.x();
        if a.abs() < null_tol {
            if b.abs() > null_tol {
                // The plane goes along the cone generatrix.
                result[0] = get_tangent(cone_semi_angle, -c / (b + b));
                return 1;
            }
            // The cone and the plane have only one common point: the apex.
            return 0;
        }
        let mut discr: f64 = pln_normal.z() / cone_semi_angle.sin();
        discr = 1.0 - discr * discr;
        if discr.abs() < null_tol {
            // The plane goes along the cone generatrix (separated from the branch
            // above to eliminate numerical instability).
            result[0] = get_tangent(cone_semi_angle, -b / a);
            return 1;
        }
        if discr > 0.0 {
            let rd: f64 = discr.sqrt();
            result[0] = get_tangent(cone_semi_angle, (-b + rd) / a);
            result[1] = get_tangent(cone_semi_angle, (-b - rd) / a);
            return 2;
        }
        // We will never come here.
        0
    }
}
