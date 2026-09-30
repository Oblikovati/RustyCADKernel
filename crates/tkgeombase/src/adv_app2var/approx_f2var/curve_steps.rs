//! The steps of the iso-curve approximation `mma2fnc_`: the file-static `mma1nop_`
//! (parameters), `mma1fdi_` (sampling), `mma1cdi_` (constraint removal), `mma1jak_`
//! and `mmmapcoe_` (Jacobi projection), `mma1cnt_` (constraint addition),
//! with `mma1fer_`, `mmaperm_` and `mma1noc_` in the `curve_errors` module.
use super::bases::{mma1her_, mmapptt_};
use super::curve::IsoApproxRequest;
use crate::adv_app2var::evaluator::{AdvApp2VarEvaluation, AdvApp2VarEvaluatorFunc2Var};
use crate::adv_app2var::f2c::{F2cArray, F2cArrayMut};
use crate::adv_app2var::math_base::{mmmpocur_, pow__di};

/// `mma1nop_`: the `nbroot + 2` parameters of the iso sampling into `ttable`: the
/// interval ends around the roots mapped on the non-constant parameter of `uvfonc`
/// (U0, U1, V0, V1). `iercod` is 1 for an `isofav` other than 1 or 2 (and is not
/// cleared otherwise, as natively).
pub(super) fn mma1nop_(
    nbroot: i32,
    rootlg: &[f64],
    uvfonc: &[f64; 4],
    isofav: i32,
    ttable: &mut [f64],
    iercod: &mut i32,
) {
    let alinu: f64 = (uvfonc[1] - uvfonc[0]) / 2.;
    let blinu: f64 = (uvfonc[1] + uvfonc[0]) / 2.;
    let alinv: f64 = (uvfonc[3] - uvfonc[2]) / 2.;
    let blinv: f64 = (uvfonc[3] + uvfonc[2]) / 2.;
    let (ends, alin, blin) = match isofav {
        1 => ([uvfonc[2], uvfonc[3]], alinv, blinv),
        2 => ([uvfonc[0], uvfonc[1]], alinu, blinu),
        _ => {
            *iercod = 1;
            return;
        }
    };
    ttable[0] = ends[0];
    for ii in 1..=nbroot as usize {
        ttable[ii] = alin * rootlg[ii - 1] + blin;
    }
    ttable[(nbroot + 1) as usize] = ends[1];
}

/// The per-piece tables of `mma1fdi_`, `mma1cdi_` and `mma1cnt_`: the sums and
/// differences `(0:nbroot/2, ndimen)` and the end constraints `(ndimen, iordre+1)`.
pub(super) struct PieceTables<'a> {
    pub(super) somtab: &'a mut [f64],
    pub(super) diftab: &'a mut [f64],
    pub(super) contr1: &'a mut [f64],
    pub(super) contr2: &'a mut [f64],
}

/// `mma1fdi_`: samples the derivative `ideriv` of the function across the iso at
/// the parameters `ttable` into `fpntab`, folds the samples into sums and
/// differences and reads the end constraints up to order `iordre`, all scaled to
/// [-1,1]. `iercod` is the evaluator code plus 100 when it stops the sampling.
pub(super) fn mma1fdi_(
    req: &IsoApproxRequest<'_>,
    uvfonc: &[f64; 4],
    foncnp: &dyn AdvApp2VarEvaluatorFunc2Var,
    ttable: &[f64],
    fpntab: &mut [f64],
    piece: &mut PieceTables<'_>,
    iercod: &mut i32,
) {
    *iercod = 0;
    sample_iso(req, uvfonc, foncnp, ttable, fpntab, piece, iercod);
    if *iercod > 0 {
        *iercod += 100;
    }
}

