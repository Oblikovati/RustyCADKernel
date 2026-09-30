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
// Rust translation of the static helpers sortbounds, coonscnd and killcorners of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_ConstrainedFilling.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The sort of the boundaries into a loop and the corner checks of
//! `GeomFill_ConstrainedFilling::Init`.
use super::{field, live};
use crate::geom_fill::{GeomFillBoundary, GeomFillCornerState, GeomFillTgtField};
use crate::law;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Point3, Vector3};
use std::f64::consts::PI;

type Bound = Handle<dyn GeomFillBoundary>;

/// `Bounds(f, l)`, swapped when the boundary is reversed.
fn oriented_bounds(bound: &dyn GeomFillBoundary, rev: bool) -> Result<(f64, f64), Failure> {
    let (f, l) = bound.bounds()?;
    Ok(if rev { (l, f) } else { (f, l) })
}

/// `Points(pf, pl)`, swapped when the boundary is reversed.
fn oriented_points(bound: &dyn GeomFillBoundary, rev: bool) -> Result<(Point3, Point3), Failure> {
    let (f, l) = bound.points()?;
    Ok(if rev { (l, f) } else { (f, l) })
}

/// `sortbounds`: chains the boundaries by nearest ends (OCC6129: each candidate
/// replaces the chosen one when its gap beats the gap stored in the corner state,
/// which a previous `Init` may have left), records the gaps, and the angles between
/// the tangents and between the normals at each corner.
pub(super) fn sort_bounds(
    bound: &mut [Bound],
    rev: &mut [bool],
    stat: &mut [GeomFillCornerState],
) -> Result<(), Failure> {
    let nb = bound.len();
    rev[0] = false;
    for i in 0..nb - 1 {
        let (_, pl) = oriented_points(live(&bound[i])?, rev[i])?;
        for j in i + 1..nb {
            let (qf, ql) = live(&bound[j])?.points()?;
            let (df, dl) = (qf.distance(&pl), ql.distance(&pl));
            let (gap, reversed) = if df < dl { (df, false) } else { (dl, true) };
            if gap < stat[i + 1].gap() {
                bound.swap(i + 1, j);
                stat[i + 1].set_gap(gap);
                rev[i + 1] = reversed;
            }
        }
    }
    let (_, pl) = oriented_points(live(&bound[nb - 1])?, rev[nb - 1])?;
    let (qf, _) = live(&bound[0])?.points()?;
    stat[0].set_gap(pl.distance(&qf));
    for i in 0..nb {
        corner_angles((bound, rev), i, stat)?;
    }
    Ok(())
}

/// The angles at the corner between boundary `i` and the next one.
fn corner_angles(
    (bound, rev): (&[Bound], &[bool]),
    i: usize,
    stat: &mut [GeomFillCornerState],
) -> Result<(), Failure> {
    let next: usize = (i + 1) % bound.len();
    let (bi, bn) = (live(&bound[i])?, live(&bound[next])?);
    let (_, li) = oriented_bounds(bi, rev[i])?;
    let mut tgi: Vector3 = bi.d1(li)?.1;
    if rev[i] {
        tgi.reverse();
    }
    let (fn_, _) = oriented_bounds(bn, rev[next])?;
    let mut tgn: Vector3 = bn.d1(fn_)?.1;
    if rev[next] {
        tgn.reverse();
    }
    stat[next].set_tgt_ang(PI - tgi.angle(tgn)?);
    if bi.has_normals() && bn.has_normals() {
        stat[next].constraint();
        let (nori, norn) = (bi.norm(li)?, bn.norm(fn_)?);
        stat[next].set_nor_ang(nori.angle(norn)?);
    }
    Ok(())
}

/// `coonscnd`: at each constrained corner, the normals must agree within the
/// smaller angular tolerance (else the corner is killed), and the twist of the two
/// boundaries must satisfy the Coons compatibility within that tolerance scaled by
/// the minimal tangent field magnitudes (else the corner is killed with a factor).
pub(super) fn coons_cnd(
    bound: &[Bound],
    rev: &[bool],
    stat: &mut [GeomFillCornerState],
    mintg: &[f64],
) -> Result<(), Failure> {
    let nb = bound.len();
    for i in 0..nb {
        if !stat[i].has_constraint() {
            continue;
        }
        let ip: usize = (i + nb - 1) % nb;
        let (bp, bi) = (live(&bound[ip])?, live(&bound[i])?);
        let tolang: f64 = bp.tolang().min(bi.tolang());
        let (an, twist) = folded_angle(stat[i].nor_ang());
        if an > tolang {
            stat[i].do_kill(0.);
            continue;
        }
        let fact: f64 = 0.5 * 27. / 4.;
        let tolang: f64 = tolang * (mintg[ip].min(mintg[i]) * fact * 100.);
        let scal1: f64 = twist_mismatch((bp, rev[ip]), (bi, rev[i]), twist)?;
        if scal1 > tolang {
            stat[i].do_kill(tolang / scal1);
        }
    }
    Ok(())
}

/// The normal angle folded below a right angle, and whether it was folded (twist).
fn folded_angle(an: f64) -> (f64, bool) {
    if an >= 0.5 * PI {
        return (PI - an, true);
    }
    (an, false)
}

/// `|tgp . dnori +- tgi . dnorp|`: the Coons compatibility of the twists at the end
/// of the previous boundary `p` and the start of boundary `i`.
fn twist_mismatch(
    (bp, revp): (&dyn GeomFillBoundary, bool),
    (bi, revi): (&dyn GeomFillBoundary, bool),
    twist: bool,
) -> Result<f64, Failure> {
    let (_, lp) = oriented_bounds(bp, revp)?;
    let tgp: Vector3 = bp.d1(lp)?.1;
    let dnorp: Vector3 = bp.d1_norm(lp)?.1;
    let (fi, _) = oriented_bounds(bi, revi)?;
    let tgi: Vector3 = bi.d1(fi)?.1;
    let dnori: Vector3 = bi.d1_norm(fi)?.1;
    let scal1: f64 = tgp.dot(dnori);
    let mut scal2: f64 = tgi.dot(dnorp);
    if !twist {
        scal2 *= -1.;
    }
    Ok((scal1 + scal2).abs())
}

/// `killcorners`: each boundary with a killed end is reparametrized on [0, 1] with
/// the scale of that end imposed as its end derivative, and its tangent field is
/// scaled when it can be (`nrev` gives the orientation of the boundaries in the loop).
pub(super) fn kill_corners(
    bound: &[Bound],
    (rev, nrev): (&[bool], &[bool]),
    stat: &[GeomFillCornerState],
    tga: &[Handle<dyn GeomFillTgtField>],
) -> Result<(), Failure> {
    let nb = bound.len();
    for i in 0..nb {
        let inext: usize = (i + 1) % nb;
        let (first, last) = if nrev[i] { (inext, i) } else { (i, inext) };
        let ((fnul, fscal), (lnul, lscal)) = (stat[first].is_to_kill(), stat[last].is_to_kill());
        if !(fnul || lnul) {
            continue;
        }
        let bi: &dyn GeomFillBoundary = live(&bound[i])?;
        bi.reparametrize(0., 1., fnul, lnul, fscal, lscal, rev[i])?;
        if bi.has_normals() && field(&tga[i])?.is_scalable() {
            let bs = law::scale_cub(0., 1., fnul, lnul, fscal, lscal)?;
            field(&tga[i])?.scale(&bs)?;
        }
    }
    Ok(())
}
