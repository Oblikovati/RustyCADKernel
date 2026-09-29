// Created on: 2004-05-11
// Created by: Sergey ZARITCHNY <szy@opencascade.com>
// Copyright (c) 2004-2014 OPEN CASCADE SAS
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

//! BinTools behaviour: the scalar encodings and stream primitives, the geometry and
//! location sets, the grouped shape set, the shape-reference writer and reader, the
//! file variants, and the preserved native quirks. The package has no upstream
//! GoogleTest suite; byte-exact parity with the native kernel is the job of the br_bin*
//! requests of the numeric oracle (tools/verify_tkbrep_numeric.py).
mod fakes;
mod files;
mod references;
mod sets;
mod shape_set;
mod streams;
#[path = "../support/mod.rs"]
mod support;
