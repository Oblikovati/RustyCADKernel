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

//! Sorting and searching of column tables: `mmbulld_`, `mvsheld_`, `mminltt_` and
//! `mmsrre2_`.
#![allow(clippy::neg_cmp_op_on_partial_ord)] // Negated native tests keep the NaN branches.
use crate::adv_app2var::f2c::{F2cArray, F2cArrayMut};
use crate::adv_app2var::sys_base::{mcrfill_, mcrfill_within};

/// Swaps columns `a` and `b` (rows `1..=rows`) of a column-major view.
fn swap_columns(table: &mut F2cArrayMut<'_, f64>, rows: i32, dim1: i32, a: i32, b: i32) {
    for row in 1..=rows {
        let daux: f64 = table[row + a * dim1];
        table[row + a * dim1] = table[row + b * dim1];
        table[row + b * dim1] = daux;
    }
}

/// Sorts the `nbcoln` columns of `dtabtr` (`(nblign, nbcoln)`) by increasing row
/// `numcle` with the bidirectional bubble sort of the native code (`mmbulld_`);
/// returns 0. `mmbulld_(4, 2, &mut table, 1)`.
pub fn mmbulld_(nbcoln: i32, nblign: i32, dtabtr: &mut [f64], numcle: i32) -> i32 {
    let dim1: i32 = nblign;
    let mut tab = F2cArrayMut::new(dtabtr, dim1 + 1);
    let (mut nite1, mut nite2) = (nbcoln, 2);
    let mut nchan: bool = true;
    while nchan {
        nchan = false;
        for i1 in nite2..=nite1 {
            if tab[numcle + i1 * dim1] < tab[numcle + (i1 - 1) * dim1] {
                swap_columns(&mut tab, nblign, dim1, i1 - 1, i1);
                nchan = true;
            }
        }
        nite1 -= 1;
        if nchan {
            nchan = false;
            for i1 in (nite2..=nite1).rev() {
                if tab[numcle + i1 * dim1] < tab[numcle + (i1 - 1) * dim1] {
                    swap_columns(&mut tab, nblign, dim1, i1 - 1, i1);
                    nchan = true;
                }
            }
            nite2 += 1;
        }
    }
    0
}

/// Sorts the `n` columns of `dtab` (`(is, n)`) by increasing row `icle` with the
/// native Shell sort (gaps 1, 4, 13, ... below `n/9`) (`mvsheld_`); returns 0.
/// `mvsheld_(5, 2, &mut table, 1)`.
pub fn mvsheld_(n: i32, is: i32, dtab: &mut [f64], icle: i32) -> i32 {
    if n <= 1 {
        return 0;
    }
    let dim1: i32 = is;
    let mut tab = F2cArrayMut::new(dtab, dim1 + 1);
    let mut incr: i32 = 1;
    while incr < n / 9 {
        incr = incr * 3 + 1;
    }
    loop {
        for i3 in incr + 1..=n {
            let mut i4: i32 = i3 - incr;
            // The native test is `<=` then skip, so a NaN key still swaps.
            while i4 >= 1 && !(tab[icle + i4 * dim1] <= tab[icle + (i4 + incr) * dim1]) {
                swap_columns(&mut tab, is, dim1, i4, i4 + incr);
                i4 -= incr;
            }
        }
        incr /= 3;
        if incr < 1 {
            return 0;
        }
    }
}

/// Inserts the line `ajoute` (`nbrcol` values) into the lexicographically sorted
/// table `tabtri` (`(ncolmx, nlgnmx)`, `nbrlgn` lines used) unless an equal line
/// is already there (`mminltt_`); `nbrlgn` grows by one on insertion. `iercod` is
/// 1 when the table is full; returns 0. The tolerance `epsega` is unused natively.
/// `mminltt_(2, 4, &mut table, 2, &mut used, &[1.0, 2.0], 0.0, &mut ier)`.
pub fn mminltt_(
    ncolmx: i32,
    nlgnmx: i32,
    tabtri: &mut [f64],
    nbrcol: i32,
    nbrlgn: &mut i32,
    ajoute: &[f64],
    _epsega: f64,
    iercod: &mut i32,
) -> i32 {
    if *nbrlgn >= nlgnmx {
        *iercod = 1;
        return 0;
    }
    *iercod = 0;
    let dim1: i32 = ncolmx;
    let ajoute = F2cArray::new(ajoute, 1);
    let table = F2cArray::new(tabtri, dim1 + 1);
    if contains_line(&table, dim1, *nbrlgn, nbrcol, &ajoute) {
        return 0;
    }
    let inser: i32 = insertion_line(&table, dim1, *nbrlgn, nbrcol, &ajoute);
    *nbrlgn += 1;
    let nlgn: i32 = *nbrlgn - inser;
    let start: usize = ((inser - 1) * dim1) as usize;
    if nlgn > 0 {
        mcrfill_within((ncolmx << 3) * nlgn, tabtri, start, start + dim1 as usize);
    }
    mcrfill_(nbrcol << 3, ajoute.tail(1), &mut tabtri[start..]);
    0
}

