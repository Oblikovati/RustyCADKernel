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

//! Behavior of the Storage package: the iostream model, the data sections, callbacks,
//! buckets and the schema. File formats are covered by `fsd_api_behavior`; bit-exact
//! agreement with OCCT by the `stor_*` requests of the differential probe.
#[path = "storage_api_behavior/buckets.rs"]
mod buckets;
#[path = "storage_api_behavior/data.rs"]
mod data;
#[path = "storage_api_behavior/fakes.rs"]
mod fakes;
#[path = "storage_api_behavior/schema.rs"]
mod schema;
#[path = "storage_api_behavior/sections.rs"]
mod sections;
#[path = "storage_api_behavior/stream.rs"]
mod stream;
