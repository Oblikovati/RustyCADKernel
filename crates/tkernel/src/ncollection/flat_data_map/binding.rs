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

use super::{Debug, FlatDataMap, HashPolicy, Inserted};
impl<K: Debug, V, H: HashPolicy<K>> FlatDataMap<K, V, H> {
    /// Inserts or replaces a value, reporting a new key; `map.bind(1,42)`.
    pub fn bind(&mut self, key: K, value: V) -> bool {
        self.table.insert(key, value, false, true).is_new
    }
    /// Inserts only when absent; `map.try_bind(1,42)`.
    pub fn try_bind(&mut self, key: K, value: V) -> bool {
        self.table.insert(key, value, false, false).is_new
    }
    /// Inserts or replaces and returns the original key's slot; `map.bound(1,42)`.
    pub fn bound(&mut self, key: K, value: V) -> &mut V {
        let result = self.table.insert(key, value, false, true);
        self.value_at(result.original)
    }
    /// Returns an existing value or inserts it without replacement; `map.try_bound(1,42)`.
    pub fn try_bound(&mut self, key: K, value: V) -> &mut V {
        let result = self.table.insert(key, value, false, false);
        self.value_at(result.original)
    }
    /// Constructs a replacement value and reports insertion; `map.emplace(1,||42)`.
    pub fn emplace(&mut self, key: K, construct: impl FnOnce() -> V) -> bool {
        self.place(key, construct, true).is_new
    }
    /// Constructs a replacement and returns its slot; `map.emplaced(1,||42)`.
    pub fn emplaced(&mut self, key: K, construct: impl FnOnce() -> V) -> &mut V {
        let result = self.place(key, construct, true);
        self.value_at(result.original)
    }
    /// Constructs only for absent keys; `map.try_emplace(1,||42)`.
    pub fn try_emplace(&mut self, key: K, construct: impl FnOnce() -> V) -> bool {
        self.place(key, construct, false).is_new
    }
    /// Borrows an existing value or constructs it lazily; `map.try_emplaced(1,||42)`.
    pub fn try_emplaced(&mut self, key: K, construct: impl FnOnce() -> V) -> &mut V {
        let result = self.place(key, construct, false);
        self.value_at(result.original)
    }
    fn place(&mut self, key: K, construct: impl FnOnce() -> V, replace: bool) -> Inserted {
        self.table.prepare();
        if let Some(index) = self.table.lookup(&key) {
            if replace {
                *self.value_at(index) = construct();
            }
            return Inserted {
                is_new: false,
                original: index,
                last: index,
            };
        }
        self.table.insert(key, construct(), false, false)
    }
    fn value_at(&mut self, index: usize) -> &mut V {
        &mut self.table.slots[index]
            .as_mut()
            .expect("inserted slot is occupied")
            .value
    }
}
