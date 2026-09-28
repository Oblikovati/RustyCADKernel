// Copyright (c) 1996-1999 Matra Datavision
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

//! The file-static `mma2ce2_`: projection of the square on the Jacobi basis,
//! error estimates and the cutting decision, with the native labels as the
//! [`Exit`] variants.
use super::patch_approx::{Ce2State, PatchApproxOutput, subspace_block};
use super::projection::{mma2er2_, mma2moy_};
use crate::adv_app2var::math_base::mzsnorm_;
use crate::adv_app2var::sys_base::mvriraz_;

/// The exits of `mma2ce2_` besides the normal end: the error L9001, the cuttings
/// L9100 (U first), L9200 (V first) and L9300 (both).
enum Exit {
    Invalid,
    CutU,
    CutV,
    CutBoth,
}

/// `mma2ce2_`: see [`mma2ce1_`](super::mma2ce1_).
pub(super) fn mma2ce2_(
    state: &mut Ce2State<'_>,
    out: &mut PatchApproxOutput<'_>,
    iercod: &mut i32,
) {
    *iercod = 0;
    let numdec: i32 = state.r.numdec;
    let exit: Option<Exit> = if arguments_are_valid(state) {
        run(state, out, iercod)
    } else {
        Some(Exit::Invalid)
    };
    let Some(exit) = exit else { return };
    let cutting: bool = (1..=5).contains(&numdec);
    out.itydec = match exit {
        Exit::CutU if cutting => {
            if numdec != 2 {
                1
            } else {
                2
            }
        }
        Exit::CutV if cutting => {
            if numdec != 1 {
                2
            } else {
                1
            }
        }
        Exit::CutBoth if cutting => match numdec {
            5 => 3,
            2 | 4 => 2,
            _ => 1,
        },
        _ => {
            *iercod = 1;
            return;
        }
    };
}

/// The argument checks of `mma2ce2_` (label L9001).
fn arguments_are_valid(state: &Ce2State<'_>) -> bool {
    let r = state.r;
    (0..=5).contains(&r.numdec)
        && (r.iordru << 1) < r.ndminu
        && r.ndminu <= r.ndguli
        && r.ndguli < r.ndjacu
        && (r.iordrv << 1) < r.ndminv
        && r.ndminv <= r.ndgvli
        && r.ndgvli < r.ndjacv
}

fn run(
    state: &mut Ce2State<'_>,
    out: &mut PatchApproxOutput<'_>,
    iercod: &mut i32,
) -> Option<Exit> {
    let r = state.r;
    out.itydec = 0;
    out.ndegpu = r.ndminu;
    out.ndegpv = r.ndminv;
    mvriraz_(r.nbsesp, out.errmax);
    mvriraz_(r.ndimen << 2, &mut state.vecerr);
    mvriraz_((r.ndjacu + 1) * (r.ndjacv + 1) * r.ndimen, out.patjac);
    if r.numdec > 0 {
        cutting_path(state, out)
    } else {
        final_path(state, out, iercod);
        None
    }
}

/// The `numdec > 0` path: the high-degree bands decide the cutting, then the
/// kept coefficients are computed and truncated.
fn cutting_path(state: &mut Ce2State<'_>, out: &mut PatchApproxOutput<'_>) -> Option<Exit> {
    let r = state.r;
    let (i2rdu, i2rdv) = state.i2rd();
    if r.ndguli + 1 > r.ndjacu || r.ndgvli + 1 > r.ndjacv {
        return Some(Exit::Invalid);
    }
    if let Some(exit) = high_u_high_v_band(state, out) {
        return Some(exit);
    }
    if i2rdu <= r.ndguli {
        project_band(state, out, [i2rdu, r.ndguli], [r.ndgvli + 1, r.ndjacv]);
        state.error_columns(out.patjac, [i2rdu, r.ndguli, r.ndgvli + 1, r.ndjacv], 2);
    }
    if i2rdv <= r.ndgvli {
        for jj in i2rdv..=r.ndgvli {
            for nd in 1..=r.ndimen {
                state.project_v(out.patjac, jj, [r.ndguli + 1, r.ndjacu], nd);
            }
        }
        state.error_columns(out.patjac, [r.ndguli + 1, r.ndjacu, i2rdv, r.ndgvli], 3);
    }
    if let Some(exit) = cutting_decision(state, out) {
        return Some(exit);
    }
    if i2rdu <= r.ndguli && i2rdv <= r.ndgvli {
        for jj in i2rdv..=r.ndgvli {
            for nd in 1..=r.ndimen {
                state.project_v(out.patjac, jj, [i2rdu, r.ndguli], nd);
            }
        }
    }
    for nd in 1..=r.nbsesp {
        truncate_subspace(state, out, nd, None);
    }
    None
}