fn sample_iso(
    req: &IsoApproxRequest<'_>,
    uvfonc: &[f64; 4],
    foncnp: &dyn AdvApp2VarEvaluatorFunc2Var,
    ttable: &[f64],
    fpntab: &mut [f64],
    piece: &mut PieceTables<'_>,
    iercod: &mut i32,
) {
    let (ndimen, nbroot, iordre) = (req.ndimen, req.nbroot, req.iordre);
    let (ideb, ifin) = if iordre >= 0 {
        (0, nbroot + 1)
    } else {
        (1, nbroot)
    };
    let (u_axis, span) = if req.isofav == 1 {
        (true, uvfonc[1] - uvfonc[0])
    } else {
        (false, uvfonc[3] - uvfonc[2])
    };
    let renor: f64 = pow__di(span / 2., req.ideriv);
    let orders = |d: i32| if u_axis { (d, 0) } else { (0, d) };
    let (ideru, iderv) = orders(req.ideriv);
    let call = |params: &[f64],
                nbp: i32,
                (u_order, v_order): (i32, i32),
                out: &mut [f64],
                code: &mut i32| {
        let request = AdvApp2VarEvaluation {
            dimension: ndimen,
            u_start_end: [uvfonc[0], uvfonc[1]],
            v_start_end: [uvfonc[2], uvfonc[3]],
            favor_iso: req.isofav,
            const_param: req.tconst,
            nb_params: nbp,
            parameters: params,
            u_order,
            v_order,
        };
        foncnp.evaluate(&request, out, code);
    };
    let fd1: i32 = ndimen;
    call(
        &ttable[ideb as usize..],
        ifin - ideb + 1,
        (ideru, iderv),
        &mut fpntab[(ideb * fd1) as usize..],
        iercod,
    );
    if *iercod > 0 {
        return;
    }
    fold_samples(req, renor, fpntab, piece);
    if iordre < 0 {
        return;
    }
    let fpnt = F2cArray::new(fpntab, 1);
    let mut c1 = F2cArrayMut::new(piece.contr1, fd1 + 1);
    let mut c2 = F2cArrayMut::new(piece.contr2, fd1 + 1);
    for nd in 1..=ndimen {
        c1[nd + fd1] = renor * fpnt[nd];
        c2[nd + fd1] = renor * fpnt[nd + (nbroot + 1) * fd1];
    }
    // The cross derivatives at both ends, in the native order: all of contr1, then contr2.
    let cross = |d: i32| {
        if u_axis {
            (req.ideriv, d)
        } else {
            (d, req.ideriv)
        }
    };
    for (table, parameter) in [(&mut c1, 0usize), (&mut c2, (nbroot + 1) as usize)] {
        for order in 1..=iordre {
            call(
                &ttable[parameter..],
                1,
                cross(order),
                table.tail_mut((order + 1) * fd1 + 1),
                iercod,
            );
            if *iercod > 0 {
                return;
            }
        }
    }
    let bid1: f64 = if u_axis {
        (uvfonc[3] - uvfonc[2]) / 2.
    } else {
        (uvfonc[1] - uvfonc[0]) / 2.
    };
    let mut bid2: f64 = renor;
    for ii in 1..=iordre {
        bid2 *= bid1;
        for nd in 1..=ndimen {
            c1[nd + (ii + 1) * fd1] *= bid2;
            c2[nd + (ii + 1) * fd1] *= bid2;
        }
    }
}

/// The folding of `mma1fdi_`: sums and differences of the samples at `+t` and
/// `-t`, times `renor`, and the middle sample of an odd count.
fn fold_samples(
    req: &IsoApproxRequest<'_>,
    renor: f64,
    fpntab: &[f64],
    piece: &mut PieceTables<'_>,
) {
    let (ndimen, nbroot) = (req.ndimen, req.nbroot);
    let fpnt = F2cArray::new(fpntab, 1);
    let sd1: i32 = nbroot / 2 + 1;
    let mut som = F2cArrayMut::new(piece.somtab, sd1);
    let mut dif = F2cArrayMut::new(piece.diftab, sd1);
    let nroo2: i32 = nbroot / 2;
    for nd in 1..=ndimen {
        for ii in 1..=nroo2 {
            let iip: i32 = (nbroot + 1) / 2 + ii;
            let iim: i32 = nroo2 - ii + 1;
            let (bid1, bid2) = (fpnt[nd + iim * ndimen], fpnt[nd + iip * ndimen]);
            som[ii + nd * sd1] = renor * (bid2 + bid1);
            dif[ii + nd * sd1] = renor * (bid2 - bid1);
        }
    }
    for nd in 1..=ndimen {
        let middle: f64 = if nbroot % 2 == 1 {
            renor * fpnt[nd + (nroo2 + 1) * ndimen]
        } else {
            0.
        };
        som[nd * sd1] = middle;
        dif[nd * sd1] = middle;
    }
}

