// Created on: 1991-01-21
// Created by: Christophe MARION
// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/TopLoc at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Shared datum chains with eagerly computed transformations (OCCT TopLoc, OCC25545).
//!
//! A [`Location`] is an immutable, shared list of `(datum, power)` items. Each list node
//! caches the product of its own powered datum and every item behind it, so reading the
//! transformation is a field read that is safe on many threads at once.
mod datum3d;
mod item_location;
mod location;
mod location_ops;
mod slist;
pub use datum3d::Datum3;
pub use item_location::ItemLocation;
pub use location::Location;
use rustycad_tkernel::ncollection::{IndexedMap, Map};
pub use slist::{SListNodeOfItemLocation, SListOfItemLocation};
/// `TopLoc_MapOfLocation`: locations hashed by datum identity and power; `MapOfLocation::default()`.
pub type MapOfLocation = Map<Location>;
/// `TopLoc_IndexedMapOfLocation`, keyed like [`MapOfLocation`]; `IndexedMapOfLocation::new(1)`.
pub type IndexedMapOfLocation = IndexedMap<Location>;
