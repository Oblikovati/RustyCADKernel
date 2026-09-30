//! Adding the boundary constraints to a patch and converting it to the canonical
//! base: `mma2ac1_`, `mma2ac2_`, `mma2ac3_`, `mma2can_` and the file-static
//! `mmjacpt_`.
use crate::adv_app2var::f2c::{F2cArray, F2cArrayMut};
use crate::adv_app2var::math_base::{mmfmca8_, mmjaccv_};
use crate::adv_app2var::sys_base::mvriraz_;

/// The corner constraints `contr1..contr4` (`(ndimen, iordru+2, iordrv+2)` each, at
/// (U0,V0), (U1,V0), (U0,V1), (U1,V1)) of a patch.
pub struct CornerConstraints<'a> {
    /// `contr1`: derivatives at (U0,V0).
    pub contr1: &'a [f64],
    /// `contr2`: derivatives at (U1,V0).
    pub contr2: &'a [f64],
    /// `contr3`: derivatives at (U0,V1).
    pub contr3: &'a [f64],
    /// `contr4`: derivatives at (U1,V1).
    pub contr4: &'a [f64],
}

/// Subtracts the tensor Hermite interpolation of the corner constraints from the
/// canonical patch `patjac` (`(0:mxujac, 0:mxvjac, ndimen)`) (`mma2ac1_`); `uhermt`
/// and `vhermt` are the `mma1her_` bases. Returns 0.
/// `mma2ac1_(3, 5, 5, 0, 0, &corners, &uh, &vh, &mut patcan)`.
pub fn mma2ac1_(
    ndimen: i32,
    mxujac: i32,
    mxvjac: i32,
    iordru: i32,
    iordrv: i32,
    corners: &CornerConstraints<'_>,
    uhermt: &[f64],
    vhermt: &[f64],
    patjac: &mut [f64],
) -> i32 {
    let (pd1, pd2) = (mxujac + 1, mxvjac + 1);
    let mut pat = F2cArrayMut::new(patjac, pd1 * pd2);
    let (ud1, vd1) = ((iordru << 1) + 2, (iordrv << 1) + 2);
    let (uh, vh) = (F2cArray::new(uhermt, ud1), F2cArray::new(vhermt, vd1));
    let (cd1, cd2) = (ndimen, iordru + 2);
    let offset: i32 = cd1 * (cd2 + 1) + 1;
    let c = [
        corners.contr1,
        corners.contr2,
        corners.contr3,
        corners.contr4,
    ]
    .map(|t| F2cArray::new(t, offset));
    let (ndgu, ndgv) = ((iordru << 1) + 1, (iordrv << 1) + 1);
    for jj in 1..=iordrv + 1 {
        for ii in 1..=iordru + 1 {
            for nd in 1..=ndimen {
                let k: i32 = nd + (ii + jj * cd2) * cd1;
                let (cnt1, cnt2, cnt3, cnt4) = (c[0][k], c[1][k], c[2][k], c[3][k]);
                for kv in 0..=ndgv {
                    let bidv1: f64 = vh[kv + ((jj << 1) - 1) * vd1];
                    let bidv2: f64 = vh[kv + (jj << 1) * vd1];
                    for ku in 0..=ndgu {
                        let bidu1: f64 = uh[ku + ((ii << 1) - 1) * ud1];
                        let bidu2: f64 = uh[ku + (ii << 1) * ud1];
                        let p: i32 = ku + (kv + nd * pd2) * pd1;
                        pat[p] = pat[p]
                            - bidu1 * bidv1 * cnt1
                            - bidu2 * bidv1 * cnt2
                            - bidu1 * bidv2 * cnt3
                            - bidu2 * bidv2 * cnt4;
                    }
                }
            }
        }
    }
    0
}

/// The iso-curve constraints of one side pair: the curves at the two ends and their
/// coefficient counts per derivative order.
pub struct IsoConstraints<'a> {
    /// `ncfiv1`/`ncfiu1`: coefficient counts of the first iso, per derivative.
    pub ncf1: &'a [i32],
    /// `crbiv1`/`crbiu1`: the first iso, `(nclim, ndimen, order+1)`.
    pub crb1: &'a [f64],
    /// `ncfiv2`/`ncfiu2`: coefficient counts of the second iso.
    pub ncf2: &'a [i32],
    /// `crbiv2`/`crbiu2`: the second iso.
    pub crb2: &'a [f64],
}

