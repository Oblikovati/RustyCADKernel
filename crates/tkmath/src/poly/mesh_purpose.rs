// Copyright (c) 2021 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/Poly/Poly_MeshPurpose.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Poly_MeshPurpose`: an `unsigned int` bit set, kept as plain flags so user bits combine freely.
/// Purpose bits of a triangulation; `mesh_purpose::CALCULATION | mesh_purpose::LOADED`.
pub type MeshPurpose = u32;
/// No special use (default).
pub const NONE: MeshPurpose = 0;
/// Mesh for algorithms.
pub const CALCULATION: MeshPurpose = 0x0001;
/// Mesh for presentation (LODs usage).
pub const PRESENTATION: MeshPurpose = 0x0002;
/// Mesh marked as currently active in a list; not set externally.
pub const ACTIVE: MeshPurpose = 0x0004;
/// Mesh has currently loaded data; not set externally.
pub const LOADED: MeshPurpose = 0x0008;
/// Lets BRep_Tools::Triangulation() fall back to any other defined mesh.
pub const ANY_FALLBACK: MeshPurpose = 0x0010;
/// First application-defined flag.
pub const USER: MeshPurpose = 0x0020;