/// `mma1cdi_`: subtracts from the folded samples the Hermite interpolation of the
/// end constraints, sampled at the roots into `fpntab` with the basis `hermit`.
/// `iercod` is 1 when the Hermite basis fails.
///
/// Native quirk: for an odd number of roots the middle correction uses only the
/// highest order's term (the order loop overwrites `bid3` before it is applied).
pub(super) fn mma1cdi_(
    req: &IsoApproxRequest<'_>,
    piece: &mut PieceTables<'_>,
    fpntab: &mut [f64],
    hermit: &mut [f64],
    iercod: &mut i32,
) {
    let (ndimen, nbroot, iordre) = (req.ndimen, req.nbroot, req.iordre);
    *iercod = 0;
    mma1her_(iordre, hermit, iercod);
    if *iercod > 0 {
        *iercod = 1;
        return;
    }
    let ncfhe: i32 = (iordre + 1) << 1;
    let her = F2cArray::new(hermit, ncfhe);
    let roots = F2cArray::new(req.rootlg, 1);
    let mut fp = F2cArrayMut::new(fpntab, nbroot + 1);
    for ii in 1..=ncfhe {
        for kk in 1..=nbroot {
            mmmpocur_(
                ncfhe,
                1,
                ncfhe,
                her.tail(ii * ncfhe),
                roots[kk],
                fp.tail_mut(kk + ii * nbroot),
            );
        }
    }
    let fp = |k: i32, col: i32| fp[k + col * nbroot];
    let sd1: i32 = nbroot / 2 + 1;
    let (c1, c2) = (
        F2cArray::new(piece.contr1, ndimen + 1),
        F2cArray::new(piece.contr2, ndimen + 1),
    );
    let mut som = F2cArrayMut::new(piece.somtab, sd1);
    let mut dif = F2cArrayMut::new(piece.diftab, sd1);
    let nroo2: i32 = nbroot / 2;
    for nd in 1..=ndimen {
        for ii in 1..=iordre + 1 {
            let (bid1, bid2) = (c1[nd + ii * ndimen], c2[nd + ii * ndimen]);
            for kk in 1..=nroo2 {
                let kkm: i32 = nroo2 - kk + 1;
                let bid3: f64 = bid1 * fp(kkm, (ii << 1) - 1) + bid2 * fp(kkm, ii << 1);
                som[kk + nd * sd1] -= bid3;
                dif[kk + nd * sd1] += bid3;
            }
            for kk in 1..=nroo2 {
                let kkp: i32 = (nbroot + 1) / 2 + kk;
                let bid3: f64 = bid1 * fp(kkp, (ii << 1) - 1) + bid2 * fp(kkp, ii << 1);
                som[kk + nd * sd1] -= bid3;
                dif[kk + nd * sd1] -= bid3;
            }
        }
    }
    if nbroot % 2 != 1 {
        return;
    }
    for nd in 1..=ndimen {
        let mut bid3: f64 = 0.;
        for ii in 1..=iordre + 1 {
            bid3 = fp(nroo2 + 1, (ii << 1) - 1) * c1[nd + ii * ndimen]
                + fp(nroo2 + 1, ii << 1) * c2[nd + ii * ndimen];
        }
        som[nd * sd1] -= bid3;
        dif[nd * sd1] -= bid3;
    }
}

