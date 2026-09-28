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

//! Behaviour of the Quantity members completed beyond the upstream GoogleTest suites.
//! Bit-exact agreement with OCCT is checked by the `qc_*`/`qrgba_*` differential probe.
#[path = "quantity_api_behavior/colors.rs"]
mod colors;
#[path = "quantity_api_behavior/encodings.rs"]
mod encodings;
#[path = "quantity_api_behavior/rgba_and_time.rs"]
mod rgba_and_time;
