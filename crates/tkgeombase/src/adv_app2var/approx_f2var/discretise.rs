//! Discretisation of the boundary interpolation of a patch: `mma2cdi_` and the
//! file-static `mma2cd2_`, `mma2cd3_` (the U- and V-iso parts); the corner part
//! `mma2cd1_` is in the `corners` module.
//!
//! The square is sampled at the Legendre roots, folded by symmetry into the four
//! tables of [`SquareTables`]: sums and differences of the values at `+t` and `-t`
//! in U (`so`/`di`) and V (`so`/`di`).
use super::bases::mma1her_;
use super::constraints::CornerConstraints;
use super::corners::mma2cd1_;
use crate::adv_app2var::f2c::{F2cArray, F2cArrayMut};
use crate::adv_app2var::math_base::mmmpocur_;
use crate::adv_app2var::sys_base::mvriraz_;

/// The four symmetric tables of a square discretisation: `sosotb` and `diditb`
/// are `(0:nbpntu/2, 0:nbpntv/2, ndimen)`, `soditb` and `disotb`
/// `(nbpntu/2, nbpntv/2, ndimen)`.
pub struct SquareTables<'a> {
    /// `sosotb`: sums in U and in V.
    pub sosotb: &'a mut [f64],
    /// `disotb`: differences in U, sums in V.
    pub disotb: &'a mut [f64],
    /// `soditb`: sums in U, differences in V.
    pub soditb: &'a mut [f64],
    /// `diditb`: differences in U and in V.
    pub diditb: &'a mut [f64],
}

/// The shifted views of [`SquareTables`] with their dimensions.
pub(super) struct SquareViews<'a> {
    pub(super) soso: F2cArrayMut<'a, f64>,
    pub(super) diso: F2cArrayMut<'a, f64>,
    pub(super) sodi: F2cArrayMut<'a, f64>,
    pub(super) didi: F2cArrayMut<'a, f64>,
    /// `(nbpntu/2 + 1, nbpntv/2 + 1)`: dimensions of `sosotb` and `diditb`.
    pub(super) full: (i32, i32),
    /// `(nbpntu/2, nbpntv/2)`: dimensions of `soditb` and `disotb`.
    pub(super) half: (i32, i32),
}

impl<'a> SquareViews<'a> {
    pub(super) fn new(tables: &'a mut SquareTables<'_>, nbpntu: i32, nbpntv: i32) -> Self {
        let full: (i32, i32) = (nbpntu / 2 + 1, nbpntv / 2 + 1);
        let half: (i32, i32) = (nbpntu / 2, nbpntv / 2);
        let half_offset: i32 = half.0 * (half.1 + 1) + 1;
        SquareViews {
            soso: F2cArrayMut::new(tables.sosotb, full.0 * full.1),
            diso: F2cArrayMut::new(tables.disotb, half_offset),
            sodi: F2cArrayMut::new(tables.soditb, half_offset),
            didi: F2cArrayMut::new(tables.diditb, full.0 * full.1),
            full,
            half,
        }
    }

    /// The index of `(i, j, nd)` in `sosotb`/`diditb`.
    pub(super) fn f(&self, i: i32, j: i32, nd: i32) -> i32 {
        i + (j + nd * self.full.1) * self.full.0
    }

    /// The index of `(i, j, nd)` in `soditb`/`disotb`.
    pub(super) fn h(&self, i: i32, j: i32, nd: i32) -> i32 {
        i + (j + nd * self.half.1) * self.half.0
    }
}

/// The sampled iso constraints of one direction: `sotb*` and `ditb*` hold the sums
/// and differences of the two isos (`(0:nbpnt/2, ndimen, order+1)`).
pub struct IsoTables<'a> {
    /// `sotbu1`/`sotbv1`: sums on the first iso.
    pub sotb1: &'a [f64],
    /// `sotbu2`/`sotbv2`: sums on the second iso.
    pub sotb2: &'a [f64],
    /// `ditbu1`/`ditbv1`: differences on the first iso.
    pub ditb1: &'a [f64],
    /// `ditbu2`/`ditbv2`: differences on the second iso.
    pub ditb2: &'a [f64],
}

