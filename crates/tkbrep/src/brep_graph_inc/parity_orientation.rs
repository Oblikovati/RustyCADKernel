// Copyright (c) 2026 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_ParityOrientation.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraphInc::ParityOrientation`: an orientation stored as one parity bit.
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;

/// `BRepGraphInc::ParityOrientation`: FORWARD or REVERSED only, stored as
/// `IsReversed`; the default is FORWARD.
///
/// ```
/// use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
/// use rustycad_tkg3d::top_abs::Orientation;
/// let parity = ParityOrientation::new(Orientation::Reversed).unwrap();
/// assert!(parity.is_reversed);
/// assert_eq!(parity.orientation(), Orientation::Reversed);
/// assert!(ParityOrientation::new(Orientation::Internal).is_err());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ParityOrientation {
    /// `IsReversed`: false for FORWARD, true for REVERSED.
    pub is_reversed: bool,
}

impl ParityOrientation {
    /// FORWARD.
    pub const FORWARD: Self = Self { is_reversed: false };
    /// REVERSED.
    pub const REVERSED: Self = Self { is_reversed: true };

    /// `ParityOrientation(theOrientation)`: INTERNAL or EXTERNAL raises
    /// Standard_ProgramError, as `toIsReversed` does.
    pub fn new(orientation: Orientation) -> Result<Self, Failure> {
        Ok(Self {
            is_reversed: to_is_reversed(orientation)?,
        })
    }

    /// `operator=(theOrientation)`: the same check as [`new`](Self::new).
    pub fn assign(&mut self, orientation: Orientation) -> Result<(), Failure> {
        self.is_reversed = to_is_reversed(orientation)?;
        Ok(())
    }

    /// `operator TopAbs_Orientation()`.
    pub fn orientation(self) -> Orientation {
        if self.is_reversed {
            Orientation::Reversed
        } else {
            Orientation::Forward
        }
    }
}

impl From<ParityOrientation> for Orientation {
    fn from(parity: ParityOrientation) -> Self {
        parity.orientation()
    }
}

fn to_is_reversed(orientation: Orientation) -> Result<bool, Failure> {
    match orientation {
        Orientation::Forward => Ok(false),
        Orientation::Reversed => Ok(true),
        other => Err(Failure::with_type(
            "Standard_ProgramError",
            &format!("BRepGraphInc::ParityOrientation stores only FORWARD/REVERSED: got {other:?}"),
            "",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_core_orientations_are_stored() {
        let mut parity = ParityOrientation::default();
        assert_eq!(Orientation::from(parity), Orientation::Forward);
        parity.assign(Orientation::Reversed).unwrap();
        assert_eq!(parity, ParityOrientation::REVERSED);
        let error = parity.assign(Orientation::External).unwrap_err();
        assert_eq!(error.exception_type(), "Standard_ProgramError");
        assert_eq!(parity, ParityOrientation::REVERSED);
    }
}
