//! Gauss-Legendre integration and curve length: `mmgaus1_`, `mmcglc1_`, `mmrtptt_`
//! and the file-static `mmloncv_`, `mvgaus0_`, `mmextrl_`, `mmexthi_`.
use crate::adv_app2var::data::getmlgdrtl;
use crate::adv_app2var::f2c::{F2cArray, F2cArrayMut};
use crate::adv_app2var::sys_base::{mcrfill_, mvriraz_};

/// `mmextrl_`: the `ndegre` Legendre roots, from table `rootab`, into
/// `rootlg[1..=ndegre]` (a 1-based view): the negative half mirrors the positive one.
fn mmextrl_(ndegre: i32, rootlg: &mut F2cArrayMut<'_, f64>) {
    let rootab: &[f64] = getmlgdrtl().rootab;
    let (ndeg2, nmod2) = (ndegre / 2, ndegre % 2);
    let iadd: i32 = ndeg2 * (ndeg2 - 1) / 2 + 1;
    let ideb: i32 = (ndegre + 1) / 2 + 1;
    for ii in ideb..=ndegre {
        let kpt: i32 = iadd + ii - ideb;
        rootlg[ii] = rootab[(kpt + nmod2 * 465 - 1) as usize];
    }
    for ii in 1..=ndeg2 {
        rootlg[ii] = -rootlg[ndegre + 1 - ii];
    }
    if nmod2 == 1 {
        rootlg[ndeg2 + 1] = 0.;
    }
}

/// `mmexthi_`: the `ndegre` Gauss-Legendre weights, from tables `hiltab` and
/// `hi0tab`, into `hwgaus[1..=ndegre]`.
fn mmexthi_(ndegre: i32, hwgaus: &mut F2cArrayMut<'_, f64>) {
    let tables = getmlgdrtl();
    let (ndeg2, nmod2) = (ndegre / 2, ndegre % 2);
    let iadd: i32 = ndeg2 * (ndeg2 - 1) / 2 + 1;
    let ideb: i32 = (ndegre + 1) / 2 + 1;
    for ii in ideb..=ndegre {
        let kpt: i32 = iadd + ii - ideb;
        hwgaus[ii] = tables.hiltab[(kpt + nmod2 * 465 - 1) as usize];
    }
    for ii in 1..=ndeg2 {
        hwgaus[ii] = hwgaus[ndegre + 1 - ii];
    }
    if nmod2 == 1 {
        hwgaus[ndeg2 + 1] = tables.hi0tab[ndeg2 as usize];
    }
}

/// `mvgaus0_`: the `nbrval = 2*kindic` positive roots (negated as natively) and
/// weights of the Gauss rule of degree `4*kindic`; a `kindic` outside 1..=10 is
/// replaced by 10 with `iercod = -1`.
fn mvgaus0_(
    kindic: i32,
    urootl: &mut [f64],
    hiltab: &mut [f64],
    nbrval: &mut i32,
    iercod: &mut i32,
) {
    let mut tampc: [f64; 40] = [0.; 40];
    let mut tamp = F2cArrayMut::new(&mut tampc, 1);
    *iercod = 0;
    let mut kg: i32 = kindic;
    if !(1..=10).contains(&kg) {
        kg = 10;
        *iercod = -1;
    }
    *nbrval = kg << 1;
    let ndegl: i32 = *nbrval << 1;
    mmextrl_(ndegl, &mut tamp);
    for ii in 1..=*nbrval {
        urootl[(ii - 1) as usize] = -tamp[ii];
    }
    mmexthi_(ndegl, &mut tamp);
    for ii in 1..=*nbrval {
        hiltab[(ii - 1) as usize] = tamp[ii];
    }
}

/// The function `mmgaus1_` integrates: `(ninteg, parame, vfunj1, iercod)`, the
/// native `bfunx` callback.
pub type GaussIntegrand<'a> = dyn FnMut(i32, f64, &mut [f64], &mut i32) + 'a;

