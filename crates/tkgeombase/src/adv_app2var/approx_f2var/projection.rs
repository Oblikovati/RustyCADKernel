//! The Gauss projections of the patch approximation: the file-static `mma2cfu_`
//! (along U, into the `chpair`/`chimpr` tables) and `mma2cfv_` (along V, into the
//! Jacobi coefficients), and the error estimates `mma2er1_`, `mma2er2_`, `mma2moy_`.
use crate::adv_app2var::f2c::{F2cArray, F2cArrayMut};
use crate::adv_app2var::math_base::mzsnorm_;

/// The four tables of one dimension of a square discretisation, each starting at
/// the dimension's block: `soso` and `didi` 0-based, `diso` and `sodi` 1-based.
pub(super) struct SquareBlock<'a> {
    pub(super) soso: &'a [f64],
    pub(super) diso: &'a [f64],
    pub(super) sodi: &'a [f64],
    pub(super) didi: &'a [f64],
}

/// `mma2cfu_`: the projection along U of one dimension's tables on the Jacobi
/// polynomial of degree `ndujac` (Gauss weights `gssutb`), into `chpair`
/// (`0:nbpntv/2`) and `chimpr` (`1:nbpntv/2`).
pub(super) fn mma2cfu_(
    ndujac: i32,
    nbpnt: [i32; 2],
    block: &SquareBlock<'_>,
    gssutb: &[f64],
    chpair: &mut [f64],
    chimpr: &mut [f64],
) {
    let [nbpntu, nbpntv] = nbpnt;
    let (d1, h1) = (nbpntu / 2 + 1, nbpntu / 2);
    let soso = F2cArray::new(block.soso, 0);
    let didi = F2cArray::new(block.didi, 0);
    let diso = F2cArray::new(block.diso, h1 + 1);
    let sodi = F2cArray::new(block.sodi, h1 + 1);
    let mut chimpr = F2cArrayMut::new(chimpr, 1);
    let (nptu2, nptv2) = (nbpntu / 2, nbpntv / 2);
    let even: bool = ndujac % 2 == 0;
    for jj in 1..=nptv2 {
        let (mut bid1, mut bid2) = (0., 0.);
        for ii in 1..=nptu2 {
            let bid0: f64 = gssutb[ii as usize];
            if even {
                bid1 += soso[ii + jj * d1] * bid0;
                bid2 += sodi[ii + jj * h1] * bid0;
            } else {
                bid1 += diso[ii + jj * h1] * bid0;
                bid2 += didi[ii + jj * d1] * bid0;
            }
        }
        chpair[jj as usize] = bid1;
        chimpr[jj] = bid2;
    }
    if nbpntu % 2 != 0 && even {
        let bid0: f64 = gssutb[0];
        for jj in 1..=nptv2 {
            chpair[jj as usize] += soso[jj * d1] * bid0;
            chimpr[jj] += didi[jj * d1] * bid0;
        }
    }
    if nbpntv % 2 != 0 {
        let middle = if even { &soso } else { &didi };
        let mut bid1: f64 = 0.;
        for ii in 1..=nptu2 {
            bid1 += middle[ii] * gssutb[ii as usize];
        }
        chpair[0] = bid1;
        if nbpntu % 2 != 0 {
            chpair[0] += soso[0] * gssutb[0];
        }
    }
}

/// `mma2cfv_`: the projection along V of the `chpair`/`chimpr` columns
/// `mindgu..=maxdgu` on the Jacobi polynomial of degree `ndvjac` (weights
/// `gssvtb`), into `patjac[mindgu..=maxdgu]` (a slice starting at `mindgu`).
pub(super) fn mma2cfv_(
    ndvjac: i32,
    degrees: [i32; 2],
    nbpntv: i32,
    gssvtb: &[f64],
    chpair: &[f64],
    chimpr: &[f64],
    patjac: &mut [f64],
) {
    let [mindgu, maxdgu] = degrees;
    let mut pat = F2cArrayMut::new(patjac, mindgu);
    let (cid1, chd1) = (nbpntv / 2, nbpntv / 2 + 1);
    let chimpr = F2cArray::new(chimpr, cid1 * mindgu + 1);
    let chpair = F2cArray::new(chpair, chd1 * mindgu);
    let nptv2: i32 = nbpntv / 2;
    let even: bool = ndvjac % 2 == 0;
    for ii in mindgu..=maxdgu {
        let mut bid1: f64 = 0.;
        for jj in 1..=nptv2 {
            let value: f64 = if even {
                chpair[jj + ii * chd1]
            } else {
                chimpr[jj + ii * cid1]
            };
            bid1 += value * gssvtb[jj as usize];
        }
        pat[ii] = bid1;
    }
    if nbpntv % 2 != 0 && even {
        let bid1: f64 = gssvtb[0];
        for ii in mindgu..=maxdgu {
            pat[ii] += bid1 * chpair[ii * chd1];
        }
    }
}

/// The Jacobi patch of one subspace and its maxima for the error bounds: `patjac`
/// starts at the subspace's first dimension, `(0:ndjacu, 0:ndjacv, ndimen)`.
pub(super) struct JacobiBlock<'a> {
    pub(super) ndjac: [i32; 2],
    pub(super) ndimen: i32,
    pub(super) iordr: [i32; 2],
    pub(super) xmaxju: &'a [f64],
    pub(super) xmaxjv: &'a [f64],
    pub(super) patjac: &'a [f64],
}

impl JacobiBlock<'_> {
    fn at(&self, ii: i32, jj: i32, nd: i32) -> f64 {
        let (pd1, pd2) = (self.ndjac[0] + 1, self.ndjac[1] + 1);
        self.patjac[(ii + (jj + nd * pd2) * pd1 - pd1 * pd2) as usize]
    }
}

