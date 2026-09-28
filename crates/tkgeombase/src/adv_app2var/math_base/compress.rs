// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Repacking of coefficient tables: `mmapcmp_`, `mmfmca8_`, `mmfmca9_`, `mmfmcb5_`,
//! `mmfmtb1_` and `mmfmcar_`.
//!
//! The native `mmfmca8_`, `mmfmca9_` and the file-static `mmarc41_` are also called
//! with the same array as input and output; each reads a coefficient before any
//! write can reach it, so an in-place call is the copy of the input followed by the
//! call, which is how the port's callers make it.
use super::reparam::mmarc41_;
use crate::adv_app2var::f2c::{F2cArray, F2cArrayMut};
use crate::adv_app2var::sys_base::mcrfill_;

/// Splits the canonical curve `crvold` (`(0:ncofmx-1, ndim)`) into its even and odd
/// coefficients (`mmapcmp_`): `crvnew` is `(0:(ncoeff-1)/2, 2, ndim)`; returns 0.
/// `mmapcmp_(1, 21, 5, &crvold, &mut crvnew)`.
pub fn mmapcmp_(ndim: i32, ncofmx: i32, ncoeff: i32, crvold: &[f64], crvnew: &mut [f64]) -> i32 {
    let old_dim1: i32 = ncofmx;
    let crvold = F2cArray::new(crvold, old_dim1);
    let new_dim1: i32 = (ncoeff - 1) / 2 + 1;
    let mut crvnew = F2cArrayMut::new(crvnew, new_dim1 << 1);
    let ndegre: i32 = ncoeff - 1;
    for nd in 1..=ndim {
        let mut ipair: i32 = 0;
        for idg in 0..=ndegre / 2 {
            crvnew[idg + (nd << 1) * new_dim1] = crvold[ipair + nd * old_dim1];
            ipair += 2;
        }
        if ndegre < 1 {
            continue;
        }
        let mut impair: i32 = 1;
        for idg in 0..=(ndegre - 1) / 2 {
            crvnew[idg + ((nd << 1) + 1) * new_dim1] = crvold[impair + nd * old_dim1];
            impair += 2;
        }
    }
    0
}

/// Expands a `(ndimen, ncoefu, ncoefv)` table into `(ndimax, ncfumx, ncoefv)`
/// (`mmfmca8_`), from the last element down, as the in-place native call requires;
/// returns 0. `mmfmca8_(3, 2, 2, 3, 4, 2, &tabini, &mut tabres)`.
pub fn mmfmca8_(
    ndimen: i32,
    ncoefu: i32,
    ncoefv: i32,
    ndimax: i32,
    ncfumx: i32,
    _ncfvmx: i32,
    tabini: &[f64],
    tabres: &mut [f64],
) -> i32 {
    let (ini1, ini2) = (ndimen, ncoefu);
    let (res1, res2) = (ndimax, ncfumx);
    let ini = F2cArray::new(tabini, ini1 * (ini2 + 1) + 1);
    let mut res = F2cArrayMut::new(tabres, res1 * (res2 + 1) + 1);
    if ndimax != ndimen {
        for k in (1..=ncoefv).rev() {
            for j in (1..=ncoefu).rev() {
                for i in (1..=ndimen).rev() {
                    res[i + (j + k * res2) * res1] = ini[i + (j + k * ini2) * ini1];
                }
            }
        }
        return 0;
    }
    if ncoefu != ncfumx {
        let ilong: i32 = (ndimen << 3) * ncoefu;
        for k in (1..=ncoefv).rev() {
            let source: &[f64] = ini.tail((k * ini2 + 1) * ini1 + 1);
            mcrfill_(ilong, source, res.tail_mut((k * res2 + 1) * res1 + 1));
        }
        return 0;
    }
    let ilong: i32 = (ndimen << 3) * ncoefu * ncoefv;
    mcrfill_(
        ilong,
        ini.tail(ini1 * (ini2 + 1) + 1),
        res.tail_mut(res1 * (res2 + 1) + 1),
    );
    0
}

