// Copyright (c) 1998-1999 Matra Datavision
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

//! The two native protection algebras: Unix reads the hand-written `TabProtAdd` and
//! `TabProtSub` tables (indexed [current][right] by `OSD_SingleProtection`), Windows
//! combines read/write/execute/delete masks.
use crate::osd::SingleProtection;

// TabProtAdd from OSD_Protection.cxx, as SingleProtection discriminants. Kept verbatim:
// cells [WX][W] = RWX and [RWX][D] = RWX are native quirks, not unions.
const UNIX_ADD: [[u8; 16]; 16] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [1, 1, 3, 3, 5, 5, 7, 7, 9, 9, 11, 11, 13, 13, 15, 15],
    [2, 3, 2, 3, 6, 7, 6, 7, 10, 11, 10, 11, 14, 15, 14, 15],
    [3, 3, 3, 3, 7, 7, 7, 7, 11, 11, 11, 11, 15, 15, 15, 15],
    [4, 5, 6, 7, 4, 5, 6, 7, 12, 13, 14, 15, 12, 13, 14, 15],
    [5, 5, 7, 7, 5, 5, 7, 7, 13, 13, 15, 15, 13, 13, 15, 15],
    [6, 7, 7, 7, 6, 7, 6, 7, 14, 15, 14, 15, 14, 15, 14, 15],
    [7, 7, 7, 7, 7, 7, 7, 7, 7, 15, 15, 15, 15, 15, 15, 15],
    [8, 9, 10, 11, 12, 13, 14, 15, 8, 9, 10, 11, 12, 13, 14, 15],
    [9, 9, 11, 11, 13, 13, 15, 15, 9, 9, 11, 11, 13, 13, 15, 15],
    [
        10, 11, 10, 11, 14, 15, 14, 15, 10, 11, 10, 11, 14, 15, 14, 15,
    ],
    [
        11, 11, 11, 11, 15, 15, 15, 15, 11, 11, 11, 11, 15, 15, 15, 15,
    ],
    [
        12, 13, 14, 15, 12, 13, 14, 15, 12, 13, 14, 15, 12, 13, 14, 15,
    ],
    [
        13, 13, 15, 15, 13, 13, 15, 15, 13, 13, 15, 15, 13, 13, 15, 15,
    ],
    [
        14, 15, 14, 15, 14, 15, 14, 15, 14, 15, 14, 15, 14, 15, 14, 15,
    ],
    [
        15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    ],
];

// TabProtSub from OSD_Protection.cxx. Native quirks in the X row: [X][RW] = None,
// [X][RWX] = X and [X][RWD] = None, where a bitwise difference gives X, None, X.
const UNIX_SUB: [[u8; 16]; 16] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0],
    [2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0],
    [3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0],
    [4, 4, 4, 0, 0, 0, 0, 4, 4, 4, 4, 0, 0, 0, 0, 0],
    [5, 4, 5, 4, 1, 0, 1, 0, 5, 4, 5, 4, 1, 0, 1, 0],
    [6, 6, 4, 4, 2, 2, 0, 0, 6, 6, 4, 4, 2, 2, 0, 0],
    [7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0],
    [8, 8, 8, 8, 8, 8, 8, 8, 0, 0, 0, 0, 0, 0, 0, 0],
    [9, 8, 9, 8, 9, 8, 9, 8, 1, 0, 1, 0, 1, 0, 1, 0],
    [10, 10, 8, 8, 10, 10, 8, 8, 2, 2, 0, 0, 2, 2, 0, 0],
    [11, 10, 9, 8, 11, 10, 9, 8, 3, 2, 1, 0, 3, 2, 1, 0],
    [12, 12, 12, 12, 8, 8, 8, 8, 4, 4, 4, 4, 0, 0, 0, 0],
    [13, 12, 13, 12, 9, 8, 9, 8, 5, 4, 5, 4, 1, 0, 1, 0],
    [14, 14, 12, 12, 10, 10, 8, 8, 6, 6, 4, 4, 2, 2, 0, 0],
    [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0],
];

fn from_cell(cell: u8) -> SingleProtection {
    SingleProtection::ALL[usize::from(cell)]
}

pub(super) fn unix_add(current: SingleProtection, right: SingleProtection) -> SingleProtection {
    from_cell(UNIX_ADD[current as usize][right as usize])
}

pub(super) fn unix_sub(current: SingleProtection, right: SingleProtection) -> SingleProtection {
    from_cell(UNIX_SUB[current as usize][right as usize])
}

// The Windows _get_mask/_get_prot pair is the identity on the enum's bit layout
// (R=1, W=2, X=4, D=8), so the mask algebra is plain bit arithmetic on discriminants.
pub(super) fn windows_add(current: SingleProtection, right: SingleProtection) -> SingleProtection {
    from_cell((current as u8) | (right as u8))
}

pub(super) fn windows_sub(current: SingleProtection, right: SingleProtection) -> SingleProtection {
    from_cell((current as u8) & !(right as u8))
}

/// Unix `S_I[RWX](USR|GRP|OTH)` bits of one right placed at `shift` (6 user, 3 group,
/// 0 other); delete implies write, as in `OSD_Protection::Internal`.
#[allow(dead_code)]
pub(super) const fn mode_bits(right: SingleProtection, shift: u32) -> u32 {
    let bits: u32 = right as u32;
    let mut mode: u32 = 0;
    if bits & 1 != 0 {
        mode |= 4;
    }
    if bits & (2 | 8) != 0 {
        mode |= 2;
    }
    if bits & 4 != 0 {
        mode |= 1;
    }
    mode << shift
}

#[cfg(test)]
mod tests {
    use super::*;
    use SingleProtection::*;
    #[test]
    fn unix_tables_differ_from_bit_algebra_only_in_the_native_quirk_cells() {
        let mut add_quirks: Vec<(SingleProtection, SingleProtection)> = Vec::new();
        let mut sub_quirks: Vec<(SingleProtection, SingleProtection)> = Vec::new();
        for &a in SingleProtection::ALL {
            for &b in SingleProtection::ALL {
                if unix_add(a, b) != windows_add(a, b) {
                    add_quirks.push((a, b));
                }
                if unix_sub(a, b) != windows_sub(a, b) {
                    sub_quirks.push((a, b));
                }
            }
        }
        assert_eq!(add_quirks, vec![(Wx, W), (Rwx, D)]);
        assert_eq!(sub_quirks, vec![(X, Rw), (X, Rwx), (X, Rwd)]);
        assert_eq!(unix_sub(X, Rwx), X);
    }
    #[test]
    fn mode_bits_fold_delete_into_write() {
        assert_eq!(mode_bits(Rwd, 6), 0o600);
        assert_eq!(mode_bits(D, 3), 0o020);
        assert_eq!(mode_bits(Rwxd, 0), 0o007);
        assert_eq!(mode_bits(None, 6), 0);
    }
}
