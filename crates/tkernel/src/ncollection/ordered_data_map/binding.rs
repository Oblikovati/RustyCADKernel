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

use super::{Debug, HashPolicy, OrderedDataMap};
impl<K: Debug, V, H: HashPolicy<K>> OrderedDataMap<K, V, H> {
    /// Replaces existing values and reports new-key insertion; `map.bind(1,42)`.
    pub fn bind(&mut self, key: K, value: V) -> bool {
        self.table.place(key, || value, false, true).0
    }
    /// Retains existing values and reports new-key insertion; `map.try_bind(1,42)`.
    pub fn try_bind(&mut self, key: K, value: V) -> bool {
        self.table.place(key, || value, false, false).0
    }
    /// Replaces existing values and returns value storage; `map.bound(1,42)`.
    pub fn bound(&mut self, key: K, value: V) -> &mut V {
        let (_, slot) = self.table.place(key, || value, false, true);
        &mut self.table.slots[slot].as_mut().unwrap().value
    }
    /// Retains existing values and returns value storage; `map.try_bound(1,42)`.
    pub fn try_bound(&mut self, key: K, value: V) -> &mut V {
        let (_, slot) = self.table.place(key, || value, false, false);
        &mut self.table.slots[slot].as_mut().unwrap().value
    }
    /// Replaces existing values and reports new-key insertion; `map.emplace(1,||42)`.
    pub fn emplace(&mut self, key: K, construct: impl FnOnce() -> V) -> bool {
        self.table.place(key, construct, false, true).0
    }
    /// Replaces existing values and returns value storage; `map.emplaced(1,||42)`.
    pub fn emplaced(&mut self, key: K, construct: impl FnOnce() -> V) -> &mut V {
        let (_, slot) = self.table.place(key, construct, false, true);
        &mut self.table.slots[slot].as_mut().unwrap().value
    }
    /// Retains existing values and reports new-key insertion; `map.try_emplace(1,||42)`.
    pub fn try_emplace(&mut self, key: K, construct: impl FnOnce() -> V) -> bool {
        self.table.place(key, construct, false, false).0
    }
    /// Retains existing values and returns value storage; `map.try_emplaced(1,||42)`.
    pub fn try_emplaced(&mut self, key: K, construct: impl FnOnce() -> V) -> &mut V {
        let (_, slot) = self.table.place(key, construct, false, false);
        &mut self.table.slots[slot].as_mut().unwrap().value
    }
}
