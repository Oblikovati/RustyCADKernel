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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/Plate/Plate_Plate.cxx (the derivative
// table of Plate_Plate::SolEm) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The closed-form derivatives `D^(IU,IV)` of the fundamental solution `r^(2(m-1)) ln r`
//! of the iterated Laplacian, one function per `(IU, IV)` case of the native switch
//! (with `IU >= IV`). The expressions are the native ones term for term: every integer
//! factor becomes the equal `f64` literal and the products keep their left-to-right
//! order, so each value is bit-identical to the native one (integer sub-products are
//! exact in `f64`). They are generated from the native source and kept as long
//! expressions because they are mathematics copied verbatim, as the native comments say.
#![allow(
    clippy::too_many_lines,
    clippy::neg_multiply,
    clippy::identity_op,
    clippy::let_and_return
)]

/// The quantities shared by every case: the scaled coordinates `u`, `v`, `u2 = u * u`,
/// `r = u2 + v * v`, `l = ln r`, the power `pr = r^(m - 1 - IU - IV)` and the order `m`
/// (with `mm1 = m - 1`), as `f64`.
#[derive(Clone, Copy, Debug)]
pub(super) struct SolEmTerms {
    pub(super) u: f64,
    pub(super) v: f64,
    pub(super) u2: f64,
    pub(super) r: f64,
    pub(super) l: f64,
    pub(super) pr: f64,
    pub(super) m: f64,
    pub(super) mm1: f64,
}

/// `DUV` for `(IU, IV)`, 0 for the combinations the native switch does not list.
pub(super) fn duv(iu: i32, iv: i32, t: &SolEmTerms) -> f64 {
    match (iu, iv) {
        (0, 0) => duv_0_0(t),
        (1, 0) => duv_1_0(t),
        (1, 1) => duv_1_1(t),
        (2, 0) => duv_2_0(t),
        (2, 1) => duv_2_1(t),
        (2, 2) => duv_2_2(t),
        (3, 0) => duv_3_0(t),
        (3, 1) => duv_3_1(t),
        (3, 2) => duv_3_2(t),
        (3, 3) => duv_3_3(t),
        (4, 0) => duv_4_0(t),
        (4, 1) => duv_4_1(t),
        (4, 2) => duv_4_2(t),
        (4, 3) => duv_4_3(t),
        (5, 0) => duv_5_0(t),
        (5, 1) => duv_5_1(t),
        (5, 2) => duv_5_2(t),
        (6, 0) => duv_6_0(t),
        _ => 0.0,
    }
}

#[allow(unused_variables)]
fn duv_0_0(t: &SolEmTerms) -> f64 {
    let SolEmTerms {
        u,
        v,
        u2,
        r,
        l,
        pr,
        m,
        mm1,
    } = *t;
    let duv: f64 = pr * l;
    duv
}

#[allow(unused_variables)]
fn duv_1_0(t: &SolEmTerms) -> f64 {
    let SolEmTerms {
        u,
        v,
        u2,
        r,
        l,
        pr,
        m,
        mm1,
    } = *t;
    let duv: f64 = 2.0 * pr * u * (1.0 + l * mm1);
    duv
}

#[allow(unused_variables)]
fn duv_1_1(t: &SolEmTerms) -> f64 {
    let SolEmTerms {
        u,
        v,
        u2,
        r,
        l,
        pr,
        m,
        mm1,
    } = *t;
    let m2: f64 = m * m;
    let duv: f64 = 4.0 * pr * u * v * ((2.0 * m - 3.0) + (m2 - 3.0 * m + 2.0) * l);
    duv
}

#[allow(unused_variables)]
fn duv_2_0(t: &SolEmTerms) -> f64 {
    let SolEmTerms {
        u,
        v,
        u2,
        r,
        l,
        pr,
        m,
        mm1,
    } = *t;
    let m2: f64 = m * m;
    let duv: f64 = 2.0
        * pr
        * (r - l * r + l * m * r - 6.0 * u2 + 4.0 * l * u2 + 4.0 * m * u2 - 6.0 * l * m * u2
            + 2.0 * l * m2 * u2);
    duv
}