/// Integrates the vector function `bfunx` on `[xd, xf]` with the Gauss rule of index
/// `k` (`mmgaus1_`): `somme` receives the `ndimf` integrals, `niter` the number of
/// evaluations. `bfunx(ninteg, parame, vfunj1, iercod)` fills `vfunj1` and must
/// clear `iercod`: any non-zero code stops the integration, including the -1 that
/// an invalid `k` leaves before the first call. Returns 0.
/// `mmgaus1_(1, &mut f, 3, 0.0, 1.0, &mut a1, &mut a2, &mut s, &mut n, &mut ier)`.
pub fn mmgaus1_(
    ndimf: i32,
    bfunx: &mut GaussIntegrand<'_>,
    k: i32,
    xd: f64,
    xf: f64,
    saux1: &mut [f64],
    saux2: &mut [f64],
    somme: &mut [f64],
    niter: &mut i32,
    iercod: &mut i32,
) -> i32 {
    let (mut h, mut u) = ([0.; 20], [0.; 20]);
    let mut ndeg: i32 = 0;
    mvriraz_(ndimf, somme);
    *iercod = 0;
    mvgaus0_(k, &mut u, &mut h, &mut ndeg, iercod);
    if *iercod > 0 {
        return 0;
    }
    let c1x: f64 = (xf + xd) * 0.5;
    let c2x: f64 = (xf - xd) * 0.5;
    for j in 1..=ndeg {
        let t: f64 = c2x * u[(j - 1) as usize];
        bfunx(ndimf, c1x + t, saux1, iercod);
        if *iercod != 0 {
            return 0;
        }
        bfunx(ndimf, c1x - t, saux2, iercod);
        if *iercod != 0 {
            return 0;
        }
        for idimf in 0..ndimf as usize {
            somme[idimf] += h[(j - 1) as usize] * (saux1[idimf] + saux2[idimf]);
        }
    }
    *niter = ndeg << 1;
    for value in somme.iter_mut().take(ndimf.max(0) as usize) {
        *value *= c2x;
    }
    0
}

/// The positive roots of the Legendre polynomial of degree `ndglgd` into `rtlegd`
/// (`mmrtptt_`); nothing below degree 2. Returns 0. `mmrtptt_(8, &mut roots)`.
pub fn mmrtptt_(ndglgd: i32, rtlegd: &mut [f64]) -> i32 {
    if ndglgd < 2 {
        return 0;
    }
    let (nsur2, nmod2) = (ndglgd / 2, ndglgd % 2);
    let ideb: i32 = nsur2 * (nsur2 - 1) / 2 + 1;
    let start: usize = (ideb + nmod2 * 465 - 1) as usize;
    mcrfill_(nsur2 << 3, &getmlgdrtl().rootab[start..], rtlegd);
    0
}

/// `mmloncv_`: the length of the canonical curve `courbe` (`(ndimax, ncoeff)`) on
/// `[tdebut, tfinal]` by a Gauss rule of `min(ncoeff/4 + 1, 10)`; `iercod` is 1 for
/// an empty curve and 2 for a failed rule.
fn mmloncv_(
    ndimax: i32,
    ndimen: i32,
    ncoeff: i32,
    courbe: &[f64],
    span: [f64; 2],
    xlongc: &mut f64,
    iercod: &mut i32,
) {
    let [tdebut, tfinal] = span;
    *iercod = 999999;
    *xlongc = 0.;
    if tdebut == tfinal {
        *iercod = 0;
        return;
    }
    if ndimen <= 0 || ncoeff <= 0 {
        *iercod = 1;
        return;
    }
    let (mut wgaus, mut uroot, mut ngaus) = ([0.; 20], [0.; 20], 0);
    let kk: i32 = (ncoeff / 4 + 1).min(10);
    mvgaus0_(kk, &mut uroot, &mut wgaus, &mut ngaus, iercod);
    if *iercod > 0 {
        *iercod = 2;
        return;
    }
    let courbe = F2cArray::new(courbe, ndimax + 1);
    let c1: f64 = (tfinal + tdebut) * 0.5;
    let c2: f64 = (tfinal - tdebut) * 0.5;
    let mut som: f64 = 0.;
    for jj in 1..=ngaus {
        let tran: f64 = c2 * uroot[(jj - 1) as usize];
        let speeds = derivative_norms(&courbe, ndimax, ndimen, ncoeff, [c1 + tran, c1 - tran]);
        som += wgaus[(jj - 1) as usize] * c2 * (speeds[0].sqrt() + speeds[1].sqrt());
    }
    *xlongc = som;
    *iercod = 0;
}

/// The squared norms of the derivative of the curve at two parameters, by the
/// native joint Horner loop of `mmloncv_`.
fn derivative_norms(
    courbe: &F2cArray<'_, f64>,
    dim1: i32,
    ndimen: i32,
    ncoeff: i32,
    x: [f64; 2],
) -> [f64; 2] {
    let (mut der1, mut der2) = (0., 0.);
    for kk in 1..=ndimen {
        let mut d1: f64 = f64::from(ncoeff - 1) * courbe[kk + ncoeff * dim1];
        let mut d2: f64 = d1;
        for ii in (2..=ncoeff - 1).rev() {
            let dd: f64 = f64::from(ii - 1) * courbe[kk + ii * dim1];
            d1 = d1 * x[0] + dd;
            d2 = d2 * x[1] + dd;
        }
        der1 += d1 * d1;
        der2 += d2 * d2;
    }
    [der1, der2]
}