/// The discretisation data of `mma2cdi_`: the Legendre roots and continuity orders.
pub struct SquareGrid<'a> {
    /// `ndimen`: dimension of the function.
    pub ndimen: i32,
    /// `nbpntu`: number of U roots.
    pub nbpntu: i32,
    /// `urootl`: the U roots.
    pub urootl: &'a [f64],
    /// `nbpntv`: number of V roots.
    pub nbpntv: i32,
    /// `vrootl`: the V roots.
    pub vrootl: &'a [f64],
    /// `iordru`: continuity order in U (-1..=2).
    pub iordru: i32,
    /// `iordrv`: continuity order in V (-1..=2).
    pub iordrv: i32,
}

/// Discretises the interpolation of the corner and iso constraints on the square
/// (`mma2cdi_`): the tables receive minus the sampled Hermite interpolation, ready
/// to be added to the samples of the function. `iercod` is 1 for an order outside
/// -1..=2; returns 0. `mma2cdi_(&grid, &corners, &isos_u, &isos_v, &mut tables, &mut ier)`.
pub fn mma2cdi_(
    grid: &SquareGrid<'_>,
    corners: &CornerConstraints<'_>,
    isos_u: &IsoTables<'_>,
    isos_v: &IsoTables<'_>,
    tables: &mut SquareTables<'_>,
    iercod: &mut i32,
) -> i32 {
    *iercod = 0;
    let (iordru, iordrv) = (grid.iordru, grid.iordrv);
    if !(-1..=2).contains(&iordru) || !(-1..=2).contains(&iordrv) {
        *iercod = 1;
        return 0;
    }
    let (nbpntu, nbpntv) = (grid.nbpntu, grid.nbpntv);
    let ilong: i32 = (nbpntu / 2 + 1) * (nbpntv / 2 + 1) * grid.ndimen;
    mvriraz_(ilong, tables.sosotb);
    mvriraz_(ilong, tables.diditb);
    let ilong: i32 = nbpntu / 2 * (nbpntv / 2) * grid.ndimen;
    mvriraz_(ilong, tables.soditb);
    mvriraz_(ilong, tables.disotb);
    if iordru == -1 && iordrv == -1 {
        return 0;
    }
    let sizes: [i32; 4] = [
        ((iordru + 1) << 2) * (iordru + 1),
        ((iordrv + 1) << 2) * (iordrv + 1),
        ((iordru + 1) << 1) * nbpntu,
        ((iordrv + 1) << 1) * nbpntv,
    ];
    let mut wrkar: Vec<f64> = vec![0.; sizes.iter().sum::<i32>() as usize];
    let (uhermt, rest) = wrkar.split_at_mut(sizes[0] as usize);
    let (vhermt, rest) = rest.split_at_mut(sizes[1] as usize);
    let (fpntbu, fpntbv) = rest.split_at_mut(sizes[2] as usize);
    if (0..=2).contains(&iordru) {
        mma1her_(iordru, uhermt, iercod);
        if *iercod > 0 {
            *iercod = 1;
            return 0;
        }
        mma2cd3_(grid, isos_u, fpntbu, uhermt, tables);
    }
    if (0..=2).contains(&iordrv) {
        mma1her_(iordrv, vhermt, iercod);
        if *iercod > 0 {
            *iercod = 1;
            return 0;
        }
        mma2cd2_(grid, isos_v, fpntbv, vhermt, tables);
    }
    if iordru >= 0 && iordrv >= 0 {
        mma2cd1_(grid, corners, [fpntbu, fpntbv], [uhermt, vhermt], tables);
    }
    0
}