#[allow(unused_variables)]
fn duv_2_1(t: &SolEmTerms) -> f64 {
    let SolEmTerms {
        u,
        v,
        u2,
        r,
        l,
        pr,
        m,
        mm1,
    } = *t;
    let m2: f64 = m * m;
    let m3: f64 = m2 * m;
    let duv: f64 = -3.0 * r + 2.0 * l * r + 2.0 * m * r - 3.0 * l * m * r + l * m2 * r + 22.0 * u2
        - 12.0 * l * u2
        - 24.0 * m * u2
        + 22.0 * l * m * u2
        + 6.0 * m2 * u2
        - 12.0 * l * m2 * u2
        + 2.0 * l * m3 * u2;
    let duv: f64 = duv * 4.0 * pr * v;
    duv
}

#[allow(unused_variables)]
fn duv_2_2(t: &SolEmTerms) -> f64 {
    let SolEmTerms {
        u,
        v,
        u2,
        r,
        l,
        pr,
        m,
        mm1,
    } = *t;
    let m2: f64 = m * m;
    let m3: f64 = m2 * m;
    let m4: f64 = m2 * m2;
    let v2: f64 = v * v;
    let r2: f64 = r * r;
    let duv: f64 =
        -3.0 * r2 + 2.0 * l * r2 + 2.0 * m * r2 - 3.0 * l * m * r2 + l * m2 * r2 + 22.0 * r * u2
            - 12.0 * l * r * u2
            - 24.0 * m * r * u2
            + 22.0 * l * m * r * u2
            + 6.0 * m2 * r * u2
            - 12.0 * l * m2 * r * u2;
    let duv: f64 = duv
        + (2.0 * l * m3 * r * u2 + 22.0 * r * v2 - 12.0 * l * r * v2 - 24.0 * m * r * v2
            + 22.0 * l * m * r * v2
            + 6.0 * m2 * r * v2
            - 12.0 * l * m2 * r * v2
            + 2.0 * l * m3 * r * v2
            - 200.0 * u2 * v2
            + 96.0 * l * u2 * v2);
    let duv: f64 = duv
        + (280.0 * m * u2 * v2 - 200.0 * l * m * u2 * v2 - 120.0 * m2 * u2 * v2
            + 140.0 * l * m2 * u2 * v2
            + 16.0 * m3 * u2 * v2
            - 40.0 * l * m3 * u2 * v2
            + 4.0 * l * m4 * u2 * v2);
    let duv: f64 = 4.0 * pr * duv;
    duv
}

#[allow(unused_variables)]
fn duv_3_0(t: &SolEmTerms) -> f64 {
    let SolEmTerms {
        u,
        v,
        u2,
        r,
        l,
        pr,
        m,
        mm1,
    } = *t;
    let m2: f64 = m * m;
    let m3: f64 = m2 * m;
    let duv: f64 =
        -9.0 * r + 6.0 * l * r + 6.0 * m * r - 9.0 * l * m * r + 3.0 * l * m2 * r + 22.0 * u2
            - 12.0 * l * u2
            - 24.0 * m * u2
            + 22.0 * l * m * u2
            + 6.0 * m2 * u2
            - 12.0 * l * m2 * u2
            + 2.0 * l * m3 * u2;
    let duv: f64 = duv * 4.0 * pr * u;
    duv
}

#[allow(unused_variables)]
fn duv_3_1(t: &SolEmTerms) -> f64 {
    let SolEmTerms {
        u,
        v,
        u2,
        r,
        l,
        pr,
        m,
        mm1,
    } = *t;
    let m2: f64 = m * m;
    let m3: f64 = m2 * m;
    let m4: f64 = m2 * m2;
    let duv: f64 = 33.0 * r - 18.0 * l * r - 36.0 * m * r + 33.0 * l * m * r + 9.0 * m2 * r
        - 18.0 * l * m2 * r
        + 3.0 * l * m3 * r
        - 100.0 * u2
        + 48.0 * l * u2
        + 140.0 * m * u2
        - 100.0 * l * m * u2
        - 60.0 * m2 * u2
        + 70.0 * l * m2 * u2;
    let duv: f64 = duv + (8.0 * m3 * u2 - 20.0 * l * m3 * u2 + 2.0 * l * m4 * u2);
    let duv: f64 = 8.0 * pr * u * v * duv;
    duv
}

