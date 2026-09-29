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

//! The trace and configuration members of `AdvApp2Var_SysBase`.
//!
//! The f2c code keeps the Fortran calls to its message routines; OCCT stubbed their
//! Fortran I/O out, so they only return 0. The debug level `mnfndeb_` is always 0,
//! which keeps every `if (ibb >= n)` trace branch of the kernels dead.
use crate::adv_app2var::data::getmaovpar;

/// The Fortran blank common `MBLANK` that `macinit_` initialises: the reader and
/// printer units and the debug level `ibb`.
///
/// Natively it is a process global that nothing reads back; here it is caller
/// state. `let mut units = SysBaseUnits::default(); macinit_(&mut units, 0, 0);`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SysBaseUnits {
    /// Reader unit (`lec`).
    pub lec: i32,
    /// Printer unit (`imp`).
    pub imp: i32,
    /// Keyboard unit (`keyb`).
    pub keyb: i32,
    /// Error unit (`mae`).
    pub mae: i32,
    /// Screen unit (`jscrn`).
    pub jscrn: i32,
    /// Tablet unit (`itblt`).
    pub itblt: i32,
    /// Debug level (`ibb`).
    pub ibb: i32,
}

/// Initialises the units (`macinit_`): mode 0 sets the defaults `imp = 6`,
/// `ibb = 0`, `lec = 5`; modes 1, 2 and 3 force `imp`, `ibb` or `lec` to `ival`;
/// other modes change nothing. Returns 0. `macinit_(&mut units, 2, 1)`.
pub fn macinit_(units: &mut SysBaseUnits, imode: i32, ival: i32) -> i32 {
    match imode {
        0 => {
            units.imp = 6;
            units.ibb = 0;
            units.lec = 5;
        }
        1 => units.imp = ival,
        2 => units.ibb = ival,
        3 => units.lec = ival,
        _ => {}
    }
    0
}

/// Reports an error code of routine `cnompg` (`maermsg_`); a no-op returning 0.
/// `maermsg_("MMARCIN", 13)`.
pub fn maermsg_(_cnompg: &str, _icoder: i32) -> i32 {
    0
}

/// Traces the entry into routine `nomprg` (`mgenmsg_`); a no-op returning 0.
/// `mgenmsg_("MMAPCMP")`.
pub fn mgenmsg_(_nomprg: &str) -> i32 {
    0
}

/// Traces the exit from routine `nomprg` (`mgsomsg_`); a no-op returning 0.
/// `mgsomsg_("MMAPCMP")`.
pub fn mgsomsg_(_nomprg: &str) -> i32 {
    0
}

/// Writes a debug text when the debug level is at least 1 (`mswrdbg_`); since
/// [`mnfndeb_`] is 0 it writes nothing and returns 0. `mswrdbg_("MMRESOL")`.
pub fn mswrdbg_(_ctexte: &str) -> i32 {
    0
}

/// The debug level of the kernels (`mnfndeb_`): always 0 (production).
/// `assert_eq!(mnfndeb_(), 0)`.
pub fn mnfndeb_() -> i32 {
    0
}

/// The number of significant decimal digits of a double (`maovsr8_`, the
/// `r8ncs` of common `MAOVPAR`: 17); returns 0. `maovsr8_(&mut digits)`.
pub fn maovsr8_(ivalcs: &mut i32) -> i32 {
    *ivalcs = getmaovpar().r8ncs;
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macinit_sets_defaults_and_forces_one_unit_per_mode() {
        let mut units = SysBaseUnits::default();
        macinit_(&mut units, 0, 99);
        assert_eq!((units.imp, units.ibb, units.lec), (6, 0, 5));
        macinit_(&mut units, 1, 7);
        macinit_(&mut units, 2, 3);
        macinit_(&mut units, 3, 4);
        macinit_(&mut units, 9, 8);
        assert_eq!((units.imp, units.ibb, units.lec), (7, 3, 4));
    }

    #[test]
    fn trace_hooks_do_nothing_and_the_precision_is_17_digits() {
        assert_eq!(
            maermsg_("X", 1) + mgenmsg_("X") + mgsomsg_("X") + mswrdbg_("X"),
            0
        );
        assert_eq!(mnfndeb_(), 0);
        let mut digits: i32 = 0;
        maovsr8_(&mut digits);
        assert_eq!(digits, 17);
    }
}
