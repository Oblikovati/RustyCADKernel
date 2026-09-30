//! Hermite interpolation bases: the file-static `mmherm0_` and `mmherm1_` with their
//! cache, the common `MMCMHER`.
//!
//! Natively `MMCMHER` is a process-global memo: `mmherm0_` fills it for one interval
//! and `mmherm1_` recomputes it only when the interval changes. The values depend
//! only on the interval, so the port computes them into a [`Mmcmher`] per call of
//! `mmherm1_`; the results and error codes are the ones the cached path returns.
use super::linear::mmmrslwd_;
use crate::adv_app2var::sys_base::{msrfill_, mvriraz_};

/// The common `MMCMHER`: the interval, its check value and the 576 Hermite
/// coefficients `(6, 3, 2, -1:2, -1:2)` (degree, derivative, side, orders).
pub(super) struct Mmcmher {
    tdebut: f64,
    tfinal: f64,
    verifi: f64,
    cmherm: [f64; 576],
}

impl Default for Mmcmher {
    fn default() -> Self {
        Self {
            tdebut: 0.,
            tfinal: 0.,
            verifi: 0.,
            cmherm: [0.; 576],
        }
    }
}

/// The interval check value of `MMCMHER`: `(|t0| + |t1|) * 16111959`.
fn check_value(debfin: [f64; 2]) -> f64 {
    let mut d1: f64 = debfin[0].abs() + debfin[1].abs();
    d1 *= 16111959.;
    d1
}

impl Mmcmher {
    /// `mmherm0_`: fills the cache for `debfin`. `iercod` is 1 for an interval that
    /// is too large, too small or too narrow, -1 when the cache already holds it.
    pub(super) fn mmherm0_(&mut self, debfin: [f64; 2], iercod: &mut i32) {
        if !interval_is_valid(debfin) {
            *iercod = 1;
            return;
        }
        *iercod = 0;
        let d1: f64 = check_value(debfin);
        if debfin[0] == self.tdebut && debfin[1] == self.tfinal && d1 == self.verifi {
            *iercod = -1;
            return;
        }
        mvriraz_(576, &mut self.cmherm);
        for iord1 in -1..=2 {
            for iord2 in -1..=2 {
                if !self.fill_orders([iord1, iord2], debfin) {
                    *iercod = 1;
                    return;
                }
            }
        }
        self.tdebut = debfin[0];
        self.tfinal = debfin[1];
        self.verifi = check_value(debfin);
    }

    /// One `(iord1, iord2)` pair of `mmherm0_`: the constraint matrix of the Hermite
    /// basis, inverted by `mmmrslwd_`; false when the inversion fails.
    fn fill_orders(&mut self, iord: [i32; 2], debfin: [f64; 2]) -> bool {
        let iof: [i32; 2] = [0, iord[0] + 1];
        let ncf: i32 = iord[0] + iord[1] + 2;
        let mut mat: [f64; 36] = [0.; 36];
        for cot in 1..=2 {
            constraint_rows(
                &mut mat,
                iord[(cot - 1) as usize],
                iof[(cot - 1) as usize],
                ncf,
                debfin[(cot - 1) as usize],
            );
        }
        let mut amat: [f64; 36] = [0.; 36];
        if ncf >= 1 {
            let mut miden: [f64; 36] = [0.; 36];
            for ii in 1..=6 {
                miden[(ii + ii * 6 - 7) as usize] = 1.;
            }
            let (mut abid, mut ier) = ([0.; 72], 0);
            mmmrslwd_(
                6, ncf, ncf, &mat, &miden, 1e-10, &mut abid, &mut amat, &mut ier,
            );
            if ier > 0 {
                return false;
            }
        }
        let (iord1, iord2) = (iord[0], iord[1]);
        for cot in 1..=2 {
            for pp in 1..=iord[(cot - 1) as usize] + 1 {
                for ii in 1..=ncf {
                    // The native shifts `(iord1 + (iord2 << 2)) << 1` may be negative.
                    let position: i32 = ii + (pp + (cot + (iord1 + iord2 * 4) * 2) * 3) * 6 + 155;
                    let source: i32 = ii + (pp + iof[(cot - 1) as usize]) * 6 - 7;
                    self.cmherm[position as usize] = amat[source as usize];
                }
            }
        }
        true
    }
}

