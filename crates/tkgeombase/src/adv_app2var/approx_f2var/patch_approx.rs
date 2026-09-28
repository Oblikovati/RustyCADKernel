// Copyright (c) 1996-1999 Matra Datavision
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

//! The approximation of one patch: `mma2ce1_`, and the shared state of the
//! file-static `mma2ce2_` (in `patch_cutting`).
use super::bases::{mma2jmx_, mmapptt_};
use super::patch_cutting::mma2ce2_;
use super::projection::{JacobiBlock, SquareBlock, mma2cfu_, mma2cfv_, mma2er1_};

/// The fixed inputs of `mma2ce1_` in the native order.
pub struct PatchApproxRequest<'a> {
    /// `numdec`: allowed cuttings (0 none, 1 U, 2 V, 3 U or V, 4 V or U, 5 both).
    pub numdec: i32,
    /// `ndimen`: total dimension.
    pub ndimen: i32,
    /// `nbsesp`: number of subspaces.
    pub nbsesp: i32,
    /// `ndimse`: dimension of each subspace.
    pub ndimse: &'a [i32],
    /// `ndminu`: minimum degree in U.
    pub ndminu: i32,
    /// `ndminv`: minimum degree in V.
    pub ndminv: i32,
    /// `ndguli`: maximum degree in U.
    pub ndguli: i32,
    /// `ndgvli`: maximum degree in V.
    pub ndgvli: i32,
    /// `ndjacu`: degree of the Jacobi approximation in U.
    pub ndjacu: i32,
    /// `ndjacv`: degree of the Jacobi approximation in V.
    pub ndjacv: i32,
    /// `iordru`: continuity order in U.
    pub iordru: i32,
    /// `iordrv`: continuity order in V.
    pub iordrv: i32,
    /// `nbpntu`: number of U roots.
    pub nbpntu: i32,
    /// `nbpntv`: number of V roots.
    pub nbpntv: i32,
    /// `epsapr`: tolerance of each subspace.
    pub epsapr: &'a [f64],
}

/// The four symmetric tables of the sampled square (read-only view of
/// [`SquareTables`](super::SquareTables)).
pub struct SquareSamples<'a> {
    /// `sosotb`.
    pub sosotb: &'a [f64],
    /// `disotb`.
    pub disotb: &'a [f64],
    /// `soditb`.
    pub soditb: &'a [f64],
    /// `diditb`.
    pub diditb: &'a [f64],
}

/// The outputs of `mma2ce1_`: the Jacobi patch `(0:ndjacu, 0:ndjacv, ndimen)`, the
/// maximum and average errors per subspace, the degrees reached and the cutting
/// decision `itydec` (0 none, 1 U, 2 V, 3 both).
pub struct PatchApproxOutput<'a> {
    /// `patjac`.
    pub patjac: &'a mut [f64],
    /// `errmax`.
    pub errmax: &'a mut [f64],
    /// `errmoy`.
    pub errmoy: &'a mut [f64],
    /// `ndegpu`.
    pub ndegpu: i32,
    /// `ndegpv`.
    pub ndegpv: i32,
    /// `itydec`.
    pub itydec: i32,
}