/// Adds the Hermite interpolation in V of the V-iso constraints to `patjac`
/// (`mma2ac2_`): `nclimu` is the leading dimension of the isos. Returns 0.
/// `mma2ac2_(3, 5, 5, 0, 6, &isos, &vh, &mut patcan)`.
pub fn mma2ac2_(
    ndimen: i32,
    mxujac: i32,
    mxvjac: i32,
    iordrv: i32,
    nclimu: i32,
    isos: &IsoConstraints<'_>,
    vhermt: &[f64],
    patjac: &mut [f64],
) -> i32 {
    let (pd1, pd2) = (mxujac + 1, mxvjac + 1);
    let mut pat = F2cArrayMut::new(patjac, pd1 * pd2);
    let vd1: i32 = (iordrv << 1) + 2;
    let vh = F2cArray::new(vhermt, vd1);
    let (cd1, cd2) = (nclimu, ndimen);
    let (crb1, crb2) = (
        F2cArray::new(isos.crb1, cd1 * (cd2 + 1)),
        F2cArray::new(isos.crb2, cd1 * (cd2 + 1)),
    );
    let (ncf1, ncf2) = (F2cArray::new(isos.ncf1, 1), F2cArray::new(isos.ncf2, 1));
    for ii in 1..=iordrv + 1 {
        let (ndgv1, ndgv2) = (ncf1[ii] - 1, ncf2[ii] - 1);
        for nd in 1..=ndimen {
            for jj in 0..=(iordrv << 1) + 1 {
                let bid1: f64 = vh[jj + ((ii << 1) - 1) * vd1];
                for kk in 0..=ndgv1 {
                    pat[kk + (jj + nd * pd2) * pd1] += bid1 * crb1[kk + (nd + ii * cd2) * cd1];
                }
                let bid2: f64 = vh[jj + (ii << 1) * vd1];
                for kk in 0..=ndgv2 {
                    pat[kk + (jj + nd * pd2) * pd1] += bid2 * crb2[kk + (nd + ii * cd2) * cd1];
                }
            }
        }
    }
    0
}

/// Adds the Hermite interpolation in U of the U-iso constraints to `patjac`
/// (`mma2ac3_`): `nclimv` is the leading dimension of the isos. Returns 0.
/// `mma2ac3_(3, 5, 5, 0, 6, &isos, &uh, &mut patcan)`.
pub fn mma2ac3_(
    ndimen: i32,
    mxujac: i32,
    mxvjac: i32,
    iordru: i32,
    nclimv: i32,
    isos: &IsoConstraints<'_>,
    uhermt: &[f64],
    patjac: &mut [f64],
) -> i32 {
    let (pd1, pd2) = (mxujac + 1, mxvjac + 1);
    let mut pat = F2cArrayMut::new(patjac, pd1 * pd2);
    let ud1: i32 = (iordru << 1) + 2;
    let uh = F2cArray::new(uhermt, ud1);
    let (cd1, cd2) = (nclimv, ndimen);
    let (crb1, crb2) = (
        F2cArray::new(isos.crb1, cd1 * (cd2 + 1)),
        F2cArray::new(isos.crb2, cd1 * (cd2 + 1)),
    );
    let (ncf1, ncf2) = (F2cArray::new(isos.ncf1, 1), F2cArray::new(isos.ncf2, 1));
    for ii in 1..=iordru + 1 {
        let (ndgu1, ndgu2) = (ncf1[ii] - 1, ncf2[ii] - 1);
        for nd in 1..=ndimen {
            for jj in 0..=ndgu1 {
                let bid1: f64 = crb1[jj + (nd + ii * cd2) * cd1];
                for kk in 0..=(iordru << 1) + 1 {
                    pat[kk + (jj + nd * pd2) * pd1] += bid1 * uh[kk + ((ii << 1) - 1) * ud1];
                }
            }
            for jj in 0..=ndgu2 {
                let bid2: f64 = crb2[jj + (nd + ii * cd2) * cd1];
                for kk in 0..=(iordru << 1) + 1 {
                    pat[kk + (jj + nd * pd2) * pd1] += bid2 * uh[kk + (ii << 1) * ud1];
                }
            }
        }
    }
    0
}

