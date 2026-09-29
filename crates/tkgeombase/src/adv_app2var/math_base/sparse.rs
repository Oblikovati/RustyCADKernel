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

//! Profile (skyline) matrices: `mmposui_`, `mmresol_` and the file-static
//! `mmchole_`, `mmrslss_`, `mmatvec_`, `mmtmave_`.
//!
//! A symmetric matrix keeps, per row `i`, the `aposit(1,i)` entries left of the
//! diagonal and the storage position `aposit(2,i)` of the diagonal; `posuiv(k)` is
//! the next row that has an entry in the column of stored entry `k` (-1 if none).
//! A rectangular matrix keeps, per row, the count, the position of the last entry
//! and the first column (`gposit(1..3,i)`).
use crate::adv_app2var::f2c::{F2cArray, F2cArrayMut};
use crate::adv_app2var::sys_base::mvriraz_;

/// Builds the `posuiv` successor table of a profile matrix of order `dimmat`
/// described by `aposit` (`mmposui_`); `iercod` is 0; returns 0.
/// `mmposui_(3, 5, &aposit, &mut posuiv, &mut ier)`.
pub fn mmposui_(
    dimmat: i32,
    _nistoc: i32,
    aposit: &[i32],
    posuiv: &mut [i32],
    iercod: &mut i32,
) -> i32 {
    let aposit = F2cArray::new(aposit, 3);
    let mut posuiv = F2cArrayMut::new(posuiv, 1);
    *iercod = 0;
    for i in 1..=dimmat {
        let jmin: i32 = i - aposit[(i << 1) + 1];
        for j in jmin..=i {
            let mut imin: i32 = i + 1;
            let mut trouve: bool = false;
            while !trouve && imin <= dimmat {
                if imin - aposit[(imin << 1) + 1] <= j {
                    trouve = true;
                } else {
                    imin += 1;
                }
            }
            let k: i32 = aposit[(i << 1) + 2] - i + j;
            posuiv[k] = if trouve { imin } else { -1 };
        }
    }
    0
}

/// `mmchole_`: the Cholesky factor of the profile matrix `amatri` into `chomat`,
/// same profile; `iercod` is 1 when a pivot falls below 1e-32.
fn mmchole_(
    dimens: i32,
    amatri: &[f64],
    aposit: &[i32],
    posuiv: &[i32],
    chomat: &mut [f64],
    iercod: &mut i32,
) {
    let amatri = F2cArray::new(amatri, 1);
    let aposit = F2cArray::new(aposit, 3);
    let posuiv = F2cArray::new(posuiv, 1);
    let mut chomat = F2cArrayMut::new(chomat, 1);
    *iercod = 0;
    for j in 1..=dimens {
        let ptini: i32 = aposit[(j << 1) + 2];
        let mut somme: f64 = 0.;
        for k in ptini - aposit[(j << 1) + 1]..=ptini - 1 {
            let d: f64 = chomat[k];
            somme += d * d;
        }
        if amatri[ptini] - somme < 1e-32 {
            *iercod = 1;
            return;
        }
        chomat[ptini] = (amatri[ptini] - somme).sqrt();
        let mut ptcou: i32 = ptini;
        while posuiv[ptcou] > 0 {
            let i: i32 = posuiv[ptcou];
            ptcou = aposit[(i << 1) + 2] - (i - j);
            let mut somme: f64 = 0.;
            let kmin: i32 = (i - aposit[(i << 1) + 1]).max(j - aposit[(j << 1) + 1]);
            for k in kmin..=j - 1 {
                somme +=
                    chomat[aposit[(i << 1) + 2] - (i - k)] * chomat[aposit[(j << 1) + 2] - (j - k)];
            }
            chomat[ptcou] = (amatri[ptcou] - somme) / chomat[ptini];
        }
    }
}

