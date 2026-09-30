//! The corner part of the boundary discretisation: the file-static `mma2cd1_`.
use super::constraints::CornerConstraints;
use super::discretise::{SquareGrid, SquareTables, SquareViews, sample_hermite};
use crate::adv_app2var::f2c::F2cArray;

/// The symmetric sums and differences of the two Hermite polynomials of one corner
/// order at a pair of roots `(+t, -t)`: `(so1, di1, so2, di2)`.
type RootPair = (f64, f64, f64, f64);

/// The sampled Hermite basis of one direction (`fpntb*`, `(nbpnt, 2*order+2)`).
struct HermiteSamples<'a> {
    fpnt: F2cArray<'a, f64>,
    nbpnt: i32,
}

impl HermiteSamples<'_> {
    fn at(&self, k: i32, col: i32) -> f64 {
        self.fpnt[k + col * self.nbpnt]
    }

    /// The sums and differences of the polynomials `2i-1` and `2i` at the pair
    /// `k` of roots.
    fn pair(&self, i: i32, k: i32) -> RootPair {
        let half: i32 = self.nbpnt / 2;
        let (kp, km) = ((self.nbpnt + 1) / 2 + k, half - k + 1);
        let (c1, c2) = ((i << 1) - 1, i << 1);
        let (p1, m1, p2, m2) = (
            self.at(kp, c1),
            self.at(km, c1),
            self.at(kp, c2),
            self.at(km, c2),
        );
        (p1 + m1, p1 - m1, p2 + m2, p2 - m2)
    }

    /// The two polynomials of order `i` at the middle root of an odd count.
    fn middle(&self, i: i32) -> (f64, f64) {
        let mid: i32 = self.nbpnt / 2 + 1;
        (self.at(mid, (i << 1) - 1), self.at(mid, i << 1))
    }
}

/// The four corner values of one (dimension, order) pair: `bid1..bid4`.
type CornerValues = [f64; 4];

/// `c1 * s1 * t1` style products of the native update, subtracted in the native
/// order: `x - b1*u1*v1 - b2*u2*v1 - b1'...`.
fn corner_update(x: f64, b: &CornerValues, u: (f64, f64), v: (f64, f64)) -> f64 {
    x - b[0] * u.0 * v.0 - b[1] * u.1 * v.0 - b[2] * u.0 * v.1 - b[3] * u.1 * v.1
}

/// `mma2cd1_`: subtracts the tensor Hermite interpolation of the corner constraints,
/// sampled at the roots, from the four tables. `fpntb` receives the sampled U and
/// V bases, computed from `hermt` (the `mma1her_` bases in U and V).
pub(super) fn mma2cd1_(
    grid: &SquareGrid<'_>,
    corners: &CornerConstraints<'_>,
    fpntb: [&mut [f64]; 2],
    hermt: [&[f64]; 2],
    tables: &mut SquareTables<'_>,
) {
    let (nbpntu, nbpntv) = (grid.nbpntu, grid.nbpntv);
    let [fpntbu, fpntbv] = fpntb;
    sample_hermite(grid.iordru, nbpntu, grid.urootl, hermt[0], fpntbu);
    sample_hermite(grid.iordrv, nbpntv, grid.vrootl, hermt[1], fpntbv);
    let fu = HermiteSamples {
        fpnt: F2cArray::new(fpntbu, nbpntu + 1),
        nbpnt: nbpntu,
    };
    let fv = HermiteSamples {
        fpnt: F2cArray::new(fpntbv, nbpntv + 1),
        nbpnt: nbpntv,
    };
    let (cd1, cd2) = (grid.ndimen, grid.iordru + 2);
    let offset: i32 = cd1 * (cd2 + 1) + 1;
    let c = [
        corners.contr1,
        corners.contr2,
        corners.contr3,
        corners.contr4,
    ]
    .map(|t| F2cArray::new(t, offset));
    let mut t = SquareViews::new(tables, nbpntu, nbpntv);
    for nd in 1..=grid.ndimen {
        for jj in 1..=grid.iordrv + 1 {
            for ii in 1..=grid.iordru + 1 {
                let k: i32 = nd + (ii + jj * cd2) * cd1;
                let bid: CornerValues = [c[0][k], c[1][k], c[2][k], c[3][k]];
                corner_order(&mut t, [&fu, &fv], &bid, [ii, jj, nd]);
            }
        }
    }
}

/// The body of the `(nd, jj, ii)` loop of `mma2cd1_`: the paired roots, then the
/// middle U root, the middle V root and the centre of odd counts.
fn corner_order(
    t: &mut SquareViews<'_>,
    f: [&HermiteSamples<'_>; 2],
    bid: &CornerValues,
    index: [i32; 3],
) {
    let [fu, fv] = f;
    let [ii, jj, nd] = index;
    let (nuroo, nvroo) = (fu.nbpnt / 2, fv.nbpnt / 2);
    for kk in 1..=nvroo {
        let (sov1, div1, sov2, div2) = fv.pair(jj, kk);
        for ll in 1..=nuroo {
            let (sou1, diu1, sou2, diu2) = fu.pair(ii, ll);
            let (f, h) = (t.f(ll, kk, nd), t.h(ll, kk, nd));
            t.soso[f] = corner_update(t.soso[f], bid, (sou1, sou2), (sov1, sov2));
            t.sodi[h] = corner_update(t.sodi[h], bid, (sou1, sou2), (div1, div2));
            t.diso[h] = corner_update(t.diso[h], bid, (diu1, diu2), (sov1, sov2));
            t.didi[f] = corner_update(t.didi[f], bid, (diu1, diu2), (div1, div2));
        }
    }
    if fu.nbpnt % 2 == 1 {
        let (sou1, sou2) = fu.middle(ii);
        for kk in 1..=nvroo {
            let (sov1, div1, sov2, div2) = fv.pair(jj, kk);
            let f: i32 = t.f(0, kk, nd);
            t.soso[f] = corner_update(t.soso[f], bid, (sou1, sou2), (sov1, sov2));
            t.didi[f] = corner_update(t.didi[f], bid, (sou1, sou2), (div1, div2));
        }
    }
    if fv.nbpnt % 2 == 1 {
        let (sov1, sov2) = fv.middle(jj);
        for ll in 1..=nuroo {
            let (sou1, diu1, sou2, diu2) = fu.pair(ii, ll);
            let f: i32 = t.f(ll, 0, nd);
            t.soso[f] = corner_update(t.soso[f], bid, (sou1, sou2), (sov1, sov2));
            t.didi[f] = corner_update(t.didi[f], bid, (diu1, diu2), (sov1, sov2));
        }
    }
    if fu.nbpnt % 2 == 1 && fv.nbpnt % 2 == 1 {
        let ((sou1, sou2), (sov1, sov2)) = (fu.middle(ii), fv.middle(jj));
        let f: i32 = t.f(0, 0, nd);
        t.soso[f] = corner_update(t.soso[f], bid, (sou1, sou2), (sov1, sov2));
        // The centre has no U or V pair; the native code subtracts the same term.
        t.didi[f] = corner_update(t.didi[f], bid, (sou1, sou2), (sov1, sov2));
    }
}