/// Compacts a `(ndimax, ncfumx, ncfvmx)` table into `(ndimen, ncoefu, ncoefv)`
/// (`mmfmca9_`), from the first element up; returns 0.
/// `mmfmca9_(3, 4, 2, 3, 2, 2, &tabini, &mut tabres)`.
pub fn mmfmca9_(
    ndimax: i32,
    ncfumx: i32,
    _ncfvmx: i32,
    ndimen: i32,
    ncoefu: i32,
    ncoefv: i32,
    tabini: &[f64],
    tabres: &mut [f64],
) -> i32 {
    let (ini1, ini2) = (ndimax, ncfumx);
    let (res1, res2) = (ndimen, ncoefu);
    let ini = F2cArray::new(tabini, ini1 * (ini2 + 1) + 1);
    let mut res = F2cArrayMut::new(tabres, res1 * (res2 + 1) + 1);
    if ndimen != ndimax {
        for k in 1..=ncoefv {
            for j in 1..=ncoefu {
                for i in 1..=ndimen {
                    res[i + (j + k * res2) * res1] = ini[i + (j + k * ini2) * ini1];
                }
            }
        }
        return 0;
    }
    if ncoefu != ncfumx {
        let ilong: i32 = (ndimen << 3) * ncoefu;
        for k in 1..=ncoefv {
            let source: &[f64] = ini.tail((k * ini2 + 1) * ini1 + 1);
            mcrfill_(ilong, source, res.tail_mut((k * res2 + 1) * res1 + 1));
        }
        return 0;
    }
    let ilong: i32 = (ndimen << 3) * ncoefu * ncoefv;
    mcrfill_(
        ilong,
        ini.tail(ini1 * (ini2 + 1) + 1),
        res.tail_mut(res1 * (res2 + 1) + 1),
    );
    0
}

/// Restricts a canonical patch `(ndimen, ncofmx, ncoefv)` on [0,1]x[0,1] to
/// `[upara1, upara2] x [vpara1, vpara2]` (`mmfmcar_`). `iercod` is 10 for invalid
/// coefficient counts and the `mmarc41_` code otherwise; returns 0.
/// `mmfmcar_(3, 4, 4, 3, &patold, 0.0, 0.5, 0.0, 1.0, &mut patnew, &mut iercod)`.
pub fn mmfmcar_(
    ndimen: i32,
    ncofmx: i32,
    ncoefu: i32,
    ncoefv: i32,
    patold: &[f64],
    upara1: f64,
    upara2: f64,
    vpara1: f64,
    vpara2: f64,
    patnew: &mut [f64],
    iercod: &mut i32,
) -> i32 {
    *iercod = 0;
    if ncofmx < ncoefu || !(1..=61).contains(&ncoefu) || !(1..=61).contains(&ncoefv) {
        *iercod = 10;
        return 0;
    }
    let u_identity: bool = upara1 == 0. && upara2 == 1.;
    if u_identity && vpara1 == 0. && vpara2 == 1. {
        mcrfill_((ndimen << 3) * ncofmx * ncoefv, patold, patnew);
        return 0;
    }
    let dims = PatchDims {
        ndimen,
        ncofmx,
        ncoefu,
        ncoefv,
    };
    if !u_identity {
        dims.restrict_u(patold, [upara1, upara2], patnew, iercod);
    }
    if vpara1 == 0. && vpara2 == 1. {
        return 0;
    }
    let source: Vec<f64> = if u_identity {
        patold.to_vec()
    } else {
        patnew.to_vec()
    };
    dims.restrict_v(&source, [vpara1, vpara2], patnew, iercod);
    0
}

/// The sizes of the patch of `mmfmcar_`.
struct PatchDims {
    ndimen: i32,
    ncofmx: i32,
    ncoefu: i32,
    ncoefv: i32,
}

impl PatchDims {
    /// The U loop of `mmfmcar_`: one `mmarc41_` per V coefficient.
    fn restrict_u(&self, patold: &[f64], range: [f64; 2], patnew: &mut [f64], iercod: &mut i32) {
        let (dim1, dim2) = (self.ndimen, self.ncofmx);
        let offset: i32 = dim1 * (dim2 + 1) + 1;
        let old = F2cArray::new(patold, offset);
        let mut new = F2cArrayMut::new(patnew, offset);
        for kk in 1..=self.ncoefv {
            let start: i32 = (kk * dim2 + 1) * dim1 + 1;
            let crvold: &[f64] = old.tail(start);
            mmarc41_(
                self.ndimen,
                self.ndimen,
                self.ncoefu,
                crvold,
                range[0],
                range[1],
                new.tail_mut(start),
                iercod,
            );
        }
    }

