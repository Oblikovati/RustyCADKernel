// Copyright (c) 1991-1999 Matra Datavision
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

//! CSLib_DerivativeStatus and CSLib_NormalStatus with their native discriminants.
use crate::gp::Direction3;

/// Outcome of a normal computed from first derivatives; `DerivativeStatus::Done as i32 == 0`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum DerivativeStatus {
    /// Normal computed successfully.
    Done = 0,
    /// D1U has null length: `||D1U||^2 <= Resolution`.
    D1uIsNull = 1,
    /// D1V has null length: `||D1V||^2 <= Resolution`.
    D1vIsNull = 2,
    /// Both derivatives are null.
    D1IsNull = 3,
    /// D1U is negligible compared to D1V (declared, never produced by CSLib).
    D1uD1vRatioIsNull = 4,
    /// D1V is negligible compared to D1U (declared, never produced by CSLib).
    D1vD1uRatioIsNull = 5,
    /// D1U and D1V are parallel.
    D1uIsParallelD1v = 6,
}

/// Outcome of a normal computed at a possibly singular point; `NormalStatus::Defined as i32 == 1`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum NormalStatus {
    /// Surface is singular at the point (normal undefined).
    Singular = 0,
    /// Normal is well defined.
    Defined = 1,
    /// Infinitely many normals are possible.
    InfinityOfSolutions = 2,
    /// DN/du has null length.
    D1NuIsNull = 3,
    /// DN/dv has null length.
    D1NvIsNull = 4,
    /// Both normal derivatives are null.
    D1NIsNull = 5,
    /// DN/du is negligible compared to DN/dv.
    D1NuNvRatioIsNull = 6,
    /// DN/dv is negligible compared to DN/du.
    D1NvNuRatioIsNull = 7,
    /// DN/du and DN/dv are parallel.
    D1NuIsParallelD1Nv = 8,
}

/// A status plus the normal the native call writes only on success.
///
/// `normal` is `None` exactly when OCCT leaves its `gp_Dir&` output untouched;
/// `let result = cslib::normal(d1u, d1v, 1e-10)?; result.normal.is_some()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NormalResult<S> {
    /// The status OCCT always writes.
    pub status: S,
    /// The normal, when OCCT writes it.
    pub normal: Option<Direction3>,
}

impl<S> NormalResult<S> {
    /// Reports whether a normal was written (the native `theDone` flag); `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.normal.is_some()
    }
    pub(crate) fn untouched(status: S) -> Self {
        Self {
            status,
            normal: None,
        }
    }
    pub(crate) fn written(status: S, normal: Direction3) -> Self {
        Self {
            status,
            normal: Some(normal),
        }
    }
}
