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
// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves_ResolConstraint.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The rows of `AppParCurves_ResolConstraint` that give consecutive curves the same
//! tangency ratio, on their most significant tangent coordinates.
use super::ResolConstraint;
use super::rows::{Tangent, out_of_range, put};
use crate::app_par_curves::line_tool::LineTool;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Vector2, Vector3};
use rustycad_tkmath::math::Matrix;

/// The native `tabV` and `tabV2d`: allocated once per constructor, so a tool that
/// leaves them unchanged (no tangent) keeps the previous values.
pub(super) struct TangentTables {
    pub(super) tab_v: Array1<'static, Vector3>,
    pub(super) tab_v2d: Array1<'static, Vector2>,
}

/// Which coordinate (1, 2 or 3) of curve `k`'s tangent a row reads, and whether it
/// is the first variable (`T1`) or the second (`T2`).
#[derive(Clone, Copy)]
enum Slot {
    First,
    Second,
}

impl TangentTables {
    // The tangent coordinate `code` of curve k at npt into T1 or T2; the 3D case 2 of
    // T1 reads the tangent without assigning T1 (a native quirk: T1 keeps its value).
    fn read(
        &mut self,
        ssp: &impl LineTool,
        [npt, k, nb3d, code]: [i32; 4],
        slot: Slot,
        t: &mut Tangent,
    ) -> Result<(), Failure> {
        let value: f64 = if k <= nb3d || code == 3 {
            ssp.tangency_3d(npt, &mut self.tab_v)?;
            let v: Vector3 = *self.tab_v.value(k).map_err(out_of_range)?;
            if code == 2 && matches!(slot, Slot::First) {
                return Ok(());
            }
            [v.x(), v.y(), v.z()][(code - 1) as usize]
        } else {
            ssp.tangency_2d(npt, &mut self.tab_v2d)?;
            let v: Vector2 = *self.tab_v2d.value(k - nb3d).map_err(out_of_range)?;
            [v.x(), v.y()][(code - 1) as usize]
        };
        match slot {
            Slot::First => t.t1 = value,
            Slot::Second => t.t2 = value,
        }
        Ok(())
    }
}

impl ResolConstraint {
    // One row per tangency point and pair of consecutive curves (k, k + 1):
    // `Daij T2` on curve k's coordinate and `-Daij T1` on curve k+1's.
    pub(super) fn ratio_rows(
        &mut self,
        ssp: &impl LineTool,
        derivative_bern: &Matrix<'_>,
        ibont: &Array2<'_, i32>,
        [nb3d, npol, mut inc3]: [i32; 3],
        t: &mut Tangent,
        tabs: &mut TangentTables,
    ) -> Result<(), Failure> {
        let nb_cu: i32 = ssp.nb_p3d() + ssp.nb_p2d();
        for i in 1..=self.inc_tan {
            let npt: i32 = self.i_tan[(i - 1) as usize];
            let mut inc_col: i32 = 0;
            for k in 1..nb_cu {
                inc3 += 1;
                let (code1, code2) = (ibont[(k, i)], ibont[(k + 1, i)]);
                tabs.read(ssp, [npt, k, nb3d, code1], Slot::First, t)?;
                t.ip = if k <= nb3d || code1 == 3 {
                    3 * npol
                } else {
                    2 * npol
                };
                tabs.read(ssp, [npt, k + 1, nb3d, code2], Slot::Second, t)?;
                let (add1, add2) = ((code1 - 1) * npol, (code2 - 1) * npol);
                for j in 1..=npol {
                    let daij: f64 = *derivative_bern.value(npt, j)?;
                    put(&mut self.cont, inc3, j + inc_col + add1, daij * t.t2)?;
                    put(
                        &mut self.cont,
                        inc3,
                        j + t.ip + inc_col + add2,
                        -daij * t.t1,
                    )?;
                }
                inc_col += t.ip;
            }
        }
        Ok(())
    }
}