/// Samples each Hermite polynomial (column of `hermit`) at the roots into `fpntab`
/// (`(nbpnt, 2*order+2)`), the first loop of `mma2cd1_`, `mma2cd2_`, `mma2cd3_`.
pub(super) fn sample_hermite(
    order: i32,
    nbpnt: i32,
    roots: &[f64],
    hermit: &[f64],
    fpntab: &mut [f64],
) {
    let ncfh: i32 = (order + 1) << 1;
    let her = F2cArray::new(hermit, ncfh);
    let roots = F2cArray::new(roots, 1);
    let mut fpnt = F2cArrayMut::new(fpntab, nbpnt + 1);
    for ii in 1..=ncfh {
        for kk in 1..=nbpnt {
            mmmpocur_(
                ncfh,
                1,
                ncfh,
                her.tail(ii * ncfh),
                roots[kk],
                fpnt.tail_mut(kk + ii * nbpnt),
            );
        }
    }
}

/// The sampled iso tables of one direction as shifted views.
struct IsoViews<'a> {
    sotb1: F2cArray<'a, f64>,
    sotb2: F2cArray<'a, f64>,
    ditb1: F2cArray<'a, f64>,
    ditb2: F2cArray<'a, f64>,
    d1: i32,
    d2: i32,
}

impl<'a> IsoViews<'a> {
    fn new(isos: &IsoTables<'a>, nbpnt: i32, ndimen: i32) -> Self {
        let (d1, d2) = (nbpnt / 2 + 1, ndimen);
        let offset: i32 = d1 * (d2 + 1);
        IsoViews {
            sotb1: F2cArray::new(isos.sotb1, offset),
            sotb2: F2cArray::new(isos.sotb2, offset),
            ditb1: F2cArray::new(isos.ditb1, offset),
            ditb2: F2cArray::new(isos.ditb2, offset),
            d1,
            d2,
        }
    }

    fn at(&self, k: i32, nd: i32, ii: i32) -> i32 {
        k + (nd + ii * self.d2) * self.d1
    }
}