/// Approximates the sampled square by a Jacobi patch and decides whether to cut
/// (`mma2ce1_`): the Gauss tables and Jacobi maxima are computed, then `mma2ce2_`
/// projects and truncates. `iercod`: 0 done, -1 when the tolerance is not reached
/// without cutting, 1 for inconsistent degrees, the `mmapptt_` code otherwise;
/// returns 0. `mma2ce1_(&request, &samples, &mut output, &mut ier)`.
pub fn mma2ce1_(
    request: &PatchApproxRequest<'_>,
    samples: &SquareSamples<'_>,
    output: &mut PatchApproxOutput<'_>,
    iercod: &mut i32,
) -> i32 {
    *iercod = 0;
    let r = request;
    let ncu: i32 = r.ndjacu - ((r.iordru + 1) << 1) + 1;
    let ncv: i32 = r.ndjacv - ((r.iordrv + 1) << 1) + 1;
    let mut gssutb: Vec<f64> = vec![0.; ((r.nbpntu / 2 + 1) * ncu).max(0) as usize];
    let mut gssvtb: Vec<f64> = vec![0.; ((r.nbpntv / 2 + 1) * ncv).max(0) as usize];
    mmapptt_(r.ndjacu, r.nbpntu, r.iordru, &mut gssutb, iercod);
    if *iercod > 0 {
        return 0;
    }
    mmapptt_(r.ndjacv, r.nbpntv, r.iordrv, &mut gssvtb, iercod);
    if *iercod > 0 {
        return 0;
    }
    let (mut xmaxju, mut xmaxjv) = (vec![0.; ncu.max(0) as usize], vec![0.; ncv.max(0) as usize]);
    mma2jmx_(r.ndjacu, r.iordru, &mut xmaxju);
    mma2jmx_(r.ndjacv, r.iordrv, &mut xmaxjv);
    let mut state = Ce2State {
        r,
        s: samples,
        gssutb: &gssutb,
        gssvtb: &gssvtb,
        xmaxju: &xmaxju,
        xmaxjv: &xmaxjv,
        chpair: vec![0.; ((r.nbpntv / 2 + 1) * ncu * r.ndimen).max(0) as usize],
        chimpr: vec![0.; (r.nbpntv / 2 * ncu * r.ndimen).max(0) as usize],
        vecerr: vec![0.; (r.ndimen << 2) as usize],
    };
    mma2ce2_(&mut state, output, iercod);
    0
}

/// The working tables of `mma2ce2_`: the Gauss tables, the Jacobi maxima, the U
/// projections `chpair`/`chimpr` (`(0:nbpntv/2, ncu, ndimen)` and
/// `(nbpntv/2, ncu, ndimen)`) and the error columns `vecerr` (`(ndimen, 4)`).
pub(super) struct Ce2State<'a> {
    pub(super) r: &'a PatchApproxRequest<'a>,
    pub(super) s: &'a SquareSamples<'a>,
    pub(super) gssutb: &'a [f64],
    pub(super) gssvtb: &'a [f64],
    pub(super) xmaxju: &'a [f64],
    pub(super) xmaxjv: &'a [f64],
    pub(super) chpair: Vec<f64>,
    pub(super) chimpr: Vec<f64>,
    pub(super) vecerr: Vec<f64>,
}