/// `mma1jak_`: the Jacobi coefficients of the folded samples by Gauss projection
/// (`mmapptt_` then `mmmapcoe_`) into `crvjac` (`(0:ndgjac, ndimen)`); `iercod` is 33
/// when the projection table fails.
pub(super) fn mma1jak_(
    req: &IsoApproxRequest<'_>,
    piece: &PieceTables<'_>,
    cgauss: &mut [f64],
    crvjac: &mut [f64],
    iercod: &mut i32,
) {
    *iercod = 0;
    mmapptt_(req.ndgjac, req.nbroot, req.iordre, cgauss, iercod);
    if *iercod > 0 {
        *iercod = 33;
        return;
    }
    mmmapcoe_(req, piece.somtab, piece.diftab, cgauss, crvjac);
}

/// `mmmapcoe_`: the Gauss projections of the sums (even degrees) and differences
/// (odd degrees) from degree `2*(iordre+1)` on.
fn mmmapcoe_(
    req: &IsoApproxRequest<'_>,
    somtab: &[f64],
    diftab: &[f64],
    gsstab: &[f64],
    crvjac: &mut [f64],
) {
    let (ndim, ndgjac, nbpnts) = (req.ndimen, req.ndgjac, req.nbroot);
    let cd1: i32 = ndgjac + 1;
    let mut crv = F2cArrayMut::new(crvjac, cd1);
    let sd1: i32 = nbpnts / 2 + 1;
    let (som, dif) = (F2cArray::new(somtab, sd1), F2cArray::new(diftab, sd1));
    let gss = |ir: i32, igss: i32| gsstab[(ir + igss * sd1) as usize];
    let ikdeb: i32 = (req.iordre + 1) << 1;
    let nbroot: i32 = nbpnts / 2;
    for nd in 1..=ndim {
        for (start, table) in [(ikdeb, &som), (ikdeb + 1, &dif)] {
            for ik in (start..=ndgjac).step_by(2) {
                let igss: i32 = ik - ikdeb;
                let mut bidon: f64 = 0.;
                for ir in 1..=nbroot {
                    bidon += table[ir + nd * sd1] * gss(ir, igss);
                }
                crv[ik + nd * cd1] = bidon;
            }
        }
    }
    if nbpnts % 2 == 0 {
        return;
    }
    for nd in 1..=ndim {
        for ik in (ikdeb..=ndgjac).step_by(2) {
            let igss: i32 = ik - ikdeb;
            crv[ik + nd * cd1] += som[nd * sd1] * gss(0, igss);
        }
    }
}

/// `mma1cnt_`: the constraint coefficients `0..=2*iordre+1` of `crvjac`, the
/// Hermite interpolation of the end constraints.
pub(super) fn mma1cnt_(
    req: &IsoApproxRequest<'_>,
    piece: &PieceTables<'_>,
    hermit: &[f64],
    crvjac: &mut [f64],
) {
    let (ndimen, iordre) = (req.ndimen, req.iordre);
    let hd1: i32 = (iordre << 1) + 2;
    let her = F2cArray::new(hermit, hd1);
    let (c1, c2) = (
        F2cArray::new(piece.contr1, ndimen + 1),
        F2cArray::new(piece.contr2, ndimen + 1),
    );
    let cd1: i32 = req.ndgjac + 1;
    let mut crv = F2cArrayMut::new(crvjac, cd1);
    for nd in 1..=ndimen {
        for ii in 0..=(iordre << 1) + 1 {
            let mut bid: f64 = 0.;
            for jj in 1..=iordre + 1 {
                bid = bid
                    + c1[nd + jj * ndimen] * her[ii + ((jj << 1) - 1) * hd1]
                    + c2[nd + jj * ndimen] * her[ii + (jj << 1) * hd1];
            }
            crv[ii + nd * cd1] = bid;
        }
    }
}