/// `mmrslss_`: solves `S * S^T * x = b` for the Cholesky factor `smatri` by the
/// forward then the backward substitution; `iercod` is 0.
fn mmrslss_(
    dimens: i32,
    smatri: &[f64],
    sposit: &[i32],
    posuiv: &[i32],
    mscnmbr: &[f64],
    soluti: &mut [f64],
    iercod: &mut i32,
) {
    let smatri = F2cArray::new(smatri, 1);
    let sposit = F2cArray::new(sposit, 3);
    let posuiv = F2cArray::new(posuiv, 1);
    let mscnmbr = F2cArray::new(mscnmbr, 1);
    let mut sol = F2cArrayMut::new(soluti, 1);
    *iercod = 0;
    for i in 1..=dimens {
        let pointe: i32 = sposit[(i << 1) + 2];
        let mut somme: f64 = 0.;
        for j in i - sposit[(i << 1) + 1]..=i - 1 {
            somme += smatri[pointe - (i - j)] * sol[j];
        }
        sol[i] = (mscnmbr[i] - somme) / smatri[pointe];
    }
    for i in (1..=dimens).rev() {
        let pointe: i32 = sposit[(i << 1) + 2];
        let mut j: i32 = posuiv[pointe];
        let mut somme: f64 = 0.;
        while j > 0 {
            let ptcour: i32 = sposit[(j << 1) + 2] - (j - i);
            somme += smatri[ptcour] * sol[j];
            j = posuiv[ptcour];
        }
        sol[i] = (sol[i] - somme) / smatri[pointe];
    }
}

/// A rectangular profile matrix `(nligne, ncolon)` described by `gposit`.
struct ProfileMatrix<'a> {
    nligne: i32,
    gposit: F2cArray<'a, i32>,
    gmatri: F2cArray<'a, f64>,
}

impl ProfileMatrix<'_> {
    /// The first and last stored columns of row `i` and the shift from a column to
    /// its storage position.
    fn row_span(&self, i: i32) -> (i32, i32, i32) {
        let jmin: i32 = self.gposit[i * 3 + 3];
        let jmax: i32 = self.gposit[i * 3 + 1] + self.gposit[i * 3 + 3] - 1;
        let aux: i32 = self.gposit[i * 3 + 2] - self.gposit[i * 3 + 1] - jmin + 1;
        (jmin, jmax, aux)
    }

    /// `mmatvec_`: `vecout = G * vecin` for the rows from `deblig`, the others zero.
    fn mmatvec_(&self, vecin: &[f64], deblig: i32, vecout: &mut [f64]) {
        let vecin = F2cArray::new(vecin, 1);
        mvriraz_(self.nligne, vecout);
        let mut vecout = F2cArrayMut::new(vecout, 1);
        for i in deblig..=self.nligne {
            let (jmin, jmax, aux) = self.row_span(i);
            let mut somme: f64 = 0.;
            for j in jmin..=jmax {
                somme += self.gmatri[j + aux] * vecin[j];
            }
            vecout[i] = somme;
        }
    }

    /// `mmtmave_`: `vecout = G^T * vecin` for the `ncolon` columns.
    fn mmtmave_(&self, ncolon: i32, vecin: &[f64], vecout: &mut [f64]) {
        let vecin = F2cArray::new(vecin, 1);
        let mut vecout = F2cArrayMut::new(vecout, 1);
        for i in 1..=ncolon {
            let mut somme: f64 = 0.;
            for j in 1..=self.nligne {
                let (imin, imax, aux) = self.row_span(j);
                if imin <= i && i <= imax {
                    somme += self.gmatri[i + aux] * vecin[j];
                }
            }
            vecout[i] = somme;
        }
    }
}

