// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntImp/IntImp_ComputeTangence.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntImp_ComputeTangence` and `ChoixRef`: the tangent of the intersection of two
//! patches and the isoparametric lines best suited to follow it.
use super::IntImpConstIsoparametric;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Vector3;

/// The native `staticChoixRef` table.
const CHOIX_REF: [IntImpConstIsoparametric; 4] = [
    IntImpConstIsoparametric::UIsoparametricOnCaro1,
    IntImpConstIsoparametric::VIsoparametricOnCaro1,
    IntImpConstIsoparametric::UIsoparametricOnCaro2,
    IntImpConstIsoparametric::VIsoparametricOnCaro2,
];

/// `ChoixRef(theIndex)`: the isoparametric of index 0 to 3 (U then V of the first
/// patch, U then V of the second); another index raises `Standard_OutOfRange`.
/// `choix_ref(1)?` is `VIsoparametricOnCaro1`.
pub fn choix_ref(index: i32) -> Result<IntImpConstIsoparametric, Failure> {
    if !(0..=3).contains(&index) {
        return Err(Failure::with_message(
            "Standard_OutOfRange",
            &format!("ChoixRef() in IntImp_ComputeTangence.cxx: index {index}; expected 0..=3"),
        ));
    }
    Ok(CHOIX_REF[index as usize])
}

/// `gp_Vec::Normalize` of a vector known to be longer than the resolution.
fn normalized(v: Vector3) -> Vector3 {
    let d: f64 = v.magnitude();
    Vector3::new(v.x() / d, v.y() / d, v.z() / d)
}

/// `IntImp_ComputeTangence(DPuv, EpsUV, Tgduv, TabIso)`: whether the patches with
/// first derivatives `dpuv` (dU and dV of patch 1, then of patch 2) are tangent.
///
/// Otherwise `tgduv` receives the components of the intersection tangent on the four
/// derivatives and `tab_iso` the isoparametrics sorted from the best (smallest
/// cosine) to the worst. A null derivative or normal answers true and writes
/// nothing; `compute_tangence(&dpuv, &eps, &mut tgduv, &mut tab_iso)`.
pub fn compute_tangence(
    dpuv: &[Vector3; 4],
    eps_uv: &[f64; 4],
    tgduv: &mut [f64; 4],
    tab_iso: &mut [IntImpConstIsoparametric; 4],
) -> bool {
    let tol2: f64 = 1.0e-32;
    let mut norm_duv: [f64; 4] = dpuv.map(|d| d.square_magnitude());
    if norm_duv.iter().any(|&n| n <= tol2) {
        return true;
    }
    let n1: Vector3 = dpuv[0].crossed(dpuv[1]);
    if n1.square_magnitude() < tol2 {
        return true;
    }
    let n1: Vector3 = normalized(n1);
    let n2: Vector3 = dpuv[2].crossed(dpuv[3]);
    if n2.square_magnitude() < tol2 {
        return true;
    }
    let n2: Vector3 = normalized(n2);
    norm_duv = norm_duv.map(f64::sqrt);
    *tgduv = [
        -dpuv[1].dot(n2),
        dpuv[0].dot(n2),
        dpuv[3].dot(n1),
        -dpuv[2].dot(n1),
    ];
    let tangent: bool = tgduv[0].abs() <= eps_uv[0] * norm_duv[1]
        && tgduv[1].abs() <= eps_uv[1] * norm_duv[0]
        && tgduv[2].abs() <= eps_uv[2] * norm_duv[3]
        && tgduv[3].abs() <= eps_uv[3] * norm_duv[2];
    if tangent || n1.dot(n2).abs() > 0.999999999 {
        return true;
    }
    let cosines: [f64; 4] = [
        tgduv[1].abs() / norm_duv[0],
        tgduv[0].abs() / norm_duv[1],
        tgduv[3].abs() / norm_duv[2],
        tgduv[2].abs() / norm_duv[3],
    ];
    sort_isoparametrics(cosines, tab_iso);
    false
}

/// The native bubble sort of the cosines, carrying the isoparametrics along.
fn sort_isoparametrics(mut cosines: [f64; 4], tab_iso: &mut [IntImpConstIsoparametric; 4]) {
    *tab_iso = CHOIX_REF;
    let mut sorted: bool = false;
    while !sorted {
        sorted = true;
        for i in 1..4 {
            if cosines[i - 1] > cosines[i] {
                sorted = false;
                cosines.swap(i - 1, i);
                tab_iso.swap(i - 1, i);
            }
        }
    }
}