impl Ce2State<'_> {
    /// `2 * (iordru + 1)` and `2 * (iordrv + 1)`: the first free Jacobi degrees.
    pub(super) fn i2rd(&self) -> (i32, i32) {
        ((self.r.iordru + 1) << 1, (self.r.iordrv + 1) << 1)
    }

    /// The column count `ncu` of the U projection tables.
    fn ncu(&self) -> i32 {
        self.r.ndjacu - ((self.r.iordru + 1) << 1) + 1
    }

    /// `mma2cfu_` for the degrees `kk_range` of dimension `nd`.
    pub(super) fn project_u(&mut self, nd: i32, kk_range: [i32; 2]) {
        let (nbpntu, nbpntv) = (self.r.nbpntu, self.r.nbpntv);
        let full: usize = ((nbpntu / 2 + 1) * (nbpntv / 2 + 1) * (nd - 1)) as usize;
        let half: usize = (nbpntu / 2 * (nbpntv / 2) * (nd - 1)) as usize;
        let block = SquareBlock {
            soso: &self.s.sosotb[full..],
            diso: &self.s.disotb[half..],
            sodi: &self.s.soditb[half..],
            didi: &self.s.diditb[full..],
        };
        let (chd1, cid1, ncu) = (nbpntv / 2 + 1, nbpntv / 2, self.ncu());
        let gdim1: i32 = nbpntu / 2 + 1;
        let i2rdu: i32 = self.i2rd().0;
        for kk in kk_range[0]..=kk_range[1] {
            let igsu: i32 = kk - i2rdu;
            let pair: usize = ((igsu + (nd - 1) * ncu) * chd1) as usize;
            let odd: usize = ((igsu + (nd - 1) * ncu) * cid1) as usize;
            let gss: &[f64] = &self.gssutb[(igsu * gdim1) as usize..];
            mma2cfu_(
                kk,
                [nbpntu, nbpntv],
                &block,
                gss,
                &mut self.chpair[pair..],
                &mut self.chimpr[odd..],
            );
        }
    }

    /// `mma2cfv_` for degree `jj` in V, U degrees `[minu, maxu]`, dimension `nd`.
    pub(super) fn project_v(&self, patjac: &mut [f64], jj: i32, u_range: [i32; 2], nd: i32) {
        let nbpntv: i32 = self.r.nbpntv;
        let (i2rdu, i2rdv) = self.i2rd();
        let (chd1, cid1, ncu) = (nbpntv / 2 + 1, nbpntv / 2, self.ncu());
        let igsu: i32 = u_range[0] - i2rdu;
        let igsv: i32 = jj - i2rdv;
        let pair: usize = ((igsu + (nd - 1) * ncu) * chd1) as usize;
        let odd: usize = ((igsu + (nd - 1) * ncu) * cid1) as usize;
        let (pd1, pd2) = (self.r.ndjacu + 1, self.r.ndjacv + 1);
        let start: usize = (u_range[0] + jj * pd1 + (nd - 1) * pd1 * pd2) as usize;
        let gss: &[f64] = &self.gssvtb[(igsv * (nbpntv / 2 + 1)) as usize..];
        mma2cfv_(
            jj,
            u_range,
            nbpntv,
            gss,
            &self.chpair[pair..],
            &self.chimpr[odd..],
            &mut patjac[start..],
        );
    }

    /// `mma2er1_` over `range` for each subspace, accumulating into column
    /// `column` (2..=4) of `vecerr`, entry per subspace.
    pub(super) fn error_columns(&mut self, patjac: &[f64], range: [i32; 4], column: usize) {
        let ndimen: usize = self.r.ndimen as usize;
        let mut idim: i32 = 1;
        for nd in 1..=self.r.nbsesp {
            let ndses: i32 = self.r.ndimse[(nd - 1) as usize];
            let block = subspace_block(self.r, [self.xmaxju, self.xmaxjv], patjac, idim, ndses);
            let (scratch, rest) = self.vecerr.split_at_mut(ndimen);
            let mut erreur: f64 = rest[(column - 2) * ndimen + (nd - 1) as usize];
            mma2er1_(&block, range, scratch, &mut erreur);
            self.vecerr[(column - 1) * ndimen + (nd - 1) as usize] = erreur;
            idim += ndses;
        }
    }

    /// The `vecerr(nd, column)` entry.
    pub(super) fn vecerr_at(&self, nd: i32, column: usize) -> f64 {
        self.vecerr[(column - 1) * self.r.ndimen as usize + (nd - 1) as usize]
    }
}

/// The Jacobi block of the subspace starting at dimension `idim` (1-based) of
/// `patjac`, with the maxima `xmaxj = [xmaxju, xmaxjv]`.
pub(super) fn subspace_block<'b>(
    r: &PatchApproxRequest<'_>,
    xmaxj: [&'b [f64]; 2],
    patjac: &'b [f64],
    idim: i32,
    ndses: i32,
) -> JacobiBlock<'b> {
    let (pd1, pd2) = (r.ndjacu + 1, r.ndjacv + 1);
    JacobiBlock {
        ndjac: [r.ndjacu, r.ndjacv],
        ndimen: ndses,
        iordr: [r.iordru, r.iordrv],
        xmaxju: xmaxj[0],
        xmaxjv: xmaxj[1],
        patjac: &patjac[((idim - 1) * pd1 * pd2) as usize..],
    }
}