#[allow(unused_variables)]
fn duv_3_2(t: &SolEmTerms) -> f64 {
    let SolEmTerms {
        u,
        v,
        u2,
        r,
        l,
        pr,
        m,
        mm1,
    } = *t;
    let m2: f64 = m * m;
    let m3: f64 = m2 * m;
    let m4: f64 = m2 * m2;
    let m5: f64 = m4 * m;
    let ru2: f64 = r * u2;
    let v2: f64 = v * v;
    let rv2: f64 = r * v2;
    let u2v2: f64 = v2 * u2;
    let r2: f64 = r * r;
    let duv: f64 =
        -100.0 * ru2 + 48.0 * l * ru2 + 140.0 * m * ru2 - 100.0 * l * m * ru2 - 60.0 * m2 * ru2
            + 70.0 * l * m2 * ru2
            + 8.0 * m3 * ru2
            - 20.0 * l * m3 * ru2
            + 2.0 * l * m4 * ru2
            - 300.0 * rv2
            + 144.0 * l * rv2
            + 420.0 * m * rv2
            - 300.0 * l * m * rv2
            - 180.0 * m2 * rv2
            + 210.0 * l * m2 * rv2
            + 24.0 * m3 * rv2
            - 60.0 * l * m3 * rv2
            + 6.0 * l * m4 * rv2
            + 33.0 * r2
            - 18.0 * l * r2
            - 36.0 * m * r2
            + 33.0 * l * m * r2
            + 9.0 * m2 * r2
            - 18.0 * l * m2 * r2
            + 3.0 * l * m3 * r2
            + 1096.0 * u2v2
            - 480.0 * l * u2v2
            - 1800.0 * m * u2v2
            + 1096.0 * l * m * u2v2
            + 1020.0 * m2 * u2v2
            - 900.0 * l * m2 * u2v2
            - 240.0 * m3 * u2v2
            + 340.0 * l * m3 * u2v2
            + 20.0 * m4 * u2v2
            - 60.0 * l * m4 * u2v2
            + 4.0 * l * m5 * u2v2;
    let duv: f64 = 8.0 * pr * u * duv;
    duv
}

#[allow(unused_variables)]
fn duv_3_3(t: &SolEmTerms) -> f64 {
    let SolEmTerms {
        u,
        v,
        u2,
        r,
        l,
        pr,
        m,
        mm1,
    } = *t;
    let m2: f64 = m * m;
    let m3: f64 = m2 * m;
    let m4: f64 = m2 * m2;
    let m5: f64 = m3 * m2;
    let m6: f64 = m3 * m3;
    let ru2: f64 = r * u2;
    let v2: f64 = v * v;
    let rv2: f64 = r * v2;
    let u2v2: f64 = v2 * u2;
    let r2: f64 = r * r;
    let duv: f64 = 1644.0 * ru2 - 720.0 * l * ru2 - 2700.0 * m * ru2
        + 1644.0 * l * m * ru2
        + 1530.0 * m2 * ru2
        - 1350.0 * l * m2 * ru2
        - 360.0 * m3 * ru2
        + 510.0 * l * m3 * ru2
        + 30.0 * m4 * ru2
        - 90.0 * l * m4 * ru2
        + 6.0 * l * m5 * ru2
        + 1644.0 * rv2
        - 720.0 * l * rv2
        - 2700.0 * m * rv2
        + 1644.0 * l * m * rv2
        + 1530.0 * m2 * rv2
        - 1350.0 * l * m2 * rv2
        - 360.0 * m3 * rv2
        + 510.0 * l * m3 * rv2
        + 30.0 * m4 * rv2
        - 90.0 * l * m4 * rv2
        + 6.0 * l * m5 * rv2
        - 450.0 * r2
        + 216.0 * l * r2
        + 630.0 * m * r2
        - 450.0 * l * m * r2
        - 270.0 * m2 * r2
        + 315.0 * l * m2 * r2
        + 36.0 * m3 * r2
        - 90.0 * l * m3 * r2
        + 9.0 * l * m4 * r2
        - 7056.0 * u2v2
        + 2880.0 * l * u2v2
        + 12992.0 * m * u2v2
        - 7056.0 * l * m * u2v2
        - 8820.0 * m2 * u2v2
        + 6496.0 * l * m2 * u2v2
        + 2800.0 * m3 * u2v2
        - 2940.0 * l * m3 * u2v2
        - 420.0 * m4 * u2v2
        + 700.0 * l * m4 * u2v2
        + 24.0 * m5 * u2v2
        - 84.0 * l * m5 * u2v2
        + 4.0 * l * m6 * u2v2;
    let duv: f64 = 16.0 * pr * u * v * duv;
    duv
}

