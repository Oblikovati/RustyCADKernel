//! Constrained Jacobi bases: `mmhjcan_`, `mmpobas_` and the file-static `mmpojac_`.
use super::compress::mmapcmp_;
use super::derivatives::mmdrvcb_;
use super::evaluation::mmpocrb_;
use super::hermite::mmherm1_;
use super::jacobi::mmjacan_;
use super::{pow__di, pow__ii};
use crate::adv_app2var::data::getminombr;
use crate::adv_app2var::f2c::{F2cArray, F2cArrayMut};
use crate::adv_app2var::sys_base::mvriraz_;

/// Converts `ncourb` Hermite-Jacobi curves of continuity `orcont` to the canonical
/// base on [-1,1] (`mmhjcan_`): `tcbold` and `tcbnew` are `(ndimen, ncflim, ncourb)`,
/// `ncftab` gives the coefficient count of each curve and `tdecop` the `ncourb + 1`
/// cut parameters. `iercod` is 1 for `orcont > 2` or more than 21 coefficients, 2
/// when the Hermite basis fails; returns 0.
/// `mmhjcan_(3, 1, &[6], 1, 6, &old, &[0.0, 1.0], &mut new, &mut ier)`.
pub fn mmhjcan_(
    ndimen: i32,
    ncourb: i32,
    ncftab: &[i32],
    orcont: i32,
    ncflim: i32,
    tcbold: &[f64],
    tdecop: &[f64],
    tcbnew: &mut [f64],
    iercod: &mut i32,
) -> i32 {
    *iercod = 0;
    if orcont > 2 || ncflim > 21 {
        *iercod = 1;
        return 0;
    }
    let mut hermit: [f64; 36] = [0.; 36];
    let mut ier: i32 = 0;
    mmherm1_([-1., 1.], 2, [orcont, orcont], &mut hermit, &mut ier);
    if ier > 0 {
        *iercod = 2;
        return 0;
    }
    let (dim1, dim2) = (ndimen, ncflim);
    let old = F2cArray::new(tcbold, dim1 * (dim2 + 1) + 1);
    let mut new = F2cArrayMut::new(tcbnew, dim1 * (dim2 + 1) + 1);
    let ncftab = F2cArray::new(ncftab, 1);
    let aux1: i32 = orcont + 1;
    let aux2: i32 = aux1 << 1;
    for e in 1..=ncourb {
        let ctenor: f64 = (tdecop[e as usize] - tdecop[(e - 1) as usize]) / 2.;
        let ncoeff: i32 = ncftab[e];
        if ncoeff > 21 {
            *iercod = 1;
            return 0;
        }
        for d in 1..=ndimen {
            let mut taux1: [f64; 21] = [0.; 21];
            mvriraz_(ncoeff, &mut taux1);
            for k in 1..=aux2 {
                for i in 1..=aux1 {
                    let mfact: f64 = pow__di(ctenor, i - 1);
                    taux1[(k - 1) as usize] += (old[d + (i + e * dim2) * dim1]
                        * hermit[(k + (i + 2) * 6 - 19) as usize]
                        + old[d + (i + aux1 + e * dim2) * dim1]
                            * hermit[(k + (i + 5) * 6 - 19) as usize])
                        * mfact;
                }
            }
            for i in aux2 + 1..=ncoeff {
                taux1[(i - 1) as usize] = old[d + (i + e * dim2) * dim1];
            }
            let mut tjacap: [f64; 21] = [0.; 21];
            mmapcmp_(getminombr()[1], 21, ncoeff, &taux1, &mut tjacap);
            mmjacan_(orcont, ncoeff - 1, &tjacap, &mut taux1);
            for i in 1..=ncoeff {
                new[d + (i + e * dim2) * dim1] = taux1[(i - 1) as usize];
            }
        }
    }
    0
}

