// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Behavior of the FSD drivers and Base64 over in-memory and failing file systems. The
//! native byte formats are compared with OCCT by the `stor_*` probe requests.
#[path = "fsd_api_behavior/base64.rs"]
mod base64;
#[path = "fsd_api_behavior/binary.rs"]
mod binary;
#[path = "fsd_api_behavior/fakes.rs"]
mod fakes;
#[path = "fsd_api_behavior/text.rs"]
mod text;
