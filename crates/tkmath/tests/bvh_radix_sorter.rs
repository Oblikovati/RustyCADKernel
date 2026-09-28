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

//! Direct BVH_RadixSorter_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, unused_mut, clippy::manual_abs_diff)]
use rustycad_tkmath::bvh::encode_morton_code;

include!("bvh_radix_sorter/suites_0.rs");
include!("bvh_radix_sorter/suites_1.rs");