/// The values at `tparam` of the `ncoeff` basis functions of continuity order
/// `iordre` (Hermite polynomials then weighted Jacobi polynomials) and of their
/// derivatives up to `nderiv` into `valbas` (`(ncoeff, 0:nderiv)`) (`mmpobas_`).
/// `iercod` is 1 for `nderiv > 3`, `ncoeff > 20` or `iordre > 2`, 2 when a
/// sub-computation fails; returns 0. `mmpobas_(0.5, 1, 8, 1, &mut valbas, &mut ier)`.
pub fn mmpobas_(
    tparam: f64,
    iordre: i32,
    ncoeff: i32,
    nderiv: i32,
    valbas: &mut [f64],
    iercod: &mut i32,
) -> i32 {
    *iercod = 0;
    if nderiv > 3 || ncoeff > 20 || iordre > 2 {
        *iercod = 1;
        return 0;
    }
    let iorjac: i32 = (iordre + 1) << 1;
    let mut herm: [f64; 24] = [0.; 24];
    if !hermite_values(tparam, iordre, nderiv, &mut herm) {
        *iercod = 2;
        return 0;
    }
    let mut vjacc: [f64; 80] = [0.; 80];
    let mut ier: i32 = 0;
    mmpojac_(
        tparam,
        iorjac,
        ncoeff - iorjac,
        nderiv,
        &mut vjacc,
        &mut ier,
    );
    if ier > 0 {
        *iercod = 2;
        return 0;
    }
    let mut wval: [f64; 4] = [0.; 4];
    if !weight_values(tparam, iordre, nderiv, &mut wval) {
        *iercod = 2;
        return 0;
    }
    let bases = BasisTables {
        herm,
        vjac: F2cArray::new(&vjacc, 1),
        wval,
        iorjac,
        iunit: nderiv + 1,
    };
    bases.write(ncoeff, nderiv, &mut F2cArrayMut::new(valbas, 1));
    0
}

/// The Hermite part of `mmpobas_`: the basis on [-1,1] from `mmherm1_`, evaluated
/// with its derivatives at `tparam` into `herm`; false when a step fails.
fn hermite_values(tparam: f64, iordre: i32, nderiv: i32, herm: &mut [f64; 24]) -> bool {
    let mut hermit: [f64; 36] = [0.; 36];
    let mut ier: i32 = 0;
    if iordre >= 0 {
        mmherm1_([-1., 1.], 2, [iordre, iordre], &mut hermit, &mut ier);
        if ier > 0 {
            return false;
        }
    }
    let iorjac: i32 = (iordre + 1) << 1;
    let iunit: i32 = nderiv + 1;
    let khe: i32 = (iordre + 1) * iunit;
    let mut jj: i32 = 1;
    for ii in 0..=iordre {
        let left: &[f64] = &hermit[((ii + 3) * 6 - 18) as usize..];
        let right: &[f64] = &hermit[((ii + 6) * 6 - 18) as usize..];
        if nderiv > 0 {
            mmdrvcb_(
                nderiv,
                1,
                iorjac,
                left,
                tparam,
                &mut herm[(jj - 1) as usize..],
                &mut ier,
            );
            if ier > 0 {
                return false;
            }
            mmdrvcb_(
                nderiv,
                1,
                iorjac,
                right,
                tparam,
                &mut herm[(jj + khe - 1) as usize..],
                &mut ier,
            );
            if ier > 0 {
                return false;
            }
        } else {
            mmpocrb_(1, iorjac, left, 1, tparam, &mut herm[(jj - 1) as usize..]);
            mmpocrb_(
                1,
                iorjac,
                right,
                1,
                tparam,
                &mut herm[(jj + khe - 1) as usize..],
            );
        }
        jj += iunit;
    }
    true
}

