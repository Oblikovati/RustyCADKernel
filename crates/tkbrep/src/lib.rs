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
// Rust translation of ModelingData/TKBRep at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT TKBRep: the boundary representation data structure.
//!
//! TKBRep depends on TKernel, TKMath, TKG2d, TKG3d and TKGeomBase natively; this
//! crate depends on their ports. Each OCCT package is one module, added when it is
//! ported: `topods` (the topological data structure), `topexp` (its exploration),
//! `toptools` (maps, hashers and the shape set), `brep` (the geometric shapes,
//! their builder and their queries), `brep_tools`, `brep_adaptor`, `brep_lprop` and
//! `bintools`. docs/topology-design.md fixes the Rust mapping
//! and docs/tkbrep-progress.md lists the ported packages.
pub mod bintools;
pub mod brep;
pub mod brep_adaptor;
pub mod brep_graph;
pub mod brep_graph_inc;
pub mod brep_lprop;
pub mod brep_tools;
pub mod topexp;
pub mod topods;
pub mod toptools;