/// The profile systems of `mmresol_`: the symmetric `H` (order `hdimen`), the
/// rectangular constraint matrix `G` (`gdimen` rows) and the profile of `G H^-1 G^T`.
pub struct ProfileSystem<'a> {
    /// `hdimen`: order of `H`.
    pub hdimen: i32,
    /// `gdimen`: number of constraint rows (0 for none).
    pub gdimen: i32,
    /// `hnstoc`: storage size of `H`; the native routines never read it.
    pub hnstoc: i32,
    /// `gnstoc`: storage size of `G`; the native routines never read it.
    pub gnstoc: i32,
    /// `mnstoc`: storage size of `G H^-1 G^T`.
    pub mnstoc: i32,
    /// `matsyh`: the profile entries of `H`.
    pub matsyh: &'a [f64],
    /// `matsyg`: the profile entries of `G`.
    pub matsyg: &'a [f64],
    /// `vecsyh`: the right-hand side of the `H` block.
    pub vecsyh: &'a [f64],
    /// `vecsyg`: the right-hand side of the constraints.
    pub vecsyg: &'a [f64],
    /// `hposit`: profile of `H`, `(2, hdimen)`.
    pub hposit: &'a [i32],
    /// `hposui`: successor table of `H`.
    pub hposui: &'a [i32],
    /// `gposit`: profile of `G`, `(3, gdimen)`.
    pub gposit: &'a [i32],
    /// `mmposui`: successor table of `G H^-1 G^T`.
    pub mmposui: &'a [i32],
    /// `mposit`: profile of `G H^-1 G^T`, `(2, gdimen)`.
    pub mposit: &'a [i32],
}

/// Solves the saddle-point system `[H G^T; G 0] (x, l) = (vecsyh, vecsyg)` by
/// Cholesky factorisations of `H` and of `G H^-1 G^T` (`mmresol_`); `vecsol`
/// receives `x`. `iercod` is 1 when a factorisation fails, 2 for a storage
/// problem; returns 0. The sixteen native arguments are the fields of
/// [`ProfileSystem`], in the native order.
///
/// Natively `mmresol_` dereferences a null pointer when a scratch vector fits in
/// its 100-entry threshold (`macrar8_` then allocates nothing and the f2c code
/// dropped the Fortran local arrays); the port always uses owned scratch vectors,
/// which is what the Fortran declared. `mmresol_(&system, &mut sol, &mut ier)`.
pub fn mmresol_(system: &ProfileSystem<'_>, vecsol: &mut [f64], iercod: &mut i32) -> i32 {
    *iercod = match resolve(system, vecsol) {
        Ok(()) => 0,
        Err(code) => code,
    };
    0
}

fn resolve(s: &ProfileSystem<'_>, vecsol: &mut [f64]) -> Result<(), i32> {
    let (hdimen, gdimen) = (s.hdimen, s.gdimen);
    let hposit = F2cArray::new(s.hposit, 3);
    let dimhch: i32 = hposit[(hdimen << 1) + 2];
    let mut v1: Vec<f64> = vec![0.; hdimen.max(0) as usize];
    let mut hchole: Vec<f64> = vec![0.; dimhch.max(0) as usize];
    let mut ier: i32 = 0;
    mmchole_(hdimen, s.matsyh, s.hposit, s.hposui, &mut hchole, &mut ier);
    fail_if(ier, 1)?;
    mmrslss_(
        hdimen, &hchole, s.hposit, s.hposui, s.vecsyh, &mut v1, &mut ier,
    );
    fail_if(ier, 2)?;
    if gdimen <= 0 {
        vecsol[..hdimen as usize].copy_from_slice(&v1);
        return Ok(());
    }
    let g = ProfileMatrix {
        nligne: gdimen,
        gposit: F2cArray::new(s.gposit, 4),
        gmatri: F2cArray::new(s.matsyg, 1),
    };
    let mut v2: Vec<f64> = vec![0.; gdimen as usize];
    g.mmatvec_(&v1, 1, &mut v2);
    for i in 1..=gdimen as usize {
        v2[i - 1] -= s.vecsyg[i - 1];
    }
    let matsym: Vec<f64> = schur_complement(s, &g, &hchole)?;
    let mut v4: Vec<f64> = vec![0.; gdimen as usize];
    let mut mcho: Vec<f64> = vec![0.; s.mnstoc.max(0) as usize];
    mmchole_(gdimen, &matsym, s.mposit, s.mmposui, &mut mcho, &mut ier);
    fail_if(ier, 1)?;
    mmrslss_(gdimen, &mcho, s.mposit, s.mmposui, &v2, &mut v4, &mut ier);
    fail_if(ier, 2)?;
    mvriraz_(hdimen, &mut v1);
    g.mmtmave_(hdimen, &v4, &mut v1);
    for i in 1..=hdimen as usize {
        v1[i - 1] = s.vecsyh[i - 1] - v1[i - 1];
    }
    mmrslss_(hdimen, &hchole, s.hposit, s.hposui, &v1, vecsol, &mut ier);
    fail_if(ier, 2)
}

