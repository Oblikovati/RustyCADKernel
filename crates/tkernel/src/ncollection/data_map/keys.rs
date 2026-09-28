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

// Set keys share the DataMap bucket engine without exposing mutable keys publicly.
use super::{DataMap, Debug, Entry, HashPolicy};
impl<K: Debug, H: HashPolicy<K>> DataMap<K, (), H> {
    pub(in crate::ncollection) fn stored_key(&self, key: &K) -> Option<&K> {
        let (bucket, index) = self.lookup(key)?;
        Some(&self.buckets[bucket][index].key)
    }
    pub(in crate::ncollection) fn place_key(&mut self, key: K, replace: bool) -> (bool, &K) {
        if self.is_empty() || self.count > self.bucket_count {
            self.resize(self.count);
        }
        if let Some((bucket, index)) = self.lookup(&key) {
            let stored = &mut self.buckets[bucket][index].key;
            if replace {
                *stored = key;
            }
            return (false, stored);
        }
        let bucket = self.hasher.hash(&key) % self.bucket_count + 1;
        let node = self.resource.allocate_value(Entry { key, value: () });
        self.buckets[bucket].insert(0, node);
        self.count += 1;
        (true, &self.buckets[bucket][0].key)
    }
}