/// `mma2cd3_`: subtracts the Hermite interpolation in U of the U-iso tables
/// (`sotbu*`, `ditbu*`) sampled at the U roots.
fn mma2cd3_(
    grid: &SquareGrid<'_>,
    isos: &IsoTables<'_>,
    fpntab: &mut [f64],
    uhermt: &[f64],
    tables: &mut SquareTables<'_>,
) {
    let (nbpntu, nbpntv, iordru) = (grid.nbpntu, grid.nbpntv, grid.iordru);
    sample_hermite(iordru, nbpntu, grid.urootl, uhermt, fpntab);
    let fpnt = F2cArray::new(fpntab, nbpntu + 1);
    let fp = |k: i32, col: i32| fpnt[k + col * nbpntu];
    let iso = IsoViews::new(isos, nbpntv, grid.ndimen);
    let mut t = SquareViews::new(tables, nbpntu, nbpntv);
    let (nvroo, nuroo) = (nbpntv / 2, nbpntu / 2);
    for nd in 1..=grid.ndimen {
        for ii in 1..=iordru + 1 {
            let (c1, c2) = ((ii << 1) - 1, ii << 1);
            for jj in 1..=nvroo {
                let k: i32 = iso.at(jj, nd, ii);
                let (bid1, bid2, bid3, bid4) =
                    (iso.sotb1[k], iso.sotb2[k], iso.ditb1[k], iso.ditb2[k]);
                for kk in 1..=nuroo {
                    let (kkp, kkm) = ((nbpntu + 1) / 2 + kk, nuroo - kk + 1);
                    let (f, h) = (t.f(kk, jj, nd), t.h(kk, jj, nd));
                    t.soso[f] = t.soso[f]
                        - bid1 * (fp(kkp, c1) + fp(kkm, c1))
                        - bid2 * (fp(kkp, c2) + fp(kkm, c2));
                    t.diso[h] = t.diso[h]
                        - bid1 * (fp(kkp, c1) - fp(kkm, c1))
                        - bid2 * (fp(kkp, c2) - fp(kkm, c2));
                    t.sodi[h] = t.sodi[h]
                        - bid3 * (fp(kkp, c1) + fp(kkm, c1))
                        - bid4 * (fp(kkp, c2) + fp(kkm, c2));
                    t.didi[f] = t.didi[f]
                        - bid3 * (fp(kkp, c1) - fp(kkm, c1))
                        - bid4 * (fp(kkp, c2) - fp(kkm, c2));
                }
            }
        }
        if nbpntu % 2 == 1 {
            for ii in 1..=iordru + 1 {
                let (c1, c2) = ((ii << 1) - 1, ii << 1);
                for jj in 1..=nvroo {
                    let k: i32 = iso.at(jj, nd, ii);
                    let bid1: f64 =
                        iso.sotb1[k] * fp(nuroo + 1, c1) + iso.sotb2[k] * fp(nuroo + 1, c2);
                    let f: i32 = t.f(0, jj, nd);
                    t.soso[f] -= bid1;
                    let bid2: f64 =
                        iso.ditb1[k] * fp(nuroo + 1, c1) + iso.ditb2[k] * fp(nuroo + 1, c2);
                    t.didi[f] -= bid2;
                }
            }
        }
        if nbpntv % 2 == 1 {
            for ii in 1..=iordru + 1 {
                let (c1, c2) = ((ii << 1) - 1, ii << 1);
                let k: i32 = iso.at(0, nd, ii);
                for kk in 1..=nuroo {
                    let (kkp, kkm) = ((nbpntu + 1) / 2 + kk, nuroo - kk + 1);
                    let bid1: f64 = iso.sotb1[k] * (fp(kkp, c1) + fp(kkm, c1))
                        + iso.sotb2[k] * (fp(kkp, c2) + fp(kkm, c2));
                    let f: i32 = t.f(kk, 0, nd);
                    t.soso[f] -= bid1;
                    let bid2: f64 = iso.sotb1[k] * (fp(kkp, c1) - fp(kkm, c1))
                        + iso.sotb2[k] * (fp(kkp, c2) - fp(kkm, c2));
                    t.didi[f] -= bid2;
                }
            }
        }
        if nbpntu % 2 == 1 && nbpntv % 2 == 1 {
            for ii in 1..=iordru + 1 {
                let k: i32 = iso.at(0, nd, ii);
                let bid1: f64 = iso.sotb1[k] * fp(nuroo + 1, (ii << 1) - 1)
                    + iso.sotb2[k] * fp(nuroo + 1, ii << 1);
                let f: i32 = t.f(0, 0, nd);
                t.soso[f] -= bid1;
            }
        }
    }
}

