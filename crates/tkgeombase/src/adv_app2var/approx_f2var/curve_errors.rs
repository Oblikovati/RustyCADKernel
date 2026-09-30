//! The truncation and normalisation steps of `mma2fnc_`: the file-static `mma1fer_`
//! and `mmaperm_` (truncation and errors) and `mma1noc_` (normalisation).
use super::curve::IsoApproxRequest;
use crate::adv_app2var::f2c::F2cArray;
use crate::adv_app2var::math_base::{mmaperx_, mmtrpjj_, pow__di};
use crate::adv_app2var::sys_base::mvriraz_;

/// The per-piece outputs of `mma1fer_`: the errors per subspace and the kept
/// coefficient count.
pub(super) struct PieceErrors<'a> {
    pub(super) errmax: &'a mut [f64],
    pub(super) errmoy: &'a mut [f64],
    pub(super) ncoeff: i32,
}

/// `mma1fer_`: truncates each subspace of `crvjac` to its tolerance within `ncflim`
/// coefficients (zeroing the rest) and measures the errors; `iercod` is -1 when a
/// subspace needs more than `ncflim`, 1 when `mmaperx_` fails.
pub(super) fn mma1fer_(
    req: &IsoApproxRequest<'_>,
    crvjac: &mut [f64],
    ycvmax: &mut [f64],
    errors: &mut PieceErrors<'_>,
    iercod: &mut i32,
) {
    *iercod = 0;
    let (ncflim, iordre) = (req.ncflim, req.iordre);
    let ncfja: i32 = req.ndgjac + 1;
    let mut idim: i32 = 1;
    errors.ncoeff = 0;
    for ii in 1..=req.nbsesp {
        let k: usize = (ii - 1) as usize;
        let ndses: i32 = req.ndimse[k];
        let column: usize = ((idim - 1) * ncfja) as usize;
        let mut ncfnw: i32 = 0;
        mmtrpjj_(
            ncfja,
            ndses,
            ncfja,
            req.epsapr[k],
            iordre,
            &crvjac[column..],
            ycvmax,
            &mut errors.errmax[k],
            &mut ncfnw,
        );
        if ncfnw <= ncflim {
            mmaperm_(
                ncfja,
                ndses,
                ncfja,
                iordre,
                &crvjac[column..],
                ncfnw,
                &mut errors.errmoy[k],
            );
            errors.ncoeff = errors.ncoeff.max(ncfnw);
            let nbr0: i32 = ncflim - ncfnw;
            if nbr0 > 0 {
                for kk in 1..=ndses {
                    let start: usize = (ncfnw + (idim + kk - 2) * ncfja) as usize;
                    mvriraz_(nbr0, &mut crvjac[start..]);
                }
            }
        } else {
            *iercod = -1;
            let mut ier: i32 = 0;
            mmaperx_(
                ncfja,
                ndses,
                ncfja,
                iordre,
                &crvjac[column..],
                ncflim,
                ycvmax,
                &mut errors.errmax[k],
                &mut ier,
            );
            if ier > 0 {
                *iercod = 1;
                return;
            }
            errors.ncoeff = ncflim;
            mmaperm_(
                ncfja,
                ndses,
                ncfja,
                iordre,
                &crvjac[column..],
                ncflim,
                &mut errors.errmoy[k],
            );
        }
        idim += ndses;
    }
}

/// `mmaperm_`: the average error of dropping the coefficients past `ncfnew` (and
/// past the constraint ones) of the curve `crvjac` (`(ncofmx, ndim)`).
fn mmaperm_(
    ncofmx: i32,
    ndim: i32,
    ncoeff: i32,
    iordre: i32,
    crvjac: &[f64],
    ncfnew: i32,
    errmoy: &mut f64,
) {
    let crv = F2cArray::new(crvjac, ncofmx + 1);
    let ia: i32 = (iordre + 1) << 1;
    let mut ncfcut: i32 = ia + 1;
    if ncfnew + 1 > ncfcut {
        ncfcut = ncfnew + 1;
    }
    *errmoy = 0.;
    let mut bid: f64 = 0.;
    for nd in 1..=ndim {
        for i in ncfcut..=ncoeff {
            let bidj: f64 = crv[i + nd * ncofmx];
            bid += bidj * bidj;
        }
    }
    bid /= 2.;
    *errmoy = bid.sqrt();
}

/// `mma1noc_`: rescales `ndimen` values of a derivative of order `ideriv` across
/// and `iordre` along the iso from the domain `dfuvin` to `duvout`, in place
/// (the native calls pass the same array as input and output).
pub(super) fn mma1noc_(
    dfuvin: &[f64; 4],
    ndimen: i32,
    iordre: i32,
    values: &mut [f64],
    duvout: &[f64; 4],
    isofav: i32,
    ideriv: i32,
) {
    let ratio_u: f64 = (dfuvin[1] - dfuvin[0]) / (duvout[1] - duvout[0]);
    let ratio_v: f64 = (dfuvin[3] - dfuvin[2]) / (duvout[3] - duvout[2]);
    let (rider, riord) = if isofav == 1 {
        (pow__di(ratio_u, ideriv), pow__di(ratio_v, iordre))
    } else {
        (pow__di(ratio_v, ideriv), pow__di(ratio_u, iordre))
    };
    let bid: f64 = rider * riord;
    for value in values.iter_mut().take(ndimen.max(0) as usize) {
        *value *= bid;
    }
}
