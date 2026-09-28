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

//! Exact ports of the libstdc++ 13 algorithms whose unstable element order is
//! observable in OCCT results on the Linux/GCC oracle profile.
//!
//! `std::sort` and `std::partition` do not preserve the order of equivalent
//! elements. OCCT code such as `BVH_QuickSorter` and `BVH_RadixSorter` applies the
//! resulting permutation in place, so the order of ties is part of the observable
//! output. Rust's own sorts use other algorithms and resolve ties differently.
//!
//! As in C++, the comparator must be a strict weak ordering. Out-of-contract
//! comparators, which are undefined behavior upstream, panic on a bounds check here.
mod heap;
mod partition;
mod sort;
pub use partition::partition;
pub use sort::sort_by;