#[allow(unused_variables)]
fn duv_4_0(t: &SolEmTerms) -> f64 {
    let SolEmTerms {
        u,
        v,
        u2,
        r,
        l,
        pr,
        m,
        mm1,
    } = *t;
    let m2: f64 = m * m;
    let m3: f64 = m2 * m;
    let m4: f64 = m2 * m2;
    let u4: f64 = u2 * u2;
    let r2: f64 = r * r;
    let duv: f64 = -9.0 * r2 + 6.0 * l * r2 + 6.0 * m * r2 - 9.0 * l * m * r2
        + 3.0 * l * m2 * r2
        + 132.0 * r * u2
        - 72.0 * l * r * u2
        - 144.0 * m * r * u2
        + 132.0 * l * m * r * u2
        + 36.0 * m2 * r * u2
        - 72.0 * l * m2 * r * u2;
    let duv: f64 = duv
        + (12.0 * l * m3 * r * u2 - 200.0 * u4 + 96.0 * l * u4 + 280.0 * m * u4
            - 200.0 * l * m * u4
            - 120.0 * m2 * u4
            + 140.0 * l * m2 * u4
            + 16.0 * m3 * u4
            - 40.0 * l * m3 * u4
            + 4.0 * l * m4 * u4);
    let duv: f64 = 4.0 * pr * duv;
    duv
}

#[allow(unused_variables)]
fn duv_4_1(t: &SolEmTerms) -> f64 {
    let SolEmTerms {
        u,
        v,
        u2,
        r,
        l,
        pr,
        m,
        mm1,
    } = *t;
    let m2: f64 = m * m;
    let m3: f64 = m2 * m;
    let m4: f64 = m2 * m2;
    let m5: f64 = m2 * m3;
    let u4: f64 = u2 * u2;
    let ru2: f64 = r * u2;
    let r2: f64 = r * r;
    let duv: f64 =
        -600.0 * ru2 + 288.0 * l * ru2 + 840.0 * m * ru2 - 600.0 * l * m * ru2 - 360.0 * m2 * ru2
            + 420.0 * l * m2 * ru2
            + 48.0 * m3 * ru2
            - 120.0 * l * m3 * ru2
            + 12.0 * l * m4 * ru2
            + 33.0 * r2
            - 18.0 * l * r2
            - 36.0 * m * r2
            + 33.0 * l * m * r2
            + 9.0 * m2 * r2
            - 18.0 * l * m2 * r2
            + 3.0 * l * m3 * r2
            + 1096.0 * u4
            - 480.0 * l * u4
            - 1800.0 * m * u4
            + 1096.0 * l * m * u4
            + 1020.0 * m2 * u4
            - 900.0 * l * m2 * u4
            - 240.0 * m3 * u4
            + 340.0 * l * m3 * u4
            + 20.0 * m4 * u4
            - 60.0 * l * m4 * u4
            + 4.0 * l * m5 * u4;
    let duv: f64 = 8.0 * pr * v * duv;
    duv
}

