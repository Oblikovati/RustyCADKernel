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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/NLPlate at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package NLPlate: empty at this commit, so there is nothing to port.
//!
//! `PACKAGES.cmake` still lists NLPlate, but its `FILES.cmake` declares no file: OCCT
//! commit caad597813 ("Clean up dead headers #1442") deleted every NLPlate source and
//! header (the non-linear plate `NLPlate_NLPlate` and its `NLPlate_HGPPConstraint`
//! family), and no toolkit uses them. The only remaining references are two
//! deprecated aliases in `src/Deprecated/NCollectionAliases` (outside TKGeomAlgo):
//! `NLPlate_StackOfPlate.hxx`, a typedef of `NCollection_List<Plate_Plate>`, and
//! `NLPlate_SequenceOfHGPPConstraint.hxx`, which includes the deleted
//! `NLPlate_HGPPConstraint.hxx` and so no longer compiles. `tools/audit_api_rows.py`
//! finds no public member, and this module stays empty until OCCT adds sources again.
