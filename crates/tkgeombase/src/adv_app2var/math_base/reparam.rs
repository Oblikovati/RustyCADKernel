//! Reparametrisation of canonical polynomial curves: `mmarcin_`, `mmcvinv_` and the
//! file-static `mmarc41_`, `mmcvstd_`, `mvcvin2_`, `mvcvinv_`.
use super::mmveps3_;
use crate::adv_app2var::data::getmmcmcnp;
use crate::adv_app2var::f2c::{F2cArray, F2cArrayMut};
use crate::adv_app2var::sys_base::mcrfill_;

/// The Taylor expansion loop shared by `mmarc41_` and `mmarcin_`: the new curve is
/// the old one composed with `x -> a + b*x`. `old(ncf, nd)` reads a coefficient and
/// `tbaux` holds the powers of the affine map, updated with the native recurrence.
struct AffineCompose<'a> {
    ndim: i32,
    ncoeff: i32,
    a: f64,
    b: f64,
    crvnew: F2cArrayMut<'a, f64>,
    dim1: i32,
}

impl AffineCompose<'_> {
    /// Runs the loops after the constant term, reading `old(ncf, nd)`.
    fn run(&mut self, old: impl Fn(i32, i32) -> f64) {
        let mut tbaux: [f64; 61] = [0.; 61];
        tbaux[0] = self.a;
        tbaux[1] = self.b;
        for ncf in 2..=self.ncoeff - 1 {
            for ncj in 1..=ncf - 1 {
                let bid: f64 = tbaux[(ncj - 1) as usize];
                self.accumulate(ncj, bid, ncf, &old);
            }
            let bid: f64 = tbaux[(ncf - 1) as usize];
            for nd in 1..=self.ndim {
                self.crvnew[nd + ncf * self.dim1] = old(ncf, nd) * bid;
            }
            tbaux[ncf as usize] = tbaux[(ncf - 1) as usize] * self.b;
            for ncj in (2..=ncf).rev() {
                let k: usize = (ncj - 1) as usize;
                tbaux[k] = tbaux[k] * self.a + tbaux[k - 1] * self.b;
            }
            tbaux[0] *= self.a;
        }
        for ncj in 1..=self.ncoeff - 1 {
            let bid: f64 = tbaux[(ncj - 1) as usize];
            self.accumulate(ncj, bid, self.ncoeff, &old);
        }
        let last: f64 = tbaux[(self.ncoeff - 1) as usize];
        for nd in 1..=self.ndim {
            self.crvnew[nd + self.ncoeff * self.dim1] = old(self.ncoeff, nd) * last;
        }
    }

    fn accumulate(&mut self, ncj: i32, bid: f64, ncf: i32, old: &impl Fn(i32, i32) -> f64) {
        for nd in 1..=self.ndim {
            self.crvnew[nd + ncj * self.dim1] += old(ncf, nd) * bid;
        }
    }
}

/// `mmarc41_`: the curve `crvold` (`(ndimax, ncoeff)`, canonical on [0,1]) restricted
/// to `[upara0, upara1]` and brought back to [0,1]. `iercod` is 10 for more than 61
/// or fewer than 1 coefficients. The native callers also pass the same array as
/// `crvold` and `crvnew`; every coefficient is read before it is overwritten, so a
/// caller copies the input first and gets the same values.
pub(super) fn mmarc41_(
    ndimax: i32,
    ndimen: i32,
    ncoeff: i32,
    crvold: &[f64],
    upara0: f64,
    upara1: f64,
    crvnew: &mut [f64],
    iercod: &mut i32,
) {
    *iercod = 0;
    if !(1..=61).contains(&ncoeff) {
        *iercod = 10;
        return;
    }
    if ndimen == ndimax && upara0 == 0. && upara1 == 1. {
        mcrfill_((ndimax << 3) * ncoeff, crvold, crvnew);
        return;
    }
    if upara0 == 1. && upara1 == 0. {
        if ndimen == 3 && ndimax == 3 && ncoeff <= 21 {
            mvcvinv_(ncoeff, crvold, crvnew, iercod);
            return;
        }
        if ndimen == 2 && ndimax == 2 && ncoeff <= 21 {
            mvcvin2_(ncoeff, crvold, crvnew, iercod);
            return;
        }
    }
    let dim1: i32 = ndimax;
    let old = F2cArray::new(crvold, dim1 + 1);
    let mut compose = AffineCompose {
        ndim: ndimen,
        ncoeff,
        a: upara0,
        b: upara1 - upara0,
        crvnew: F2cArrayMut::new(crvnew, dim1 + 1),
        dim1,
    };
    for nd in 1..=ndimen {
        compose.crvnew[nd + dim1] = old[nd + dim1];
    }
    if ncoeff == 1 {
        return;
    }
    compose.run(|ncf, nd| old[nd + ncf * dim1]);
}