#[allow(unused_variables)]
fn duv_4_2(t: &SolEmTerms) -> f64 {
    let SolEmTerms {
        u,
        v,
        u2,
        r,
        l,
        pr,
        m,
        mm1,
    } = *t;
    let m2: f64 = m * m;
    let m3: f64 = m2 * m;
    let m4: f64 = m2 * m2;
    let m5: f64 = m2 * m3;
    let m6: f64 = m3 * m3;
    let u4: f64 = u2 * u2;
    let r2: f64 = r * r;
    let r3: f64 = r2 * r;
    let v2: f64 = v * v;
    let u2v2: f64 = v2 * u2;
    let ru2v2: f64 = r * u2v2;
    let u4v2: f64 = u4 * v2;
    let r2u2: f64 = r2 * u2;
    let ru4: f64 = r * u4;
    let r2v2: f64 = r2 * v2;
    let duv: f64 = 6576.0 * ru2v2 - 2880.0 * l * ru2v2 - 10800.0 * m * ru2v2
        + 6576.0 * l * m * ru2v2
        + 6120.0 * m2 * ru2v2
        - 5400.0 * l * m2 * ru2v2
        - 1440.0 * m3 * ru2v2
        + 2040.0 * l * m3 * ru2v2
        + 120.0 * m4 * ru2v2
        - 360.0 * l * m4 * ru2v2
        + 24.0 * l * m5 * ru2v2
        + 1096.0 * ru4
        - 480.0 * l * ru4
        - 1800.0 * m * ru4
        + 1096.0 * l * m * ru4
        + 1020.0 * m2 * ru4
        - 900.0 * l * m2 * ru4
        - 240.0 * m3 * ru4
        + 340.0 * l * m3 * ru4
        + 20.0 * m4 * ru4
        - 60.0 * l * m4 * ru4
        + 4.0 * l * m5 * ru4
        - 600.0 * r2u2
        + 288.0 * l * r2u2
        + 840.0 * m * r2u2
        - 600.0 * l * m * r2u2
        - 360.0 * m2 * r2u2
        + 420.0 * l * m2 * r2u2
        + 48.0 * m3 * r2u2
        - 120.0 * l * m3 * r2u2
        + 12.0 * l * m4 * r2u2
        - 300.0 * r2v2
        + 144.0 * l * r2v2
        + 420.0 * m * r2v2
        - 300.0 * l * m * r2v2
        - 180.0 * m2 * r2v2
        + 210.0 * l * m2 * r2v2
        + 24.0 * m3 * r2v2
        - 60.0 * l * m3 * r2v2
        + 6.0 * l * m4 * r2v2
        + 33.0 * r3
        - 18.0 * l * r3
        - 36.0 * m * r3
        + 33.0 * l * m * r3
        + 9.0 * m2 * r3
        - 18.0 * l * m2 * r3
        + 3.0 * l * m3 * r3
        - 14112.0 * u4v2
        + 5760.0 * l * u4v2
        + 25984.0 * m * u4v2
        - 14112.0 * l * m * u4v2
        - 17640.0 * m2 * u4v2
        + 12992.0 * l * m2 * u4v2
        + 5600.0 * m3 * u4v2
        - 5880.0 * l * m3 * u4v2
        - 840.0 * m4 * u4v2
        + 1400.0 * l * m4 * u4v2
        + 48.0 * m5 * u4v2
        - 168.0 * l * m5 * u4v2
        + 8.0 * l * m6 * u4v2;
    let duv: f64 = 8.0 * pr * duv;
    duv
}

