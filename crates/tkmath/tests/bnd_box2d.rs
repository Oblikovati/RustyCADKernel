// Copyright (c) 2025 OPEN CASCADE SAS
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

//! Complete Bnd_Box2d legacy suite.
#![allow(non_snake_case, unused_mut, unused_assignments, unused_variables)]
use rustycad_tkmath::{
    bnd::BoundingBox2,
    gp::{Direction2, Line2, Point2, Transform2},
};
include!("bnd_box2d/suites_01.rs");
include!("bnd_box2d/suites_11.rs");
include!("bnd_box2d/suites_21.rs");
include!("bnd_box2d/suites_31.rs");