/// The duplicate search of `mminltt_`: lines are scanned while their first value
/// does not exceed the new one, and a line equal in every column stops the insertion.
fn contains_line(
    table: &F2cArray<'_, f64>,
    dim1: i32,
    nbrlgn: i32,
    nbrcol: i32,
    ajoute: &F2cArray<'_, f64>,
) -> bool {
    let epsega: f64 = 0.;
    for ilgn in 1..=nbrlgn {
        // The native `>=` and `<=` tests, negated, so NaN values branch as natively.
        if !(table[ilgn * dim1 + 1] >= ajoute[1] - epsega) {
            continue;
        }
        if !(table[ilgn * dim1 + 1] <= ajoute[1] + epsega) {
            return false;
        }
        let differs = (1..=nbrcol).any(|icol| {
            table[icol + ilgn * dim1] < ajoute[icol] - epsega
                || table[icol + ilgn * dim1] > ajoute[icol] + epsega
        });
        if !differs {
            return true;
        }
    }
    false
}

/// The insertion line of `mminltt_`: the first line greater than `ajoute` in the
/// first differing column, or the line after the last one (labels L30 to L70).
fn insertion_line(
    table: &F2cArray<'_, f64>,
    dim1: i32,
    nbrlgn: i32,
    nbrcol: i32,
    ajoute: &F2cArray<'_, f64>,
) -> i32 {
    for ilgn in 1..=nbrlgn {
        for icol in 1..=nbrcol {
            if table[icol + ilgn * dim1] < ajoute[icol] {
                break;
            }
            if table[icol + ilgn * dim1] > ajoute[icol] {
                return ilgn;
            }
        }
    }
    nbrlgn + 1
}

/// Locates `tparam` among the `nbrval` increasing values `tablev` by bisection
/// (`mmsrre2_`): `numint` is the interval index and `itypen` 1 or 2 when `tparam`
/// is within `epsil` of the interval start or end. `iercod` is 1 for fewer than 2
/// values, 2 outside the table; returns 0.
/// `mmsrre2_(0.5, 3, &[0.0, 1.0, 2.0], 1e-9, &mut n, &mut kind, &mut ier)`.
pub fn mmsrre2_(
    tparam: f64,
    nbrval: i32,
    tablev: &[f64],
    epsil: f64,
    numint: &mut i32,
    itypen: &mut i32,
    iercod: &mut i32,
) -> i32 {
    let tablev = F2cArray::new(tablev, 1);
    *iercod = 0;
    *numint = 0;
    *itypen = 0;
    if nbrval < 2 {
        *iercod = 1;
        return 0;
    }
    if tparam < tablev[1] || tparam > tablev[nbrval] {
        *iercod = 2;
        return 0;
    }
    let (mut ideb, mut ifin) = (1, nbrval);
    while ideb + 1 != ifin {
        let imil: i32 = (ideb + ifin) / 2;
        if tparam >= tablev[ideb] && tparam <= tablev[imil] {
            ifin = imil;
        } else {
            ideb = imil;
        }
    }
    *numint = ideb;
    if (tparam - tablev[ideb]).abs() < epsil {
        *itypen = 1;
    } else if (tparam - tablev[ifin]).abs() < epsil {
        *itypen = 2;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_sorts_order_columns_by_the_key_row() {
        let mut table: Vec<f64> = vec![3.0, 30.0, 1.0, 10.0, 2.0, 20.0];
        mmbulld_(3, 2, &mut table, 1);
        assert_eq!(table, [1.0, 10.0, 2.0, 20.0, 3.0, 30.0]);
        let mut table: Vec<f64> = (0..12).rev().map(f64::from).collect();
        mvsheld_(12, 1, &mut table, 1);
        assert_eq!(table, (0..12).map(f64::from).collect::<Vec<f64>>());
    }

    #[test]
    fn lines_are_inserted_in_order_once() {
        let mut table: Vec<f64> = vec![0.0; 8];
        let (mut used, mut ier) = (0, 9);
        for line in [[2.0, 1.0], [1.0, 5.0], [2.0, 0.5], [1.0, 5.0]] {
            mminltt_(2, 4, &mut table, 2, &mut used, &line, 0.0, &mut ier);
        }
        assert_eq!((used, ier), (3, 0));
        assert_eq!(table[..6], [1.0, 5.0, 2.0, 0.5, 2.0, 1.0]);
        used = 4;
        mminltt_(2, 4, &mut table, 2, &mut used, &[9.0, 9.0], 0.0, &mut ier);
        assert_eq!(ier, 1);
    }

    #[test]
    fn bisection_finds_the_interval_and_its_ends() {
        let (mut n, mut kind, mut ier) = (0, 0, 0);
        mmsrre2_(1.0, 3, &[0.0, 1.0, 2.0], 1e-9, &mut n, &mut kind, &mut ier);
        assert_eq!((n, kind, ier), (1, 2, 0));
        mmsrre2_(3.0, 3, &[0.0, 1.0, 2.0], 1e-9, &mut n, &mut kind, &mut ier);
        assert_eq!(ier, 2);
    }
}