#[allow(unused_variables)]
fn duv_4_3(t: &SolEmTerms) -> f64 {
    let SolEmTerms {
        u,
        v,
        u2,
        r,
        l,
        pr,
        m,
        mm1,
    } = *t;
    let m2: f64 = m * m;
    let m3: f64 = m2 * m;
    let m4: f64 = m2 * m2;
    let m5: f64 = m2 * m3;
    let m6: f64 = m3 * m3;
    let m7: f64 = m3 * m4;
    let u4: f64 = u2 * u2;
    let r2: f64 = r * r;
    let r3: f64 = r2 * r;
    let v2: f64 = v * v;
    let u2v2: f64 = v2 * u2;
    let ru2v2: f64 = r * u2v2;
    let u4v2: f64 = u4 * v2;
    let r2u2: f64 = r2 * u2;
    let r2v2: f64 = r2 * v2;
    let ru4: f64 = r * u4;
    let duv: f64 = -42336.0 * ru2v2 + 17280.0 * l * ru2v2 + 77952.0 * m * ru2v2
        - 42336.0 * l * m * ru2v2
        - 52920.0 * m2 * ru2v2
        + 38976.0 * l * m2 * ru2v2
        + 16800.0 * m3 * ru2v2
        - 17640.0 * l * m3 * ru2v2
        - 2520.0 * m4 * ru2v2
        + 4200.0 * l * m4 * ru2v2
        + 144.0 * m5 * ru2v2
        - 504.0 * l * m5 * ru2v2
        + 24.0 * l * m6 * ru2v2
        - 21168.0 * ru4
        + 8640.0 * l * ru4
        + 38976.0 * m * ru4
        - 21168.0 * l * m * ru4
        - 26460.0 * m2 * ru4
        + 19488.0 * l * m2 * ru4
        + 8400.0 * m3 * ru4
        - 8820.0 * l * m3 * ru4
        - 1260.0 * m4 * ru4
        + 2100.0 * l * m4 * ru4
        + 72.0 * m5 * ru4
        - 252.0 * l * m5 * ru4
        + 12.0 * l * m6 * ru4
        + 9864.0 * r2u2
        - 4320.0 * l * r2u2
        - 16200.0 * m * r2u2
        + 9864.0 * l * m * r2u2
        + 9180.0 * m2 * r2u2
        - 8100.0 * l * m2 * r2u2
        - 2160.0 * m3 * r2u2
        + 3060.0 * l * m3 * r2u2
        + 180.0 * m4 * r2u2
        - 540.0 * l * m4 * r2u2
        + 36.0 * l * m5 * r2u2
        + 1644.0 * r2v2
        - 720.0 * l * r2v2
        - 2700.0 * m * r2v2
        + 1644.0 * l * m * r2v2
        + 1530.0 * m2 * r2v2
        - 1350.0 * l * m2 * r2v2
        - 360.0 * m3 * r2v2
        + 510.0 * l * m3 * r2v2
        + 30.0 * m4 * r2v2
        - 90.0 * l * m4 * r2v2
        + 6.0 * l * m5 * r2v2
        - 450.0 * r3
        + 216.0 * l * r3
        + 630.0 * m * r3
        - 450.0 * l * m * r3
        - 270.0 * m2 * r3
        + 315.0 * l * m2 * r3
        + 36.0 * m3 * r3
        - 90.0 * l * m3 * r3
        + 9.0 * l * m4 * r3
        + 104544.0 * u4v2
        - 40320.0 * l * u4v2
        - 210112.0 * m * u4v2
        + 104544.0 * l * m * u4v2
        + 162456.0 * m2 * u4v2
        - 105056.0 * l * m2 * u4v2
        - 62720.0 * m3 * u4v2
        + 54152.0 * l * m3 * u4v2
        + 12880.0 * m4 * u4v2
        - 15680.0 * l * m4 * u4v2
        - 1344.0 * m5 * u4v2
        + 2576.0 * l * m5 * u4v2
        + 56.0 * m6 * u4v2
        - 224.0 * l * m6 * u4v2
        + 8.0 * l * m7 * u4v2;
    let duv: f64 = 16.0 * pr * v * duv;
    duv
}

#[allow(unused_variables)]
fn duv_5_0(t: &SolEmTerms) -> f64 {
    let SolEmTerms {
        u,
        v,
        u2,
        r,
        l,
        pr,
        m,
        mm1,
    } = *t;
    let m2: f64 = m * m;
    let m3: f64 = m2 * m;
    let m4: f64 = m2 * m2;
    let m5: f64 = m2 * m3;
    let u4: f64 = u2 * u2;
    let r2: f64 = r * r;
    let ru2: f64 = r * u2;
    let duv: f64 = -1000.0 * ru2 + 480.0 * l * ru2 + 1400.0 * m * ru2
        - 1000.0 * l * m * ru2
        - 600.0 * m2 * ru2
        + 700.0 * l * m2 * ru2
        + 80.0 * m3 * ru2
        - 200.0 * l * m3 * ru2
        + 20.0 * l * m4 * ru2
        + 165.0 * r2
        - 90.0 * l * r2
        - 180.0 * m * r2
        + 165.0 * l * m * r2
        + 45.0 * m2 * r2
        - 90.0 * l * m2 * r2
        + 15.0 * l * m3 * r2
        + 1096.0 * u4
        - 480.0 * l * u4
        - 1800.0 * m * u4
        + 1096.0 * l * m * u4
        + 1020.0 * m2 * u4
        - 900.0 * l * m2 * u4
        - 240.0 * m3 * u4
        + 340.0 * l * m3 * u4
        + 20.0 * m4 * u4
        - 60.0 * l * m4 * u4
        + 4.0 * l * m5 * u4;
    let duv: f64 = 8.0 * pr * u * duv;
    duv
}