/// The interval checks of `mmherm0_`, with its single-precision thresholds.
#[allow(clippy::neg_cmp_op_on_partial_ord)] // The native rejection test, negated.
fn interval_is_valid(debfin: [f64; 2]) -> bool {
    let d1: f64 = debfin[0].abs();
    if d1 > f64::from(100.0_f32) {
        return false;
    }
    let mut d2: f64 = debfin[1].abs();
    if d2 > f64::from(100.0_f32) {
        return false;
    }
    d2 += d1;
    if d2 < f64::from(0.01_f32) {
        return false;
    }
    // Written as the native rejection test, so a NaN ratio is accepted as natively.
    let ratio: f64 = (debfin[1] - debfin[0]).abs() / d2;
    !(ratio < f64::from(0.01_f32))
}

/// The rows of one side of the Hermite constraint matrix: row `pp + iof` holds the
/// `pp-1`-th derivative of the monomials at `t` (the `cot` loop of `mmherm0_`).
fn constraint_rows(mat: &mut [f64; 36], order: i32, iof: i32, ncf: i32, t: f64) {
    let mut cof: [f64; 6] = [0.; 6];
    if order > -1 {
        for jj in 1..=ncf {
            cof[(jj - 1) as usize] = 1.;
        }
    }
    for pp in 1..=order + 1 {
        let ii: i32 = pp + iof;
        let mut prod: f64 = 1.;
        for jj in 1..=pp - 1 {
            mat[(ii + jj * 6 - 7) as usize] = f64::from(0.0_f32);
        }
        for jj in pp..=ncf {
            let k: usize = (jj - 1) as usize;
            mat[(ii + jj * 6 - 7) as usize] = cof[k] * prod;
            cof[k] *= f64::from(jj - pp);
            prod *= t;
        }
    }
}

/// `mmherm1_`: the Hermite basis `(6, 3, 2)` for the continuity orders `iordre` at
/// both ends of `debfin` into `hermit` (36 values). `iercod` is 2 unless
/// `ordrmx == 2` and both orders lie in -1..=2, 1 for an invalid interval.
pub(super) fn mmherm1_(
    debfin: [f64; 2],
    ordrmx: i32,
    iordre: [i32; 2],
    hermit: &mut [f64],
    iercod: &mut i32,
) {
    *iercod = 0;
    if ordrmx != 2 || iordre.iter().any(|order| *order < -1 || *order > ordrmx) {
        *iercod = 2;
        return;
    }
    let mut cache = Mmcmher::default();
    let d1: f64 = check_value(debfin);
    if debfin[0] != cache.tdebut || debfin[1] != cache.tfinal || d1 != cache.verifi {
        *iercod = -1;
        cache.mmherm0_(debfin, iercod);
        if *iercod > 0 {
            *iercod = 1;
            return;
        }
    }
    let start: i32 = ((((iordre[0] + iordre[1] * 4) * 2) + 1) * 3 + 1) * 6 + 156;
    msrfill_(36, &cache.cmherm[start as usize..], hermit);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c0_hermite_basis_on_minus_one_one() {
        let (mut hermit, mut ier) = ([0.0; 36], 9);
        mmherm1_([-1.0, 1.0], 2, [0, 0], &mut hermit, &mut ier);
        assert_eq!(ier, 0);
        // (1 - t)/2 and (1 + t)/2.
        assert_eq!(&hermit[..2], &[0.5, -0.5]);
        assert_eq!(&hermit[18..20], &[0.5, 0.5]);
    }

    #[test]
    fn invalid_orders_and_intervals_are_refused() {
        let (mut hermit, mut ier) = ([0.0; 36], 0);
        mmherm1_([-1.0, 1.0], 1, [0, 0], &mut hermit, &mut ier);
        assert_eq!(ier, 2);
        mmherm1_([-1.0, 1.0], 2, [3, 0], &mut hermit, &mut ier);
        assert_eq!(ier, 2);
        mmherm1_([0.0, 200.0], 2, [0, 0], &mut hermit, &mut ier);
        assert_eq!(ier, 1);
        let mut cache = Mmcmher::default();
        cache.mmherm0_([-1.0, 1.0], &mut ier);
        assert_eq!(ier, 0);
        cache.mmherm0_([-1.0, 1.0], &mut ier);
        assert_eq!(ier, -1);
    }
}