/// Projects the U degrees `u_range` of every dimension, then the V degrees
/// `v_range` on them (the `mma2cfu_` then `mma2cfv_` loops of `mma2ce2_`).
fn project_band(
    state: &mut Ce2State<'_>,
    out: &mut PatchApproxOutput<'_>,
    u_range: [i32; 2],
    v_range: [i32; 2],
) {
    for nd in 1..=state.r.ndimen {
        state.project_u(nd, u_range);
    }
    for jj in v_range[0]..=v_range[1] {
        for nd in 1..=state.r.ndimen {
            state.project_v(out.patjac, jj, u_range, nd);
        }
    }
}

/// The first band of `mma2ce2_`: the degrees above both limits, checked row by row
/// against the tolerance of each subspace (column 4 of `vecerr`); exits with a
/// double cutting as soon as one row exceeds it.
fn high_u_high_v_band(state: &mut Ce2State<'_>, out: &mut PatchApproxOutput<'_>) -> Option<Exit> {
    let r = state.r;
    let (minu, maxu) = (r.ndguli + 1, r.ndjacu);
    for nd in 1..=r.ndimen {
        state.project_u(nd, [minu, maxu]);
    }
    for jj in r.ndgvli + 1..=r.ndjacv {
        for nd in 1..=r.ndimen {
            state.project_v(out.patjac, jj, [minu, maxu], nd);
        }
        let mut idim: i32 = 1;
        for nd in 1..=r.nbsesp {
            let ndses: i32 = r.ndimse[(nd - 1) as usize];
            state.error_row(out.patjac, idim, ndses, [minu, maxu, jj, jj], nd);
            if state.vecerr_at(nd, 4) > r.epsapr[(nd - 1) as usize] {
                return Some(Exit::CutBoth);
            }
            idim += ndses;
        }
    }
    None
}

/// Label L400 of `mma2ce2_`: the combined error of the three high bands decides
/// which direction to cut when a subspace exceeds its tolerance.
fn cutting_decision(state: &Ce2State<'_>, out: &mut PatchApproxOutput<'_>) -> Option<Exit> {
    for nd in 1..=state.r.nbsesp {
        let vaux: [f64; 3] = [
            state.vecerr_at(nd, 2),
            state.vecerr_at(nd, 4),
            state.vecerr_at(nd, 3),
        ];
        let eps: f64 = state.r.epsapr[(nd - 1) as usize];
        out.errmax[(nd - 1) as usize] = mzsnorm_(3, &vaux);
        if out.errmax[(nd - 1) as usize] > eps {
            let zv: f64 = mzsnorm_(2, &vaux);
            let zu: f64 = mzsnorm_(2, &vaux[1..]);
            if zu > eps && zv > eps {
                return Some(Exit::CutBoth);
            }
            return Some(if zu > zv { Exit::CutU } else { Exit::CutV });
        }
    }
    None
}

/// The `numdec == 0` path: every coefficient is computed, the errors of the bands
/// beyond the limits are measured, and the degrees are truncated; a subspace out
/// of tolerance keeps the limit degrees and sets `iercod = -1`.
fn final_path(state: &mut Ce2State<'_>, out: &mut PatchApproxOutput<'_>, iercod: &mut i32) {
    let r = state.r;
    let (i2rdu, i2rdv) = state.i2rd();
    for nd in 1..=r.ndimen {
        state.project_u(nd, [i2rdu, r.ndjacu]);
        for jj in i2rdv..=r.ndjacv {
            state.project_v(out.patjac, jj, [i2rdu, r.ndjacu], nd);
        }
    }
    let mut idim: i32 = 1;
    for nd in 1..=r.nbsesp {
        let ndses: i32 = r.ndimse[(nd - 1) as usize];
        let mut errmax: f64 = out.errmax[(nd - 1) as usize];
        state.error_into(
            out.patjac,
            idim,
            ndses,
            [i2rdu, r.ndjacu, r.ndgvli + 1, r.ndjacv],
            &mut errmax,
        );
        if i2rdv <= r.ndgvli {
            state.error_into(
                out.patjac,
                idim,
                ndses,
                [r.ndguli + 1, r.ndjacu, i2rdv, r.ndgvli],
                &mut errmax,
            );
        }
        out.errmax[(nd - 1) as usize] = errmax;
        let limits: Option<[i32; 2]> = if errmax > r.epsapr[(nd - 1) as usize] {
            *iercod = -1;
            Some([r.ndguli, r.ndgvli])
        } else {
            None
        };
        truncate_subspace(state, out, nd, limits);
        idim += ndses;
    }
}