#[allow(unused_variables)]
fn duv_5_1(t: &SolEmTerms) -> f64 {
    let SolEmTerms {
        u,
        v,
        u2,
        r,
        l,
        pr,
        m,
        mm1,
    } = *t;
    let m2: f64 = m * m;
    let m3: f64 = m2 * m;
    let m4: f64 = m2 * m2;
    let m5: f64 = m2 * m3;
    let m6: f64 = m3 * m3;
    let u4: f64 = u2 * u2;
    let ru2: f64 = r * u2;
    let r2: f64 = r * r;
    let duv: f64 = 5480.0 * ru2 - 2400.0 * l * ru2 - 9000.0 * m * ru2
        + 5480.0 * l * m * ru2
        + 5100.0 * m2 * ru2
        - 4500.0 * l * m2 * ru2
        - 1200.0 * m3 * ru2
        + 1700.0 * l * m3 * ru2
        + 100.0 * m4 * ru2
        - 300.0 * l * m4 * ru2
        + 20.0 * l * m5 * ru2
        - 750.0 * r2
        + 360.0 * l * r2
        + 1050.0 * m * r2
        - 750.0 * l * m * r2
        - 450.0 * m2 * r2
        + 525.0 * l * m2 * r2
        + 60.0 * m3 * r2
        - 150.0 * l * m3 * r2
        + 15.0 * l * m4 * r2
        - 7056.0 * u4
        + 2880.0 * l * u4
        + 12992.0 * m * u4
        - 7056.0 * l * m * u4
        - 8820.0 * m2 * u4
        + 6496.0 * l * m2 * u4
        + 2800.0 * m3 * u4
        - 2940.0 * l * m3 * u4
        - 420.0 * m4 * u4
        + 700.0 * l * m4 * u4
        + 24.0 * m5 * u4
        - 84.0 * l * m5 * u4
        + 4.0 * l * m6 * u4;
    let duv: f64 = 16.0 * pr * u * v * duv;
    duv
}

#[allow(unused_variables)]
fn duv_5_2(t: &SolEmTerms) -> f64 {
    let SolEmTerms {
        u,
        v,
        u2,
        r,
        l,
        pr,
        m,
        mm1,
    } = *t;
    let m2: f64 = m * m;
    let m3: f64 = m2 * m;
    let m4: f64 = m2 * m2;
    let m5: f64 = m2 * m3;
    let m6: f64 = m3 * m3;
    let m7: f64 = m3 * m4;
    let u4: f64 = u2 * u2;
    let r2: f64 = r * r;
    let r3: f64 = r2 * r;
    let v2: f64 = v * v;
    let u2v2: f64 = v2 * u2;
    let ru2v2: f64 = r * u2v2;
    let u4v2: f64 = u4 * v2;
    let r2u2: f64 = r2 * u2;
    let r2v2: f64 = r2 * v2;
    let ru4: f64 = r * u4;
    let duv: f64 = -70560.0 * ru2v2 + 28800.0 * l * ru2v2 + 129920.0 * m * ru2v2
        - 70560.0 * l * m * ru2v2
        - 88200.0 * m2 * ru2v2
        + 64960.0 * l * m2 * ru2v2
        + 28000.0 * m3 * ru2v2
        - 29400.0 * l * m3 * ru2v2
        - 4200.0 * m4 * ru2v2
        + 7000.0 * l * m4 * ru2v2
        + 240.0 * m5 * ru2v2
        - 840.0 * l * m5 * ru2v2
        + 40.0 * l * m6 * ru2v2
        - 7056.0 * ru4
        + 2880.0 * l * ru4
        + 12992.0 * m * ru4
        - 7056.0 * l * m * ru4
        - 8820.0 * m2 * ru4
        + 6496.0 * l * m2 * ru4
        + 2800.0 * m3 * ru4
        - 2940.0 * l * m3 * ru4
        - 420.0 * m4 * ru4
        + 700.0 * l * m4 * ru4
        + 24.0 * m5 * ru4
        - 84.0 * l * m5 * ru4
        + 4.0 * l * m6 * ru4
        + 5480.0 * r2u2
        - 2400.0 * l * r2u2
        - 9000.0 * m * r2u2
        + 5480.0 * l * m * r2u2
        + 5100.0 * m2 * r2u2
        - 4500.0 * l * m2 * r2u2
        - 1200.0 * m3 * r2u2
        + 1700.0 * l * m3 * r2u2
        + 100.0 * m4 * r2u2
        - 300.0 * l * m4 * r2u2
        + 20.0 * l * m5 * r2u2
        + 8220.0 * r2v2
        - 3600.0 * l * r2v2
        - 13500.0 * m * r2v2
        + 8220.0 * l * m * r2v2
        + 7650.0 * m2 * r2v2
        - 6750.0 * l * m2 * r2v2
        - 1800.0 * m3 * r2v2
        + 2550.0 * l * m3 * r2v2
        + 150.0 * m4 * r2v2
        - 450.0 * l * m4 * r2v2
        + 30.0 * l * m5 * r2v2
        - 750.0 * r3
        + 360.0 * l * r3
        + 1050.0 * m * r3
        - 750.0 * l * m * r3
        - 450.0 * m2 * r3
        + 525.0 * l * m2 * r3
        + 60.0 * m3 * r3
        - 150.0 * l * m3 * r3
        + 15.0 * l * m4 * r3
        + 104544.0 * u4v2
        - 40320.0 * l * u4v2
        - 210112.0 * m * u4v2
        + 104544.0 * l * m * u4v2
        + 162456.0 * m2 * u4v2
        - 105056.0 * l * m2 * u4v2
        - 62720.0 * m3 * u4v2
        + 54152.0 * l * m3 * u4v2
        + 12880.0 * m4 * u4v2
        - 15680.0 * l * m4 * u4v2
        - 1344.0 * m5 * u4v2
        + 2576.0 * l * m5 * u4v2
        + 56.0 * m6 * u4v2
        - 224.0 * l * m6 * u4v2
        + 8.0 * l * m7 * u4v2;
    let duv: f64 = 16.0 * pr * u * duv;
    duv
}

