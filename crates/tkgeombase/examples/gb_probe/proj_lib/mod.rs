// Copyright (c) 1993-1999 Matra Datavision
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

//! The Rust twin of tools/oracle/tkgeombase_probe_projlib.cxx and
//! tkgeombase_probe_projlib_curves.cxx.
mod comp;
mod curves;
mod elementary;
mod on_plane;

pub use comp::comp_case;
pub use curves::{approx_case, polar_case, projected_case};
pub use elementary::{elem_case, prjfunc_case, prjresolve_case, static_case};
pub use on_plane::on_plane_case;