/// Label L600 of `mma2ce2_` and the end of the `numdec == 0` loop for subspace
/// `nd`: the degrees `(nu, nv)` from `mma2er2_` (or `limits`), the average error,
/// the zeroed coefficients beyond them and the running maximum degrees.
fn truncate_subspace(
    state: &mut Ce2State<'_>,
    out: &mut PatchApproxOutput<'_>,
    nd: i32,
    limits: Option<[i32; 2]>,
) {
    let r = state.r;
    let idim: i32 = 1 + r.ndimse[..(nd - 1) as usize].iter().sum::<i32>();
    let ndses: i32 = r.ndimse[(nd - 1) as usize];
    let [nu, nv] = limits.unwrap_or_else(|| state.lowered_degrees(out, nd, idim, ndses));
    let (nu1, nv1) = (nu + 1, nv + 1);
    let (pd1, pd2) = (r.ndjacu + 1, r.ndjacv + 1);
    let block_start: usize = ((idim - 1) * pd1 * pd2) as usize;
    let range: [i32; 4] = [nu1, r.ndjacu, nv1, r.ndjacv];
    mma2moy_(
        [r.ndjacu, r.ndjacv],
        ndses,
        range,
        [r.iordru, r.iordrv],
        &out.patjac[block_start..],
        &mut out.errmoy[(nd - 1) as usize],
    );
    for ii in idim..=idim + ndses - 1 {
        for jj in nv1..=r.ndjacv {
            for kk in nu1..=r.ndjacu {
                out.patjac[(kk + (jj + ii * pd2) * pd1 - pd1 * pd2) as usize] = 0.;
            }
        }
    }
    out.ndegpu = out.ndegpu.max(nu);
    out.ndegpv = out.ndegpv.max(nv);
}

impl Ce2State<'_> {
    /// `mma2er1_` on one row band of subspace `nd`, accumulating into `vecerr(nd, 4)`.
    fn error_row(&mut self, patjac: &[f64], idim: i32, ndses: i32, range: [i32; 4], nd: i32) {
        let ndimen: usize = self.r.ndimen as usize;
        let mut erreur: f64 = self.vecerr_at(nd, 4);
        self.error_into(patjac, idim, ndses, range, &mut erreur);
        self.vecerr[3 * ndimen + (nd - 1) as usize] = erreur;
    }

    /// `mma2er1_` on `range` of the subspace at `idim` into `erreur`, with the
    /// first column of `vecerr` as scratch.
    fn error_into(
        &mut self,
        patjac: &[f64],
        idim: i32,
        ndses: i32,
        range: [i32; 4],
        erreur: &mut f64,
    ) {
        let ndimen: usize = self.r.ndimen as usize;
        let block = subspace_block(self.r, [self.xmaxju, self.xmaxjv], patjac, idim, ndses);
        super::projection::mma2er1_(&block, range, &mut self.vecerr[..ndimen], erreur);
    }

    /// The degree search of label L600: `mma2er2_` when the limits leave room above
    /// the constraint degrees, the limits themselves otherwise.
    fn lowered_degrees(
        &mut self,
        out: &mut PatchApproxOutput<'_>,
        nd: i32,
        idim: i32,
        ndses: i32,
    ) -> [i32; 2] {
        let r = self.r;
        let minu: i32 = 1.max((r.iordru << 1) + 1).max(r.ndminu);
        let minv: i32 = 1.max((r.iordrv << 1) + 1).max(r.ndminv);
        let (maxu, maxv) = (r.ndguli, r.ndgvli);
        let (i2rdu, i2rdv) = self.i2rd();
        if maxu < i2rdu || maxv < i2rdv {
            return [maxu, maxv];
        }
        let ndimen: usize = r.ndimen as usize;
        let mut degrees: [i32; 2] = [0; 2];
        let block = subspace_block(r, [self.xmaxju, self.xmaxjv], out.patjac, idim, ndses);
        let eps: f64 = r.epsapr[(nd - 1) as usize];
        mma2er2_(
            &block,
            [minu, maxu, minv, maxv],
            eps,
            &mut self.vecerr[..ndimen],
            &mut out.errmax[(nd - 1) as usize],
            &mut degrees,
        );
        degrees
    }
}
