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
// Rust translation of GeomFill_GuideTrihedronAC::D2 of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_GuideTrihedronAC.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The second derivatives of the arc-length guided frame.
use super::trihedron_law::TrihedronD2;
use rustycad_tkmath::gp::{Point3, Vector3};

/// `a1 V1 + a2 V2 + V3`.
fn form3(a1: f64, v1: Vector3, a2: f64, v2: Vector3, v3: Vector3) -> Vector3 {
    let mut out = Vector3::default();
    out.set_linear_form_offset(a1, v1, a2, v2, v3);
    out
}

/// `a V1 + V2`.
fn scaled(a: f64, v1: Vector3, v2: Vector3) -> Vector3 {
    let mut out = Vector3::default();
    out.set_scaled_sum(a, v1, v2);
    out
}

/// `V1 + V2`.
fn sum(v1: Vector3, v2: Vector3) -> Vector3 {
    let mut out = Vector3::default();
    out.set_sum(v1, v2);
    out
}

/// The frame and derivatives from the path point and three derivatives, the guide
/// point and two derivatives; `scales` is `[Orig2 - Orig1, Lguide / L]`.
pub(crate) fn ac_d2(
    frame: &mut TrihedronD2,
    (p, to, d_to, d2_to): (Point3, Vector3, Vector3, Vector3),
    (pg, tg, dtg): (Point3, Vector3, Vector3),
    [orig, ratio]: [f64; 2],
) {
    let (n_to, n2_to) = (to.magnitude(), to.square_magnitude());
    let (n_tg, n2_tp) = (tg.magnitude(), tg.square_magnitude());
    let dtg_dt = orig * (n_to / n_tg) * ratio;
    let mut n = Vector3::between(p, pg);
    let norm = n.magnitude();
    let mut dn = Vector3::default();
    dn.set_linear_form(dtg_dt, tg, -1.0, to);
    let d2tp_dt2 = orig
        * ratio
        * (d_to.dot(to) / (n_to * n_tg) - (n2_to * tg) * dtg * ratio / (n2_tp * n2_tp));
    let mut d2n = Vector3::default();
    d2n.set_linear_form_three([dtg_dt * dtg_dt, d2tp_dt2, -1.0], [dtg, tg, d_to]);
    if norm > 1.0e-9 {
        n /= norm;
        dn /= norm;
        d2n /= norm;
    }
    frame.normal = n;
    let mut tn = to.crossed(frame.normal);
    let norma = tn.magnitude();
    if norma > 1.0e-9 {
        tn /= norma;
    }
    frame.binormal = tn;
    frame.tangent = frame.normal.crossed(frame.binormal);
    let ndn = n.dot(dn);
    frame.d_normal = scaled(-ndn, n, dn);
    let mut dtn = sum(d_to.crossed(frame.normal), to.crossed(frame.d_normal));
    dtn /= norma;
    let tndtn = tn.dot(dtn);
    frame.d_binormal = scaled(-tndtn, tn, dtn);
    frame.d_tangent = sum(
        frame.normal.crossed(frame.d_binormal),
        frame.d_normal.crossed(frame.binormal),
    );
    let tn2 = tn.square_magnitude();
    frame.d2_normal = form3(
        -2.0 * ndn,
        dn,
        3.0 * ndn * ndn - (dn.square_magnitude() + n.dot(d2n)),
        n,
        d2n,
    );
    let mut d2tn = form3(
        1.0,
        d2_to.crossed(frame.normal),
        2.0,
        d_to.crossed(frame.d_normal),
        to.crossed(frame.d2_normal),
    );
    d2tn /= norma;
    frame.d2_binormal = form3(
        -2.0 * tndtn,
        dtn,
        3.0 * tndtn * tndtn - (tn2 + tn.dot(d2tn)),
        tn,
        d2tn,
    );
    frame.d2_tangent = form3(
        1.0,
        frame.d2_normal.crossed(frame.binormal),
        2.0,
        frame.d_normal.crossed(frame.d_binormal),
        frame.normal.crossed(frame.d2_binormal),
    );
}