/// The weight `(1 - t^2)^(iordre + 1)` of `mmpobas_` and its derivatives at
/// `tparam` into `wval`; false when `mmdrvcb_` fails.
fn weight_values(tparam: f64, iordre: i32, nderiv: i32, wval: &mut [f64; 4]) -> bool {
    let iorjac: i32 = (iordre + 1) << 1;
    let nwcof: i32 = (iorjac + 1).max(1);
    let mut wpoly: [f64; 7] = [0.; 7];
    mvriraz_(nwcof, &mut wpoly);
    wpoly[0] = 1.;
    match iordre {
        2 => {
            wpoly[2] = -3.;
            wpoly[4] = 3.;
            wpoly[6] = -1.;
        }
        1 => {
            wpoly[2] = -2.;
            wpoly[4] = 1.;
        }
        0 => wpoly[2] = -1.,
        _ => {}
    }
    let mut ier: i32 = 0;
    mmdrvcb_(nderiv, 1, nwcof, &wpoly, tparam, wval, &mut ier);
    ier <= 0
}

/// The evaluated pieces `mmpobas_` combines: Hermite values (stride `iunit`), the
/// Jacobi values `(kk1, 0:nderiv)` and the weight derivatives.
struct BasisTables<'a> {
    herm: [f64; 24],
    vjac: F2cArray<'a, f64>,
    wval: [f64; 4],
    iorjac: i32,
    iunit: i32,
}

impl BasisTables<'_> {
    /// The final loops of `mmpobas_`: the Hermite values, then the Leibniz products of
    /// the weight and Jacobi derivatives, one derivative order after the other.
    fn write(&self, ncoeff: i32, nderiv: i32, valbas: &mut F2cArrayMut<'_, f64>) {
        let dim1: i32 = ncoeff;
        let kk1: i32 = ncoeff - self.iorjac;
        let (kk2, kk3) = (kk1 << 1, kk1 * 3);
        let (w, v) = (&self.wval, &self.vjac);
        for order in 0..=nderiv.min(3) {
            let mut jj: i32 = order + 1;
            for ii in 1..=self.iorjac {
                valbas[ii + order * dim1] = self.herm[(jj - 1) as usize];
                jj += self.iunit;
            }
            for ii in 1..=kk1 {
                valbas[ii + self.iorjac + order * dim1] = match order {
                    0 => w[0] * v[ii],
                    1 => w[0] * v[ii + kk1] + w[1] * v[ii],
                    2 => w[0] * v[ii + kk2] + w[1] * 2. * v[ii + kk1] + w[2] * v[ii],
                    _ => {
                        w[0] * v[ii + kk3]
                            + w[1] * 3. * v[ii + kk2]
                            + w[2] * 3. * v[ii + kk1]
                            + w[3] * v[ii]
                    }
                };
            }
        }
    }
}

/// `mmpojac_`: the normalised Jacobi polynomials of order `iordre` and degrees
/// `0..ncoeff` at `tparam`, with derivatives up to `nderiv`, into `valjac`
/// (`(ncoeff, 0:nderiv)`); `iercod` is 1 for `nderiv > 3` or `ncoeff > 100`.
fn mmpojac_(
    tparam: f64,
    iordre: i32,
    ncoeff: i32,
    nderiv: i32,
    valjac: &mut [f64],
    iercod: &mut i32,
) {
    *iercod = 0;
    if nderiv > 3 || ncoeff > 100 {
        *iercod = 1;
        return;
    }
    let dim1: i32 = ncoeff;
    let mut tnorm: [f64; 100] = [0.; 100];
    for ii in 1..=ncoeff {
        let kk1: i32 = ii - 1;
        let mut aux2: f64 = 1.;
        for jj in 1..=iordre {
            aux2 = aux2 * f64::from(kk1 + iordre + jj) / f64::from(kk1 + jj);
        }
        let scale: f64 = f64::from(kk1) * 2. + f64::from(iordre << 1) + 1.;
        tnorm[(ii - 1) as usize] = (aux2 * scale / f64::from(pow__ii(2, (iordre << 1) + 1))).sqrt();
    }
    let mut v = F2cArrayMut::new(valjac, 1);
    jacobi_start(&mut v, tparam, iordre, dim1, nderiv);
    for ii in 3..=ncoeff {
        jacobi_step(&mut v, tparam, iordre, dim1, nderiv, ii);
    }
    for ii in 1..=ncoeff {
        for jj in 0..=nderiv {
            v[ii + jj * dim1] *= tnorm[(ii - 1) as usize];
        }
    }
}

