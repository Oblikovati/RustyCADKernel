//! Degree reduction of the canonical patches: `mma2fx6_`.
use crate::adv_app2var::f2c::{F2cArray, F2cArrayMut};

/// `std::min(a, b)` for doubles: `b` when `b < a`, else `a` (NaN-sensitive like C++).
fn cpp_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// The patch grid of `mma2fx6_`: `nbupat x nbvpat` canonical patches
/// `(ncfmxu, ncfmxv, ndimen)` of `nbsesp` subspaces.
pub struct PatchGrid<'a> {
    /// `ncfmxu`: coefficient capacity in U.
    pub ncfmxu: i32,
    /// `ncfmxv`: coefficient capacity in V.
    pub ncfmxv: i32,
    /// `ndimen`: total dimension.
    pub ndimen: i32,
    /// `nbsesp`: number of subspaces.
    pub nbsesp: i32,
    /// `ndimse`: dimension of each subspace.
    pub ndimse: &'a [i32],
    /// `nbupat`: patches in U.
    pub nbupat: i32,
    /// `nbvpat`: patches in V.
    pub nbvpat: i32,
    /// `iordru`: continuity order in U.
    pub iordru: i32,
    /// `iordrv`: continuity order in V.
    pub iordrv: i32,
    /// `epsapr`: tolerance per subspace.
    pub epsapr: &'a [f64],
    /// `epsfro`: frontier (columns 1..4) and cutting (5..8) tolerances, `(nbsesp, 8)`.
    pub epsfro: &'a [f64],
    /// `patcan`: the patches.
    pub patcan: &'a [f64],
    /// `errmax`: maximum error per subspace and patch, `(nbsesp, nbupat, nbvpat)`.
    pub errmax: &'a [f64],
}

/// Lowers the degrees of each patch while the dropped constraint coefficients are
/// negligible (`mma2fx6_`): `ncoefu` and `ncoefv` (`(nbupat, nbvpat)`) shrink, never
/// below 2, while the sum of the dropped row or column stays within `1e-6` of the
/// tightest tolerance and within the patch error; only degrees up to the
/// constraint count `2*(iordr+1)` are candidates. Returns 0.
/// `mma2fx6_(&grid, &mut ncoefu, &mut ncoefv)`.
pub fn mma2fx6_(grid: &PatchGrid<'_>, ncoefu: &mut [i32], ncoefv: &mut [i32]) -> i32 {
    let n1: i32 = grid.nbupat;
    let mut ncu = F2cArrayMut::new(ncoefu, n1 + 1);
    let mut ncv = F2cArrayMut::new(ncoefv, n1 + 1);
    for jj in 1..=grid.nbvpat {
        for ii in 1..=grid.nbupat {
            let mut ncfu: i32 = ncu[ii + jj * n1];
            let mut ncfv: i32 = ncv[ii + jj * n1];
            while ncfu <= (grid.iordru + 1) << 1
                && ncfu > 2
                && grid.negligible(ii, jj, Side::U(ncfu, ncfv))
            {
                ncfu -= 1;
            }
            while ncfv <= (grid.iordrv + 1) << 1
                && ncfv > 2
                && grid.negligible(ii, jj, Side::V(ncfu, ncfv))
            {
                ncfv -= 1;
            }
            ncu[ii + jj * n1] = ncfu.max(2);
            ncv[ii + jj * n1] = ncfv.max(2);
        }
    }
    0
}

/// The candidate row: the last U coefficient `ncfu` over `ncfv` V coefficients, or
/// the last V coefficient `ncfv` over `ncfu` U coefficients.
#[derive(Clone, Copy)]
enum Side {
    U(i32, i32),
    V(i32, i32),
}

impl PatchGrid<'_> {
    /// Whether every subspace's candidate row sums below its thresholds (the loops
    /// before labels L300 and L400 of `mma2fx6_`).
    fn negligible(&self, ii: i32, jj: i32, side: Side) -> bool {
        let mut idim: i32 = 0;
        for ns in 1..=self.nbsesp {
            let bid: f64 = self.row_sum(ii, jj, idim, self.ndimse[(ns - 1) as usize], side);
            let errmax = F2cArray::new(self.errmax, self.nbsesp * (self.nbupat + 1) + 1);
            let error: f64 = errmax[ns + (ii + jj * self.nbupat) * self.nbsesp];
            if bid > self.tolerance(ns, ii, jj) * 1e-6 || bid > error {
                return false;
            }
            idim += self.ndimse[(ns - 1) as usize];
        }
        true
    }

    /// The tightest of the approximation tolerance, the cutting tolerances and, on
    /// a border patch, the frontier tolerances of subspace `ns`.
    fn tolerance(&self, ns: i32, ii: i32, jj: i32) -> f64 {
        let e1: i32 = self.nbsesp;
        let epsfro = F2cArray::new(self.epsfro, e1 * 5 + 1);
        let mut tol: f64 = self.epsapr[(ns - 1) as usize];
        for column in 9..=12 {
            tol = cpp_min(tol, epsfro[ns + e1 * column]);
        }
        if ii == 1 || ii == self.nbupat || jj == 1 || jj == self.nbvpat {
            for column in 5..=8 {
                tol = cpp_min(tol, epsfro[ns + e1 * column]);
            }
        }
        tol
    }

    /// The sum of the absolute coefficients of the candidate row of the subspace
    /// whose dimensions follow `idim`.
    fn row_sum(&self, ii: i32, jj: i32, idim: i32, ndses: i32, side: Side) -> f64 {
        let (d1, d2, d3, d4) = (self.ncfmxu, self.ncfmxv, self.ndimen, self.nbupat);
        let pat = F2cArray::new(self.patcan, d1 * (d2 * (d3 * (d4 + 1) + 1) + 1) + 1);
        let at = |u: i32, v: i32, id: i32| pat[u + (v + (id + (ii + jj * d4) * d3) * d2) * d1];
        let mut bid: f64 = 0.;
        for nd in 1..=ndses {
            let id: i32 = idim + nd;
            match side {
                Side::U(ncfu, ncfv) => (1..=ncfv).for_each(|kv| bid += at(ncfu, kv, id).abs()),
                Side::V(ncfu, ncfv) => (1..=ncfu).for_each(|ku| bid += at(ku, ncfv, id).abs()),
            }
        }
        bid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negligible_constraint_rows_are_dropped_down_to_two() {
        // One 4x4 patch of dimension 1 with order 1 (4 constraint coefficients).
        let mut patcan: Vec<f64> = vec![0.0; 16];
        patcan[0] = 1.0;
        patcan[1] = 1.0;
        patcan[4] = 1.0;
        let epsfro: Vec<f64> = vec![1.0; 8];
        let grid = PatchGrid {
            ncfmxu: 4,
            ncfmxv: 4,
            ndimen: 1,
            nbsesp: 1,
            ndimse: &[1],
            nbupat: 1,
            nbvpat: 1,
            iordru: 1,
            iordrv: 1,
            epsapr: &[1.0],
            epsfro: &epsfro,
            patcan: &patcan,
            errmax: &[1.0],
        };
        let (mut ncoefu, mut ncoefv) = ([4], [4]);
        mma2fx6_(&grid, &mut ncoefu, &mut ncoefv);
        assert_eq!((ncoefu, ncoefv), ([2], [2]));
    }
}