#[allow(unused_variables)]
fn duv_6_0(t: &SolEmTerms) -> f64 {
    let SolEmTerms {
        u,
        v,
        u2,
        r,
        l,
        pr,
        m,
        mm1,
    } = *t;
    let m2: f64 = m * m;
    let m3: f64 = m2 * m;
    let m4: f64 = m2 * m2;
    let m5: f64 = m2 * m3;
    let m6: f64 = m3 * m3;
    let u4: f64 = u2 * u2;
    let u6: f64 = u2 * u4;
    let r2: f64 = r * r;
    let r3: f64 = r2 * r;
    let r2u2: f64 = r2 * u2;
    let ru4: f64 = r * u4;
    let duv: f64 = 16440.0 * ru4 - 7200.0 * l * ru4 - 27000.0 * m * ru4
        + 16440.0 * l * m * ru4
        + 15300.0 * m2 * ru4
        - 13500.0 * l * m2 * ru4
        - 3600.0 * m3 * ru4
        + 5100.0 * l * m3 * ru4
        + 300.0 * m4 * ru4
        - 900.0 * l * m4 * ru4
        + 60.0 * l * m5 * ru4
        - 4500.0 * r2u2
        + 2160.0 * l * r2u2
        + 6300.0 * m * r2u2
        - 4500.0 * l * m * r2u2
        - 2700.0 * m2 * r2u2
        + 3150.0 * l * m2 * r2u2
        + 360.0 * m3 * r2u2
        - 900.0 * l * m3 * r2u2
        + 90.0 * l * m4 * r2u2
        + 165.0 * r3
        - 90.0 * l * r3
        - 180.0 * m * r3
        + 165.0 * l * m * r3
        + 45.0 * m2 * r3
        - 90.0 * l * m2 * r3
        + 15.0 * l * m3 * r3
        - 14112.0 * u6
        + 5760.0 * l * u6
        + 25984.0 * m * u6
        - 14112.0 * l * m * u6
        - 17640.0 * m2 * u6
        + 12992.0 * l * m2 * u6
        + 5600.0 * m3 * u6
        - 5880.0 * l * m3 * u6
        - 840.0 * m4 * u6
        + 1400.0 * l * m4 * u6
        + 48.0 * m5 * u6
        - 168.0 * l * m5 * u6
        + 8.0 * l * m6 * u6;
    let duv: f64 = 8.0 * pr * duv;
    duv
}