/// `mma2cd2_`: subtracts the Hermite interpolation in V of the V-iso tables
/// (`sotbv*`, `ditbv*`) sampled at the V roots.
fn mma2cd2_(
    grid: &SquareGrid<'_>,
    isos: &IsoTables<'_>,
    fpntab: &mut [f64],
    vhermt: &[f64],
    tables: &mut SquareTables<'_>,
) {
    let (nbpntu, nbpntv, iordrv) = (grid.nbpntu, grid.nbpntv, grid.iordrv);
    sample_hermite(iordrv, nbpntv, grid.vrootl, vhermt, fpntab);
    let fpnt = F2cArray::new(fpntab, nbpntv + 1);
    let fp = |k: i32, col: i32| fpnt[k + col * nbpntv];
    let iso = IsoViews::new(isos, nbpntu, grid.ndimen);
    let mut t = SquareViews::new(tables, nbpntu, nbpntv);
    let (nuroo, nvroo) = (nbpntu / 2, nbpntv / 2);
    for nd in 1..=grid.ndimen {
        for ii in 1..=iordrv + 1 {
            let (c1, c2) = ((ii << 1) - 1, ii << 1);
            for kk in 1..=nuroo {
                let k: i32 = iso.at(kk, nd, ii);
                let (bid1, bid2, bid3, bid4) =
                    (iso.sotb1[k], iso.sotb2[k], iso.ditb1[k], iso.ditb2[k]);
                for jj in 1..=nvroo {
                    let (jjp, jjm) = ((nbpntv + 1) / 2 + jj, nvroo - jj + 1);
                    let (f, h) = (t.f(kk, jj, nd), t.h(kk, jj, nd));
                    t.soso[f] = t.soso[f]
                        - bid1 * (fp(jjp, c1) + fp(jjm, c1))
                        - bid2 * (fp(jjp, c2) + fp(jjm, c2));
                    t.diso[h] = t.diso[h]
                        - bid3 * (fp(jjp, c1) + fp(jjm, c1))
                        - bid4 * (fp(jjp, c2) + fp(jjm, c2));
                    t.sodi[h] = t.sodi[h]
                        - bid1 * (fp(jjp, c1) - fp(jjm, c1))
                        - bid2 * (fp(jjp, c2) - fp(jjm, c2));
                    t.didi[f] = t.didi[f]
                        - bid3 * (fp(jjp, c1) - fp(jjm, c1))
                        - bid4 * (fp(jjp, c2) - fp(jjm, c2));
                }
            }
        }
        if nbpntv % 2 == 1 {
            for ii in 1..=iordrv + 1 {
                let (c1, c2) = ((ii << 1) - 1, ii << 1);
                for kk in 1..=nuroo {
                    let k: i32 = iso.at(kk, nd, ii);
                    let bid1: f64 =
                        iso.sotb1[k] * fp(nvroo + 1, c1) + iso.sotb2[k] * fp(nvroo + 1, c2);
                    let f: i32 = t.f(kk, 0, nd);
                    t.soso[f] -= bid1;
                    let bid2: f64 =
                        iso.ditb1[k] * fp(nvroo + 1, c1) + iso.ditb2[k] * fp(nvroo + 1, c2);
                    t.didi[f] -= bid2;
                }
            }
        }
        if nbpntu % 2 == 1 {
            for ii in 1..=iordrv + 1 {
                let (c1, c2) = ((ii << 1) - 1, ii << 1);
                let k: i32 = iso.at(0, nd, ii);
                for jj in 1..=nvroo {
                    let (jjp, jjm) = ((nbpntv + 1) / 2 + jj, nvroo - jj + 1);
                    let bid1: f64 = iso.sotb1[k] * (fp(jjp, c1) + fp(jjm, c1))
                        + iso.sotb2[k] * (fp(jjp, c2) + fp(jjm, c2));
                    let f: i32 = t.f(0, jj, nd);
                    t.soso[f] -= bid1;
                    let bid2: f64 = iso.sotb1[k] * (fp(jjp, c1) - fp(jjm, c1))
                        + iso.sotb2[k] * (fp(jjp, c2) - fp(jjm, c2));
                    // Native quirk: this correction goes to diditb(jj, 0), not to the
                    // diditb(0, jj) that the sosotb line above and mma2cd3_ use.
                    let transposed: i32 = t.f(jj, 0, nd);
                    t.didi[transposed] -= bid2;
                }
            }
        }
        if nbpntu % 2 == 1 && nbpntv % 2 == 1 {
            for ii in 1..=iordrv + 1 {
                let k: i32 = iso.at(0, nd, ii);
                let bid1: f64 = iso.sotb1[k] * fp(nvroo + 1, (ii << 1) - 1)
                    + iso.sotb2[k] * fp(nvroo + 1, ii << 1);
                let f: i32 = t.f(0, 0, nd);
                t.soso[f] -= bid1;
            }
        }
    }
}