/// The length of the canonical curve `courbe` (`(ndimax, ncoeff)`) on
/// `[tdebut, tfinal]`, doubling the number of sub-intervals until two successive
/// sums differ by at most `epsiln` (`mmcglc1_`): `xlongc` receives the length and
/// `erreur` the last difference. `iercod` is 1 after 13 iterations, 2 for a
/// failed rule, 3 for an empty curve; returns 0.
/// `mmcglc1_(3, 3, 4, &courbe, 0.0, 1.0, 1e-9, &mut len, &mut err, &mut ier)`.
pub fn mmcglc1_(
    ndimax: i32,
    ndimen: i32,
    ncoeff: i32,
    courbe: &[f64],
    tdebut: f64,
    tfinal: f64,
    epsiln: f64,
    xlongc: &mut f64,
    erreur: &mut f64,
    iercod: &mut i32,
) -> i32 {
    *iercod = 0;
    *xlongc = 0.;
    *erreur = 0.;
    if tdebut == tfinal {
        return 0;
    }
    if ndimen <= 0 || ncoeff <= 0 {
        *iercod = 3;
        return 0;
    }
    let length = |ndec: i32, iercod: &mut i32| -> Option<f64> {
        let pas: f64 = (tfinal - tdebut) / f64::from(ndec);
        let mut sottc: f64 = 0.;
        for kk in 1..=ndec {
            let tdeb: f64 = tdebut + f64::from(kk - 1) * pas;
            let mut som: f64 = 0.;
            mmloncv_(
                ndimax,
                ndimen,
                ncoeff,
                courbe,
                [tdeb, tdeb + pas],
                &mut som,
                iercod,
            );
            if *iercod > 0 {
                *iercod = 2;
                return None;
            }
            sottc += som;
        }
        Some(sottc)
    };
    let Some(mut oldso) = length(1, iercod) else {
        return 0;
    };
    let (mut ndec, mut iter) = (2, 2);
    loop {
        let Some(sottc) = length(ndec, iercod) else {
            return 0;
        };
        iter += 1;
        let dif: f64 = (sottc - oldso).abs();
        if dif > epsiln && iter <= 13 {
            oldso = sottc;
            ndec <<= 1;
            continue;
        }
        if dif > epsiln {
            *iercod = 1;
        }
        *xlongc = sottc;
        *erreur = dif;
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gauss_rules_integrate_polynomials() {
        let mut square = |_: i32, x: f64, out: &mut [f64], ier: &mut i32| {
            out[0] = x * x;
            *ier = 0;
        };
        let (mut a1, mut a2, mut s, mut n, mut ier) = ([0.0], [0.0], [0.0], 0, 9);
        mmgaus1_(
            1,
            &mut square,
            1,
            0.0,
            3.0,
            &mut a1,
            &mut a2,
            &mut s,
            &mut n,
            &mut ier,
        );
        assert_eq!((ier, n), (0, 4));
        assert!((s[0] - 9.0).abs() < 1e-12);
    }

    #[test]
    fn an_invalid_rule_index_stops_at_the_first_silent_evaluation() {
        let mut silent = |_: i32, _: f64, out: &mut [f64], _: &mut i32| out[0] = 1.0;
        let (mut a1, mut a2, mut s, mut n, mut ier) = ([0.0], [0.0], [5.0], 7, 0);
        mmgaus1_(
            1,
            &mut silent,
            11,
            0.0,
            1.0,
            &mut a1,
            &mut a2,
            &mut s,
            &mut n,
            &mut ier,
        );
        assert_eq!((ier, n, s[0]), (-1, 7, 0.0));
    }

    #[test]
    fn lengths_of_a_segment() {
        // The segment (0,0,0) + t (3,4,0) on [0,1] has length 5.
        let courbe: [f64; 6] = [0.0, 0.0, 0.0, 3.0, 4.0, 0.0];
        let (mut len, mut err, mut ier) = (0.0, 0.0, 9);
        mmcglc1_(
            3, 3, 2, &courbe, 0.0, 1.0, 1e-9, &mut len, &mut err, &mut ier,
        );
        assert_eq!((ier, len, err), (0, 5.0, 0.0));
        mmcglc1_(
            3, 0, 2, &courbe, 0.0, 1.0, 1e-9, &mut len, &mut err, &mut ier,
        );
        assert_eq!(ier, 3);
        let mut roots: [f64; 2] = [0.0; 2];
        mmrtptt_(2, &mut roots);
        assert_eq!(roots[0], getmlgdrtl().rootab[0]);
    }
}