/// Converts the Jacobi patch `patjac` (`(ncoefu, ncoefv, ndimen)`) to the canonical
/// base in `patcan` (`(ncfmxu, ncfmxv, ndimen)`, zero-padded), with the scratch
/// `pataux` of three `ncoefu*ncoefv*ndimen` blocks (`mma2can_`). `iercod` is 1 for
/// orders outside -1..=2 or counts beyond the maxima; returns 0.
/// `mma2can_(9, 9, 3, 1, 1, 6, 5, &patjac, &mut aux, &mut patcan, &mut ier)`.
pub fn mma2can_(
    ncfmxu: i32,
    ncfmxv: i32,
    ndimen: i32,
    iordru: i32,
    iordrv: i32,
    ncoefu: i32,
    ncoefv: i32,
    patjac: &[f64],
    pataux: &mut [f64],
    patcan: &mut [f64],
    iercod: &mut i32,
) -> i32 {
    *iercod = 0;
    if !(-1..=2).contains(&iordru)
        || !(-1..=2).contains(&iordrv)
        || ncoefu > ncfmxu
        || ncoefv > ncfmxv
    {
        *iercod = 1;
        return 0;
    }
    mmjacpt_(
        ndimen,
        [ncoefu, ncoefv],
        [iordru, iordrv],
        patjac,
        pataux,
        patcan,
    );
    // In place natively: the expansion runs from the last coefficient down.
    let compact: Vec<f64> = patcan.to_vec();
    mmfmca8_(
        ncoefu, ncoefv, ndimen, ncfmxu, ncfmxv, ndimen, &compact, patcan,
    );
    let (pd1, pd2) = (ncfmxu, ncfmxv);
    let mut can = F2cArrayMut::new(patcan, pd1 * (pd2 + 1) + 1);
    let ilon1: i32 = ncfmxu - ncoefu;
    let ilon2: i32 = ncfmxu * (ncfmxv - ncoefv);
    for nd in 1..=ndimen {
        if ilon1 > 0 {
            for ii in 1..=ncoefv {
                mvriraz_(ilon1, can.tail_mut(ncoefu + 1 + (ii + nd * pd2) * pd1));
            }
        }
        if ilon2 > 0 {
            mvriraz_(ilon2, can.tail_mut((ncoefv + 1 + nd * pd2) * pd1 + 1));
        }
    }
    0
}

/// `mmjacpt_`: the Jacobi patch `ptclgd` to the canonical base `ptccan`, first along
/// U (`mmjaccv_` on the columns, with the first block of `ptcaux` as scratch), then
/// along V on the transposed first block into the second one, with `ptccan` as
/// scratch. `ptcaux` holds two `ncoefu*ncoefv*ndimen` blocks.
fn mmjacpt_(
    ndimen: i32,
    ncoef: [i32; 2],
    iordr: [i32; 2],
    ptclgd: &[f64],
    ptcaux: &mut [f64],
    ptccan: &mut [f64],
) {
    let [ncoefu, ncoefv] = ncoef;
    let block: usize = (ncoefu * ncoefv * ndimen) as usize;
    mmjaccv_(ncoefu, ndimen * ncoefv, iordr[0], ptclgd, ptcaux, ptccan);
    transpose_blocks(ndimen, ncoefu, ncoefv, ptcaux, ptccan, true);
    let (first, second) = ptcaux.split_at_mut(block);
    mmjaccv_(ncoefv, ndimen * ncoefu, iordr[1], first, ptccan, second);
    transpose_blocks(ndimen, ncoefu, ncoefv, second, ptccan, false);
}

/// Moves `(ncoefu, ncoefv, ndimen)` `ptccan` to `(ncoefv, ncoefu, ndimen)` `aux`
/// (`into_aux`) or back.
fn transpose_blocks(
    ndimen: i32,
    ncoefu: i32,
    ncoefv: i32,
    aux: &mut [f64],
    can: &mut [f64],
    into_aux: bool,
) {
    for nd in 1..=ndimen {
        for jj in 1..=ncoefv {
            for ii in 1..=ncoefu {
                let a: usize = ((jj - 1) + ((ii - 1) + (nd - 1) * ncoefu) * ncoefv) as usize;
                let c: usize = ((ii - 1) + ((jj - 1) + (nd - 1) * ncoefv) * ncoefu) as usize;
                if into_aux {
                    aux[a] = can[c];
                } else {
                    can[c] = aux[a];
                }
            }
        }
    }
}
