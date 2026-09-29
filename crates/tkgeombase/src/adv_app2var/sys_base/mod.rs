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
// Rust translation of ModelingData/TKGeomBase/AdvApp2Var/AdvApp2Var_SysBase.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AdvApp2Var_SysBase`: the Fortran system layer of the f2c kernels.
//!
//! The static members are memory fills, trace hooks that do nothing, and the unit
//! initialisation `macinit_`; the instance members are a dynamic allocator that
//! hands out scratch zones and records them in the `MCRGENE` table.
//!
//! The native allocator returns an *offset* between the caller's reference array
//! and a heap zone, so that `t[iofset + i]` addresses the zone; that is pointer
//! arithmetic. Here each zone is an owned byte buffer, `iofset` is a positive key
//! that stays unique while the zone lives, and [`AdvApp2VarSysBase::zone_mut`]
//! lends the zone. The ported kernels, whose native code creates one
//! `AdvApp2Var_SysBase` per call only for scratch memory, use local buffers
//! instead: the allocator can refuse only a 33rd live request or a unit other than
//! 1, 2, 4 or 8, and no kernel makes either request.
mod allocation;
mod memory;
mod messages;

pub use memory::{maitbr8_, mcrfill_, mcrfill_within, miraz_, msifill_, msrfill_, mvriraz_};
pub use messages::{
    SysBaseUnits, macinit_, maermsg_, maovsr8_, mgenmsg_, mgsomsg_, mnfndeb_, mswrdbg_,
};

/// Maximum number of live allocations (`MAX_ALLOC_NB`); the native comment names 7 as
/// the largest known need (`mmresol_`).
pub const MAX_ALLOC_NB: usize = 32;

/// The `alloctype` of a heap zone (`heap_allocation`); static zones are unused natively.
const HEAP_ALLOCATION: usize = 1;

/// The value of the start and end guard words of a zone (`macrgfl_`).
const GUARD_WORD: f64 = -134744073.;

/// One live allocation of the `MCRGENE` table (`mitem`).
#[derive(Clone, Debug, PartialEq)]
struct Mitem {
    prot: u8,
    unit: u8,
    reqsize: i32,
    loc: usize,
    offset: isize,
    alloctype: u8,
    size: i32,
    rank: i32,
    guards: [f64; 2],
    zone: Vec<u8>,
}

/// The dynamic allocator of the f2c kernels (`AdvApp2Var_SysBase`).
///
/// `let mut sys = AdvApp2VarSysBase::new(); let mut ofs = 0; let mut ier = 0;
/// sys.macrar8_(200, 100, 0, &mut ofs, &mut ier);`.
#[derive(Debug, Default)]
pub struct AdvApp2VarSysBase {
    /// `mcrgene_.icore[0..ncore]`.
    icore: Vec<Mitem>,
    /// `mcrgene_.lprot`: the protection level of the next request.
    lprot: u8,
    /// `mcrstac_`: requests, releases, live and peak bytes, per allocation type.
    nrqst: [i32; 2],
    ndelt: [i32; 2],
    nbyte: [i32; 2],
    mbyte: [i32; 2],
    /// The key of the next zone; the native offset is an address difference.
    next_offset: isize,
}

impl AdvApp2VarSysBase {
    /// An allocator with an empty table and zero statistics (`AdvApp2Var_SysBase()`,
    /// which calls `mainial_`); `AdvApp2VarSysBase::new()`.
    pub fn new() -> Self {
        let mut sys: Self = Self::default();
        sys.mainial_();
        sys
    }

    /// Empties the allocation table and clears the protection level (`mainial_`);
    /// returns 0. `sys.mainial_()`.
    pub fn mainial_(&mut self) -> i32 {
        self.icore.clear();
        self.lprot = 0;
        0
    }

    /// The number of live allocations (`mcrgene_.ncore`); `sys.ncore()`.
    pub fn ncore(&self) -> usize {
        self.icore.len()
    }

    /// The live bytes of heap zones (`mcrstac_.nbyte[1]`); `sys.live_bytes()`.
    pub fn live_bytes(&self) -> i32 {
        self.nbyte[HEAP_ALLOCATION]
    }

    /// The user zone of the live allocation keyed by `iofset`, or `None` when no
    /// allocation has this key; the zone has `unit * isize` bytes rounded up to 8.
    /// `sys.zone_mut(ofs)`.
    pub fn zone_mut(&mut self, iofset: isize) -> Option<&mut [u8]> {
        let item: &mut Mitem = self.icore.iter_mut().find(|item| item.offset == iofset)?;
        Some(&mut item.zone)
    }
}
