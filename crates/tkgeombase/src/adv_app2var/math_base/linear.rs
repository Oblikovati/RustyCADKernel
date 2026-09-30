//! Dense linear systems: `mmmrslwd_`, `mmcvctx_` and the file-static `mmrslw_`.
use super::mmeps1_;
use crate::adv_app2var::f2c::{F2cArray, F2cArrayMut};

/// `mmrslw_`: Gauss elimination with partial pivoting on the augmented matrix
/// `abmatr` (`(nordre + ndimen, nordre)`, one equation per column), solutions into
/// `xmatri` (`(normax, ndimen)`); `iercod` is 1 when no pivot exceeds `epspiv`.
fn mmrslw_(
    normax: i32,
    nordre: i32,
    ndimen: i32,
    epspiv: f64,
    abmatr: &mut [f64],
    xmatri: &mut [f64],
    iercod: &mut i32,
) {
    let adim1: i32 = nordre + ndimen;
    let mut ab = F2cArrayMut::new(abmatr, adim1 + 1);
    let mut x = F2cArrayMut::new(xmatri, normax + 1);
    *iercod = 0;
    for kk in 1..=nordre {
        let mut pivot: f64 = epspiv;
        let mut kpiv: i32 = 0;
        for jj in kk..=nordre {
            let akj: f64 = ab[kk + jj * adim1].abs();
            if akj > pivot {
                pivot = akj;
                kpiv = jj;
            }
        }
        if kpiv == 0 {
            *iercod = 1;
            return;
        }
        if kpiv != kk {
            for jj in kk..=nordre + ndimen {
                let akj: f64 = ab[jj + kk * adim1];
                ab[jj + kk * adim1] = ab[jj + kpiv * adim1];
                ab[jj + kpiv * adim1] = akj;
            }
        }
        pivot = -ab[kk + kk * adim1];
        for ii in kk + 1..=nordre {
            let akj: f64 = ab[kk + ii * adim1] / pivot;
            for jj in kk + 1..=nordre + ndimen {
                ab[jj + ii * adim1] += akj * ab[jj + kk * adim1];
            }
        }
    }
    for kk in (1..=nordre).rev() {
        let pivot: f64 = ab[kk + kk * adim1];
        for ii in 1..=ndimen {
            let mut akj: f64 = ab[ii + nordre + kk * adim1];
            for jj in kk + 1..=nordre {
                akj -= ab[jj + kk * adim1] * x[jj + ii * normax];
            }
            x[kk + ii * normax] = akj / pivot;
        }
    }
}

/// Solves `amat * xmat = bmat` for `ndim` right-hand sides (`mmmrslwd_`): `amat`,
/// `bmat` and `xmat` have the leading dimension `normax`, and `aaux`
/// (`(nordre + ndim) * nordre` values) is the scratch augmented matrix. `iercod` is
/// 1 when no pivot exceeds `epspiv`; returns 0.
/// `mmmrslwd_(2, 2, 1, &a, &b, 1e-10, &mut aux, &mut x, &mut ier)`.
pub fn mmmrslwd_(
    normax: i32,
    nordre: i32,
    ndim: i32,
    amat: &[f64],
    bmat: &[f64],
    epspiv: f64,
    aaux: &mut [f64],
    xmat: &mut [f64],
    iercod: &mut i32,
) -> i32 {
    let adim1: i32 = nordre + ndim;
    let amat = F2cArray::new(amat, normax + 1);
    let bmat = F2cArray::new(bmat, normax + 1);
    {
        let mut aux = F2cArrayMut::new(aaux, adim1 + 1);
        for i in 1..=nordre {
            for j in 1..=nordre {
                aux[j + i * adim1] = amat[i + j * normax];
            }
        }
        for i in 1..=nordre {
            for j in 1..=ndim {
                aux[j + nordre + i * adim1] = bmat[i + j * normax];
            }
        }
    }
    mmrslw_(normax, nordre, ndim, epspiv, aaux, xmat, iercod);
    0
}

