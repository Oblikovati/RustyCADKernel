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

//! Direct MathPoly_Test.cxx translation at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(
    non_snake_case,
    clippy::excessive_precision,
    clippy::needless_range_loop
)]
use rustycad_tkmath::{math_poly::*, math_utils::Status};
include!("mathpoly/fixtures.rs");
include!("mathpoly/linear_quadratic.rs");
include!("mathpoly/cubic.rs");
include!("mathpoly/quartic.rs");