/// Converts a canonical curve given on `[u0, u1]` to [-1,1] (`mmarcin_`): `crvold`
/// is `(ncoeff, ndim)`, `crvnew` `(ndimax, ncoeff)`. `iercod` is 13 when
/// `|u1 - u0| < eps3`, 10 for more than 61 or fewer than 1 coefficients; returns 0.
/// `mmarcin_(3, 3, 4, &crvold, 0.0, 2.0, &mut crvnew, &mut iercod)`.
pub fn mmarcin_(
    ndimax: i32,
    ndim: i32,
    ncoeff: i32,
    crvold: &[f64],
    u0: f64,
    u1: f64,
    crvnew: &mut [f64],
    iercod: &mut i32,
) -> i32 {
    let mut eps3: f64 = 0.;
    mmveps3_(&mut eps3);
    if (u1 - u0).abs() < eps3 {
        *iercod = 13;
        return 0;
    }
    *iercod = 0;
    if !(1..=61).contains(&ncoeff) {
        *iercod = 10;
        return 0;
    }
    if ndim == ndimax && u0 == -1. && u1 == 1. {
        return mmcvinv_(ndim, ncoeff, ndim, crvold, crvnew);
    }
    if u0 == 0. && u1 == 1. {
        mmcvstd_(ncoeff, ndimax, ncoeff, ndim, crvold, crvnew);
        return 0;
    }
    let x0: f64 = -(u1 + u0) / (u1 - u0);
    let x1: f64 = 2. / (u1 - u0);
    let (dim1, odim1) = (ndimax, ncoeff);
    let old = F2cArray::new(crvold, odim1 + 1);
    let mut compose = AffineCompose {
        ndim,
        ncoeff,
        a: x0,
        b: x1,
        crvnew: F2cArrayMut::new(crvnew, dim1 + 1),
        dim1,
    };
    for nd in 1..=ndim {
        compose.crvnew[nd + dim1] = old[nd * odim1 + 1];
    }
    if ncoeff == 1 {
        return 0;
    }
    compose.run(|ncf, nd| old[ncf + nd * odim1]);
    0
}

/// Transposes a `(ncoef, ndim)` curve into `(ndimax, ncoef)` (`mmcvinv_`); returns 0.
/// `mmcvinv_(3, 4, 3, &curveo, &mut curve)`.
pub fn mmcvinv_(ndimax: i32, ncoef: i32, ndim: i32, curveo: &[f64], curve: &mut [f64]) -> i32 {
    let mut curve = F2cArrayMut::new(curve, ndimax + 1);
    let curveo = F2cArray::new(curveo, ncoef + 1);
    for i in 1..=ncoef {
        for nd in 1..=ndim {
            curve[nd + i * ndimax] = curveo[i + nd * ncoef];
        }
    }
    0
}

/// `mmcvstd_`: a curve canonical on [0,1], `(0:ncofmx-1, ndimen)`, rewritten on
/// [-1,1] as `(ndimax, 0:ncoeff-1)` with the binomial table `MMCMCNP`.
fn mmcvstd_(
    ncofmx: i32,
    ndimax: i32,
    ncoeff: i32,
    ndimen: i32,
    crvcan: &[f64],
    courbe: &mut [f64],
) {
    let cnp: &[f64] = getmmcmcnp().cnp;
    let cnp_at = |i: i32, j: i32| cnp[(i + j * 61) as usize];
    let (cdim1, dim1) = (ndimax, ncofmx);
    let mut courbe = F2cArrayMut::new(courbe, 1);
    let crvcan = F2cArray::new(crvcan, dim1);
    let ndeg: i32 = ncoeff - 1;
    for nd in 1..=ndimen {
        for j in 0..=ndeg {
            let mut bid: f64 = 0.;
            for i in (j..=ndeg).step_by(2) {
                bid += crvcan[i + nd * dim1] * cnp_at(i, j);
            }
            courbe[nd + j * cdim1] = bid;
            bid = 0.;
            for i in (j + 1..=ndeg).step_by(2) {
                bid += crvcan[i + nd * dim1] * cnp_at(i, j);
            }
            courbe[nd + j * cdim1] -= bid;
        }
    }
    let mut bid: f64 = 1.;
    for i in 0..=ndeg {
        for nd in 1..=ndimen {
            courbe[nd + i * cdim1] *= bid;
        }
        bid *= 2.;
    }
}

/// `mvcvin2_` and `mvcvinv_`: a 2D or 3D curve canonical on [0,1] reversed
/// (`t -> 1 - t`); `iercod` is 10 outside 1..=61 coefficients.
fn mvcvin_(ndim: i32, ncoeff: i32, crvold: &[f64], crvnew: &mut [f64], iercod: &mut i32) {
    if ncoeff < 1 || ncoeff - 1 > 60 {
        *iercod = 10;
        return;
    }
    *iercod = 0;
    let old = F2cArray::new(crvold, ndim + 1);
    let mut new = F2cArrayMut::new(crvnew, ndim + 1);
    let mut cij: [f64; 3] = [0.; 3];
    for c in 1..=ndim {
        cij[(c - 1) as usize] = old[ndim + c];
    }
    for k in 2..=ncoeff {
        for c in 1..=ndim {
            cij[(c - 1) as usize] += old[k * ndim + c];
        }
    }
    for c in 1..=ndim {
        new[ndim + c] = cij[(c - 1) as usize];
    }
    if ncoeff == 1 {
        return;
    }
    let cnp: &[f64] = getmmcmcnp().cnp;
    let mut m1jm1: i32 = 1;
    for j in 2..=ncoeff - 1 {
        m1jm1 = -m1jm1;
        for c in 1..=ndim {
            cij[(c - 1) as usize] = old[j * ndim + c];
        }
        for k in j + 1..=ncoeff {
            let bid: f64 = cnp[(k - 1 + (j - 1) * 61) as usize];
            for c in 1..=ndim {
                cij[(c - 1) as usize] += old[k * ndim + c] * bid;
            }
        }
        for c in 1..=ndim {
            new[j * ndim + c] = cij[(c - 1) as usize] * f64::from(m1jm1);
        }
    }
    for c in 1..=ndim {
        new[ncoeff * ndim + c] = -old[ncoeff * ndim + c] * f64::from(m1jm1);
    }
}

fn mvcvin2_(ncoeff: i32, crvold: &[f64], crvnew: &mut [f64], iercod: &mut i32) {
    mvcvin_(2, ncoeff, crvold, crvnew, iercod);
}

fn mvcvinv_(ncoeff: i32, crvold: &[f64], crvnew: &mut [f64], iercod: &mut i32) {
    mvcvin_(3, ncoeff, crvold, crvnew, iercod);
}