/// Degrees 0 and 1 of `mmpojac_` and their derivatives.
fn jacobi_start(v: &mut F2cArrayMut<'_, f64>, tparam: f64, iordre: i32, dim1: i32, nderiv: i32) {
    v[1] = 1.;
    let aux1: f64 = f64::from(iordre + 1);
    v[2] = aux1 * tparam;
    if nderiv >= 1 {
        v[dim1 + 1] = 0.;
        v[dim1 + 2] = aux1;
        if nderiv >= 2 {
            v[(dim1 << 1) + 1] = 0.;
            v[(dim1 << 1) + 2] = 0.;
            if nderiv >= 3 {
                v[dim1 * 3 + 1] = 0.;
                v[dim1 * 3 + 2] = 0.;
            }
        }
    }
}

/// Degree `ii - 1` of `mmpojac_` from the two previous ones (the Jacobi recurrence).
fn jacobi_step(
    v: &mut F2cArrayMut<'_, f64>,
    tparam: f64,
    iordre: i32,
    dim1: i32,
    nderiv: i32,
    ii: i32,
) {
    let (kk1, kk2) = (ii - 1, ii - 2);
    let aux1: f64 = f64::from(iordre + kk2);
    let aux2: f64 = aux1 * 2.;
    let cofa: f64 = aux2 * (aux2 + 1.) * (aux2 + 2.);
    let cofb: f64 = (aux2 + 2.) * -2. * aux1 * aux1;
    let mut denom: f64 = f64::from(kk1) * 2. * f64::from(kk2 + (iordre << 1) + 1) * aux2;
    denom = 1. / denom;
    v[ii] = (cofa * tparam * v[kk1] + cofb * v[kk2]) * denom;
    if nderiv < 1 {
        return;
    }
    v[ii + dim1] = (cofa * tparam * v[kk1 + dim1] + cofa * v[kk1] + cofb * v[kk2 + dim1]) * denom;
    let (d2, d3) = (dim1 << 1, dim1 * 3);
    if nderiv >= 2 {
        v[ii + d2] =
            (cofa * tparam * v[kk1 + d2] + cofa * 2. * v[kk1 + dim1] + cofb * v[kk2 + d2]) * denom;
    }
    if nderiv >= 3 {
        v[ii + d3] =
            (cofa * tparam * v[kk1 + d3] + cofa * 3. * v[kk1 + d2] + cofb * v[kk2 + d3]) * denom;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c0_basis_starts_with_the_two_hermite_functions() {
        let (mut valbas, mut ier) = ([0.0; 12], 9);
        mmpobas_(0.5, 0, 4, 2, &mut valbas, &mut ier);
        assert_eq!(ier, 0);
        // (1 - t)/2 and (1 + t)/2 at 0.5, then their derivatives -1/2 and 1/2.
        assert_eq!(&valbas[..2], &[0.25, 0.75]);
        assert_eq!(&valbas[4..6], &[-0.5, 0.5]);
        assert_eq!(&valbas[8..10], &[0.0, 0.0]);
        mmpobas_(0.5, 3, 4, 2, &mut valbas, &mut ier);
        assert_eq!(ier, 1);
    }

    #[test]
    fn hermite_jacobi_conversion_refuses_long_curves() {
        let (old, mut new, mut ier) = ([0.0; 66], [0.0; 66], 0);
        mmhjcan_(3, 1, &[22], 0, 21, &old, &[0.0, 1.0], &mut new, &mut ier);
        assert_eq!(ier, 1);
        mmhjcan_(3, 1, &[4], 0, 21, &old, &[0.0, 1.0], &mut new, &mut ier);
        assert_eq!(ier, 0);
    }
}