fn fail_if(ier: i32, code: i32) -> Result<(), i32> {
    if ier > 0 { Err(code) } else { Ok(()) }
}

/// The profile entries of `G H^-1 G^T`, one row per constraint (the `i` loop of
/// `mmresol_`), from the Cholesky factor `hchole` of `H`.
fn schur_complement(
    s: &ProfileSystem<'_>,
    g: &ProfileMatrix<'_>,
    hchole: &[f64],
) -> Result<Vec<f64>, i32> {
    let (hdimen, gdimen) = (s.hdimen, s.gdimen);
    let mposit = F2cArray::new(s.mposit, 3);
    let mmposui = F2cArray::new(s.mmposui, 1);
    let mut matsym: Vec<f64> = vec![0.; s.mnstoc.max(0) as usize];
    let mut sym = F2cArrayMut::new(&mut matsym, 1);
    let (mut v1, mut v3) = (vec![0.; hdimen as usize], vec![0.; hdimen as usize]);
    let mut v4: Vec<f64> = vec![0.; gdimen as usize];
    let mut ier: i32 = 0;
    for i in 1..=gdimen {
        mvriraz_(hdimen, &mut v1);
        mvriraz_(hdimen, &mut v3);
        mvriraz_(gdimen, &mut v4);
        let (jmin, jmax, aux) = g.row_span(i);
        for j in jmin..=jmax {
            v1[(j - 1) as usize] = g.gmatri[j + aux];
        }
        mmrslss_(hdimen, hchole, s.hposit, s.hposui, &v1, &mut v3, &mut ier);
        fail_if(ier, 1)?;
        g.mmatvec_(&v3, i, &mut v4);
        let mut k: i32 = mposit[(i << 1) + 2];
        sym[k] = v4[(i - 1) as usize];
        while mmposui[k] > 0 {
            let l: i32 = mmposui[k];
            k = mposit[(l << 1) + 2] - l + i;
            sym[k] = v4[(l - 1) as usize];
        }
    }
    Ok(matsym)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tridiagonal_system_with_one_constraint() {
        // H = [[2,-1,0],[-1,2,-1],[0,-1,2]] in profile form, G = [1 1 1], b = (1,0,1), c = 3.
        let hposit: [i32; 6] = [0, 1, 1, 3, 1, 5];
        let (mut hposui, mut ier) = ([0; 5], 9);
        mmposui_(3, 5, &hposit, &mut hposui, &mut ier);
        assert_eq!(hposui, [2, -1, 3, -1, -1]);
        let mposit: [i32; 2] = [0, 1];
        let mut mmposui: [i32; 1] = [0];
        mmposui_(1, 1, &mposit, &mut mmposui, &mut ier);
        let system = ProfileSystem {
            hdimen: 3,
            gdimen: 1,
            hnstoc: 5,
            gnstoc: 3,
            mnstoc: 1,
            matsyh: &[2.0, -1.0, 2.0, -1.0, 2.0],
            matsyg: &[1.0, 1.0, 1.0],
            vecsyh: &[1.0, 0.0, 1.0],
            vecsyg: &[3.0],
            hposit: &hposit,
            hposui: &hposui,
            gposit: &[3, 3, 1],
            mmposui: &mmposui,
            mposit: &mposit,
        };
        let mut sol: [f64; 3] = [0.0; 3];
        mmresol_(&system, &mut sol, &mut ier);
        assert_eq!(ier, 0);
        assert!((sol.iter().sum::<f64>() - 3.0).abs() < 1e-12);
        assert!((sol[0] - sol[2]).abs() < 1e-12);
    }
}