    /// The V part of `mmfmcar_` (label L2000) on `source`, the input patch or the
    /// U-restricted one: in place when the rows are full, through the compacted
    /// scratch table `tbaux` otherwise.
    fn restrict_v(&self, source: &[f64], range: [f64; 2], patnew: &mut [f64], iercod: &mut i32) {
        let numax: i32 = self.ndimen * self.ncoefu;
        if self.ncofmx == self.ncoefu {
            mmarc41_(
                numax,
                numax,
                self.ncoefv,
                source,
                range[0],
                range[1],
                patnew,
                iercod,
            );
            return;
        }
        let ksize: i32 = self.ndimen * self.ncoefu * self.ncoefv;
        let mut tbaux: Vec<f64> = vec![0.; ksize as usize];
        mmfmca9_(
            self.ndimen,
            self.ncofmx,
            self.ncoefv,
            self.ndimen,
            self.ncoefu,
            self.ncoefv,
            source,
            &mut tbaux,
        );
        let compact: Vec<f64> = tbaux.clone();
        mmarc41_(
            numax,
            numax,
            self.ncoefv,
            &compact,
            range[0],
            range[1],
            &mut tbaux,
            iercod,
        );
        mmfmca8_(
            self.ndimen,
            self.ncoefu,
            self.ncoefv,
            self.ndimen,
            self.ncofmx,
            self.ncoefv,
            &tbaux,
            patnew,
        );
    }
}

/// Copies or transposes a curve between the `(ndimax, ncf1mx)` layout `courb1` and
/// the `(ncf2mx, ndimen)` layout `courb2` (`mmfmcb5_`): `isenmsc` 1 fills `courb2`,
/// -1 fills `courb1`. `iercod` is 3119 for inconsistent sizes, -3136 for the plain
/// copy of a 1D curve and 3164 for another direction; returns 0.
/// `mmfmcb5_(1, 3, 4, &mut c1, 4, 4, 3, &mut c2, &mut iercod)`.
pub fn mmfmcb5_(
    isenmsc: i32,
    ndimax: i32,
    ncf1mx: i32,
    courb1: &mut [f64],
    ncoeff: i32,
    ncf2mx: i32,
    ndimen: i32,
    courb2: &mut [f64],
    iercod: &mut i32,
) -> i32 {
    if ndimen > ndimax || ncoeff > ncf1mx || ncoeff > ncf2mx {
        *iercod = 3119;
        return 0;
    }
    if ndimen == 1 && ncf1mx == ncf2mx {
        let nboct: i32 = ncf2mx << 3;
        match isenmsc {
            1 => mcrfill_(nboct, courb1, courb2),
            -1 => mcrfill_(nboct, courb2, courb1),
            _ => 0,
        };
        *iercod = -3136;
        return 0;
    }
    *iercod = 0;
    let (dim1, dim2) = (ndimax, ncf2mx);
    let mut c1 = F2cArrayMut::new(courb1, dim1 + 1);
    let mut c2 = F2cArrayMut::new(courb2, dim2 + 1);
    match isenmsc {
        1 => each_coefficient(ndimen, ncoeff, |nd, i| {
            c2[i + nd * dim2] = c1[nd + i * dim1]
        }),
        -1 => each_coefficient(ndimen, ncoeff, |nd, i| {
            c1[nd + i * dim1] = c2[i + nd * dim2]
        }),
        _ => *iercod = 3164,
    }
    0
}

fn each_coefficient(ndimen: i32, ncoeff: i32, mut visit: impl FnMut(i32, i32)) {
    for nd in 1..=ndimen {
        for i in 1..=ncoeff {
            visit(nd, i);
        }
    }
}

/// Transposes the `(isize1, jsize1)` block of `table1` (leading dimension `maxsz1`)
/// into `table2` (leading dimension `maxsz2`) through a scratch table, so both may
/// be the same array (`mmfmtb1_`). `isize2`/`jsize2` receive the new sizes;
/// `iercod` is 1 when the block exceeds the dimensions; returns 0.
/// `mmfmtb1_(2, &t1, 2, 3, 3, &mut t2, &mut i2, &mut j2, &mut iercod)`.
pub fn mmfmtb1_(
    maxsz1: i32,
    table1: &[f64],
    isize1: i32,
    jsize1: i32,
    maxsz2: i32,
    table2: &mut [f64],
    isize2: &mut i32,
    jsize2: &mut i32,
    iercod: &mut i32,
) -> i32 {
    *iercod = 0;
    if isize1 > maxsz1 || jsize1 > maxsz2 {
        *iercod = 1;
        return 0;
    }
    let table1 = F2cArray::new(table1, maxsz1 + 1);
    let isize: i32 = maxsz2 * isize1;
    // The native zone is uninitialised; entries past jsize1 are copied as they are.
    let mut work: Vec<f64> = vec![0.; isize.max(0) as usize];
    for ii in 1..=isize1 {
        let iipt: i32 = (ii - 1) * maxsz2;
        for jj in 1..=jsize1 {
            work[(iipt + (jj - 1)) as usize] = table1[ii + jj * maxsz1];
        }
    }
    mcrfill_(isize << 3, &work, table2);
    *isize2 = jsize1;
    *jsize2 = isize1;
    0
}
