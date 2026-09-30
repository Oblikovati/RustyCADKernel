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
// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepCheck at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package BRepCheck: the validity checks of a BRep shape.
//!
//! Each checker derives from the abstract [`BRepCheckResult`] (docs/topalgo-design.md
//! section 6): [`BRepCheckVertex`], [`BRepCheckEdge`], [`BRepCheckWire`],
//! [`BRepCheckFace`], [`BRepCheckShell`] and [`BRepCheckSolid`] record the defects
//! ([`BRepCheckStatus`]) of their shape itself and of the shape in each context it
//! was checked in, and [`BRepCheckAnalyzer`] runs them over a whole shape.
mod edge;
mod edge_context;
mod edge_tolerance;
mod errors;
mod explore;
mod face;
mod face_classify;
mod face_wires;
mod package;
mod result;
mod shell;
mod shell_closed;
mod shell_orientation;
mod status;
mod vertex;
mod wire;
mod wire_choix_uv;
mod wire_closed;
mod wire_orientation;
mod wire_self_intersect;
mod wire_self_pairs;

pub use edge::BRepCheckEdge;
pub use face::BRepCheckFace;
pub use package::BRepCheck;
pub use result::{BRepCheckResult, ResultBase};
pub use shell::BRepCheckShell;
pub use status::{BRepCheckStatus, ListOfStatus};
pub use vertex::BRepCheckVertex;
pub use wire::BRepCheckWire;
