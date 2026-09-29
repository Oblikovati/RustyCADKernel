// Copyright (c) 1995-1999 Matra Datavision
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

//! AppDef behaviour: the error contracts and the preserved native quirks of the
//! containers, the line tool, LinearCriteria and Variational. The package has no
//! upstream GoogleTest suite; bit-exact parity with the native kernel is the numeric
//! oracle's job (tools/verify_tkgeombase_numeric.py), these tests pin the documented
//! behaviour. The process-wide reference cache of the FEmTool criteria is pinned in
//! its own binary (`app_def_reference_cache.rs`), as it depends on earlier calls.
mod containers;
mod criteria;
mod support;
mod variational;

pub use support::{arc_line, constraints, kind};
