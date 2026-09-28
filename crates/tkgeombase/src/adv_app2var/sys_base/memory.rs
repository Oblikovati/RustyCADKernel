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

//! The memory fills of `AdvApp2Var_SysBase`: `mcrfill_`, `msifill_`, `msrfill_`,
//! `maitbr8_`, `miraz_` and `mvriraz_`.
//!
//! The native routines take `void*` and a byte count. Here they take typed slices
//! and keep the byte count, so a caller passes the same `n << 3` the f2c code
//! computes; the count must be a whole number of elements.

/// The element count of a native byte count; a partial element would copy half a
/// value natively and is refused here with the offending count.
fn elements<T>(bytes: i32) -> usize {
    let size: usize = std::mem::size_of::<T>();
    let bytes: usize = usize::try_from(bytes).unwrap_or(0);
    assert!(
        bytes.is_multiple_of(size),
        "mcrfill_ byte count {bytes} is not a multiple of the element size {size}"
    );
    bytes / size
}

/// Copies `size` bytes of `tin` into `tout` (`mcrfill_`); returns 0. The native
/// routine also accepts overlapping zones, which [`mcrfill_within`] covers.
/// `mcrfill_(16, &[1.0, 2.0], &mut out)`.
pub fn mcrfill_<T: Copy>(size: i32, tin: &[T], tout: &mut [T]) -> i32 {
    let count: usize = elements::<T>(size);
    tout[..count].copy_from_slice(&tin[..count]);
    0
}

/// `mcrfill_` between two zones of one array that may overlap: the native byte loop
/// runs forward when the source follows the target and backward otherwise, which
/// is what `copy_within` does; `mcrfill_within(16, &mut tab, 3, 1)`.
pub fn mcrfill_within<T: Copy>(size: i32, data: &mut [T], from: usize, to: usize) -> i32 {
    let count: usize = elements::<T>(size);
    data.copy_within(from..from + count, to);
    0
}

/// Copies `nbintg` integers (`msifill_`); returns 0. `msifill_(2, &[1, 2], &mut out)`.
pub fn msifill_(nbintg: i32, ivecin: &[i32], ivecou: &mut [i32]) -> i32 {
    mcrfill_(nbintg * 4, ivecin, ivecou)
}

/// Copies `nbreel` doubles (`msrfill_`); returns 0. `msrfill_(2, &[1.0, 2.0], &mut out)`.
pub fn msrfill_(nbreel: i32, vecent: &[f64], vecsor: &mut [f64]) -> i32 {
    mcrfill_(nbreel * 8, vecent, vecsor)
}

/// Sets the first `itaill` doubles of `xtab` to `xval` (`maitbr8_`), by blocks of 63
/// copied from a local buffer as natively; returns 0. `maitbr8_(3, &mut tab, 1.5)`.
pub fn maitbr8_(itaill: i32, xtab: &mut [f64], xval: f64) -> i32 {
    // The native zero branch copies a zero buffer; the value branch fills one first.
    let buffer: [f64; 63] = [if xval == 0. { 0. } else { xval }; 63];
    let nbfois: i32 = itaill / 63;
    let noffst: i32 = nbfois * 63;
    let nreste: i32 = itaill - noffst;
    for nufois in 1..=nbfois {
        let start: usize = ((nufois - 1) * 63) as usize;
        mcrfill_(504, &buffer, &mut xtab[start..]);
    }
    if nreste >= 1 {
        mcrfill_(nreste << 3, &buffer, &mut xtab[noffst as usize..]);
    }
    0
}

/// Zeroes the first `taille` bytes (`miraz_`); `miraz_(16, &mut values)`.
pub fn miraz_<T: Copy + Default>(taille: i32, adt: &mut [T]) {
    adt[..elements::<T>(taille)].fill(T::default());
}

/// Zeroes the first `taille` doubles (`mvriraz_`); `mvriraz_(2, &mut values)`.
pub fn mvriraz_(taille: i32, adt: &mut [f64]) {
    miraz_(taille * 8, adt);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_copy_and_zero_whole_elements() {
        let mut out: [f64; 3] = [9.0; 3];
        mcrfill_(16, &[1.0, 2.0], &mut out);
        assert_eq!(out, [1.0, 2.0, 9.0]);
        mvriraz_(1, &mut out);
        assert_eq!(out, [0.0, 2.0, 9.0]);
        let mut ints: [i32; 2] = [0; 2];
        msifill_(2, &[4, 5], &mut ints);
        assert_eq!(ints, [4, 5]);
        miraz_(4, &mut ints);
        assert_eq!(ints, [0, 5]);
    }

    #[test]
    fn overlapping_fills_move_like_memmove() {
        let mut values: [f64; 4] = [1.0, 2.0, 3.0, 4.0];
        mcrfill_within(24, &mut values, 0, 1);
        assert_eq!(values, [1.0, 1.0, 2.0, 3.0]);
    }

    #[test]
    fn maitbr8_fills_blocks_and_the_remainder() {
        let mut values: Vec<f64> = vec![0.0; 130];
        maitbr8_(128, &mut values, 2.5);
        assert!(values[..128].iter().all(|v| *v == 2.5));
        assert_eq!(values[128], 0.0);
        maitbr8_(64, &mut values, 0.0);
        assert!(values[..64].iter().all(|v| *v == 0.0));
    }

    #[test]
    #[should_panic(expected = "mcrfill_ byte count 12 is not a multiple of the element size 8")]
    fn partial_elements_are_refused() {
        mcrfill_(12, &[1.0, 2.0], &mut [0.0; 2]);
    }
}