/// `mma2er1_`: the bound of the coefficients `[mindgu, maxdgu] x [mindgv, maxdgv]`
/// per dimension into `vecerr`, combined into `erreur` as `|(erreur, norm)|`.
pub(super) fn mma2er1_(
    block: &JacobiBlock<'_>,
    range: [i32; 4],
    vecerr: &mut [f64],
    erreur: &mut f64,
) {
    let [mindgu, maxdgu, mindgv, maxdgv] = range;
    let minu: i32 = (block.iordr[0] + 1) << 1;
    let minv: i32 = (block.iordr[1] + 1) << 1;
    for nd in 1..=block.ndimen {
        let mut bid1: f64 = 0.;
        for jj in mindgv..=maxdgv {
            let mut bid0: f64 = 0.;
            for ii in mindgu..=maxdgu {
                bid0 += block.at(ii, jj, nd).abs() * block.xmaxju[(ii - minu) as usize];
            }
            bid1 += bid0 * block.xmaxjv[(jj - minv) as usize];
        }
        vecerr[(nd - 1) as usize] = bid1;
    }
    let bid1: f64 = mzsnorm_(block.ndimen, vecerr);
    let vaux: [f64; 2] = [*erreur, bid1];
    *erreur = mzsnorm_(2, &vaux);
}

/// `mma2er2_`: lowers the degrees `(maxdgu, maxdgv)` one row or column at a time,
/// dropping the cheaper of the last U column and V row while the combined error
/// stays below `epmscut`; `erreur` accumulates the dropped error and `newdg`
/// receives the degrees (at least 1). `vecerr` is the caller's scratch: when a
/// direction reaches its minimum only its first entry is reset, as natively.
pub(super) fn mma2er2_(
    block: &JacobiBlock<'_>,
    range: [i32; 4],
    epmscut: f64,
    vecerr: &mut [f64],
    erreur: &mut f64,
    newdg: &mut [i32; 2],
) {
    let [mindgu, maxdgu, mindgv, maxdgv] = range;
    let i2rdu: i32 = (block.iordr[0] + 1) << 1;
    let i2rdv: i32 = (block.iordr[1] + 1) << 1;
    let (mut nu, mut nv) = (maxdgu, maxdgv);
    loop {
        if nv > mindgv {
            let bid0: f64 = block.xmaxjv[(nv - i2rdv) as usize];
            for nd in 1..=block.ndimen {
                let mut bid1: f64 = 0.;
                for ii in i2rdu..=nu {
                    bid1 += block.at(ii, nv, nd).abs() * block.xmaxju[(ii - i2rdu) as usize] * bid0;
                }
                vecerr[(nd - 1) as usize] = bid1;
            }
        } else {
            vecerr[0] = epmscut * 2.;
        }
        let errnv: f64 = mzsnorm_(block.ndimen, vecerr);
        if nu > mindgu {
            let bid0: f64 = block.xmaxju[(nu - i2rdu) as usize];
            for nd in 1..=block.ndimen {
                let mut bid1: f64 = 0.;
                for jj in i2rdv..=nv {
                    bid1 += block.at(nu, jj, nd).abs() * block.xmaxjv[(jj - i2rdv) as usize] * bid0;
                }
                vecerr[(nd - 1) as usize] = bid1;
            }
        } else {
            vecerr[0] = epmscut * 2.;
        }
        let errnu: f64 = mzsnorm_(block.ndimen, vecerr);
        let mut vaux: [f64; 2] = [*erreur, errnu];
        let errnu: f64 = mzsnorm_(2, &vaux);
        vaux[1] = errnv;
        let errnv: f64 = mzsnorm_(2, &vaux);
        if errnu > errnv {
            if errnv >= epmscut || errnv.is_nan() {
                break;
            }
            *erreur = errnv;
            nv -= 1;
        } else {
            if errnu >= epmscut || errnu.is_nan() {
                break;
            }
            *erreur = errnu;
            nu -= 1;
        }
    }
    *newdg = [nu.max(1), nv.max(1)];
}

/// `mma2moy_`: the average error of dropping the coefficients from degree
/// `(mindgu, mindgv)` on, bounded by `(maxdgu, maxdgv)`, of the patch `patjac`
/// (`(0:ndgumx, 0:ndgvmx, ndimen)`).
pub(super) fn mma2moy_(
    ndgmx: [i32; 2],
    ndimen: i32,
    range: [i32; 4],
    iordr: [i32; 2],
    patjac: &[f64],
    errmoy: &mut f64,
) {
    let [mindgu, maxdgu, mindgv, maxdgv] = range;
    let (pd1, pd2) = (ndgmx[0] + 1, ndgmx[1] + 1);
    let pat = F2cArray::new(patjac, pd1 * pd2);
    let idebu: i32 = (iordr[0] + 1) << 1;
    let idebv: i32 = (iordr[1] + 1) << 1;
    let minu: i32 = idebu.max(mindgu);
    let minv: i32 = idebv.max(mindgv);
    let mut bid0: f64 = 0.;
    *errmoy = 0.;
    for nd in 1..=ndimen {
        for jj in minv..=maxdgv {
            for ii in idebu..=maxdgu {
                let bid1: f64 = pat[ii + (jj + nd * pd2) * pd1];
                bid0 += bid1 * bid1;
            }
        }
    }
    for nd in 1..=ndimen {
        for jj in idebv..=minv - 1 {
            for ii in minu..=maxdgu {
                let bid1: f64 = pat[ii + (jj + nd * pd2) * pd1];
                bid0 += bid1 * bid1;
            }
        }
    }
    bid0 /= 4.;
    *errmoy = bid0.sqrt();
}
