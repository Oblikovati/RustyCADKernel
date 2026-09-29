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
// Rust translation of ModelingData/TKGeomBase/AdvApp2Var/AdvApp2Var_ApproxF2var.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AdvApp2Var_ApproxF2var`: the f2c routines of the two-variable approximation.
//!
//! [`mma2fnc_`] approximates the iso-curves of the constraint frame, [`mma2cdi_`]
//! and [`mma2ds1_`] discretise a patch, [`mma2ce1_`] approximates it by a Jacobi
//! patch and decides the cutting, [`mma2can_`] converts it to the canonical base,
//! [`mma2ac1_`], [`mma2ac2_`] and [`mma2ac3_`] add the boundary constraints back and
//! [`mma2fx6_`] lowers the degrees. [`mma1her_`], [`mma2roo_`], [`mma2jmx_`] and
//! [`mmapptt_`] give the fixed bases.
//!
//! The native routines take up to 26 pointers; the port groups the arrays that
//! travel together (the corner constraints, the iso tables, the four symmetric
//! square tables, the request and output of a patch or iso approximation) into
//! structs whose fields keep the native names and order.
#![allow(clippy::too_many_arguments)] // The f2c routines keep the Fortran argument lists.

mod bases;
mod constraints;
mod corners;
mod curve;
mod curve_errors;
mod curve_steps;
mod degree;
mod discretise;
mod patch_approx;
mod patch_cutting;
mod projection;
mod sampling;

pub use bases::{mma1her_, mma2jmx_, mma2roo_, mmapptt_};
pub use constraints::{CornerConstraints, IsoConstraints, mma2ac1_, mma2ac2_, mma2ac3_, mma2can_};
pub use curve::{IsoApproxOutput, IsoApproxRequest, mma2fnc_};
pub use degree::{PatchGrid, mma2fx6_};
pub use discretise::{IsoTables, SquareGrid, SquareTables, mma2cdi_};
pub use patch_approx::{PatchApproxOutput, PatchApproxRequest, SquareSamples, mma2ce1_};
pub use sampling::{SampledSquare, mma2ds1_};
