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

//! Behavior of the Message package members that need no process-wide state: statuses,
//! messages and catalogs, alerts and reports, printers and algorithms.
#[path = "message_api_behavior/algorithm.rs"]
mod algorithm;
#[path = "message_api_behavior/catalog.rs"]
mod catalog;
#[path = "message_api_behavior/fakes.rs"]
mod fakes;
#[path = "message_api_behavior/msg.rs"]
mod msg;
#[path = "message_api_behavior/printers.rs"]
mod printers;
#[path = "message_api_behavior/report.rs"]
mod report;
#[path = "message_api_behavior/status.rs"]
mod status;