/// The canonical curve of degree `2 * nderiv + 1` on [-1,1] whose value and
/// derivatives up to `nderiv` at both ends are `ctrtes` (`(ndimen, 2, 0:nderiv)`)
/// (`mmcvctx_`): coefficients into `crvres` (`(ncofmx, ndimen)`), through the
/// scratch `tabaux` and `xmatri`. `iercod` is the `mmrslw_` code; returns 0.
/// `mmcvctx_(3, 4, 1, &ctrtes, &mut crvres, &mut aux, &mut x, &mut ier)`.
pub fn mmcvctx_(
    ndimen: i32,
    ncofmx: i32,
    nderiv: i32,
    ctrtes: &[f64],
    crvres: &mut [f64],
    tabaux: &mut [f64],
    xmatri: &mut [f64],
    iercod: &mut i32,
) -> i32 {
    let mut eps1: f64 = 0.;
    mmeps1_(&mut eps1);
    let nordr: i32 = nderiv + 1;
    let system = EndConditions {
        ndimen,
        nordr,
        ctrtes: F2cArray::new(ctrtes, ndimen * 3 + 1),
    };
    // The even coefficients, then the odd ones, each from its own system.
    for (parity_shift, first_sign, column) in [(0, 1, -1), (1, -1, 0)] {
        system.fill(tabaux, parity_shift, first_sign);
        mmrslw_(nordr, nordr, ndimen, eps1, tabaux, xmatri, iercod);
        if *iercod > 0 {
            return 0;
        }
        let x = F2cArray::new(xmatri, nordr + 1);
        let mut res = F2cArrayMut::new(crvres, ncofmx + 1);
        for nd in 1..=ndimen {
            for ncf in 1..=nordr {
                res[(ncf << 1) + column + nd * ncofmx] = x[ncf + nd * nordr];
            }
        }
    }
    0
}

/// The two linear systems of `mmcvctx_`: derivative factors of the even (or odd)
/// monomials at 1, and the half sums (or half differences) of the end conditions.
struct EndConditions<'a> {
    ndimen: i32,
    nordr: i32,
    ctrtes: F2cArray<'a, f64>,
}

impl EndConditions<'_> {
    fn fill(&self, tabaux: &mut [f64], parity_shift: i32, first_sign: i32) {
        let (nordr, cdim1) = (self.nordr, self.ndimen);
        let tdim1: i32 = nordr + self.ndimen;
        let mut tab = F2cArrayMut::new(tabaux, tdim1 + 1);
        for ncf in 1..=nordr {
            tab[ncf + tdim1] = 1.;
        }
        for ndv in 2..=nordr {
            for ncf in 1..=nordr {
                tab[ncf + ndv * tdim1] =
                    tab[ncf + (ndv - 1) * tdim1] * f64::from((ncf << 1) - ndv + parity_shift);
            }
        }
        let mut moup1: i32 = first_sign;
        for ndv in 1..=nordr {
            for nd in 1..=self.ndimen {
                tab[nordr + nd + ndv * tdim1] = (self.ctrtes[nd + ((ndv << 1) + 2) * cdim1]
                    + f64::from(moup1) * self.ctrtes[nd + ((ndv << 1) + 1) * cdim1])
                    / 2.;
            }
            moup1 = -moup1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_two_by_two_system_is_solved_with_pivoting() {
        // [0 1; 2 3] x = [1; 5] -> x = (1, 1); amat is column major (normax 2).
        let (amat, bmat) = ([0.0, 2.0, 1.0, 3.0], [1.0, 5.0]);
        let (mut aux, mut x, mut ier) = ([0.0; 6], [0.0; 2], 9);
        mmmrslwd_(2, 2, 1, &amat, &bmat, 1e-10, &mut aux, &mut x, &mut ier);
        assert_eq!((ier, x), (0, [1.0, 1.0]));
        mmmrslwd_(2, 2, 1, &[0.0; 4], &bmat, 1e-10, &mut aux, &mut x, &mut ier);
        assert_eq!(ier, 1);
    }

    #[test]
    fn end_conditions_give_the_interpolating_line() {
        // Values -1 at t=-1 and 3 at t=1 give 1 + 2t.
        let ctrtes: [f64; 2] = [-1.0, 3.0];
        let (mut crvres, mut aux, mut x, mut ier) = ([0.0; 2], [0.0; 2], [0.0; 1], 9);
        mmcvctx_(1, 2, 0, &ctrtes, &mut crvres, &mut aux, &mut x, &mut ier);
        assert_eq!((ier, crvres), (0, [1.0, 2.0]));
    }
}
