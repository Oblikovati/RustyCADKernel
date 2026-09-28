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

//! Shared hash chains and insertion-order links over stable allocator-owned nodes.
use super::hash_policy::next_map_prime;
use super::{AllocatedBox, BaseAllocator, HashPolicy};
pub(super) struct OrderedNode<K, V> {
    pub key: K,
    pub value: V,
    previous: Option<usize>,
    pub next: Option<usize>,
}
pub(super) type OrderedSlots<K, V> = Vec<Option<AllocatedBox<OrderedNode<K, V>>>>;
pub(super) struct OrderedTable<K, V, H> {
    pub slots: OrderedSlots<K, V>,
    free: Vec<usize>,
    buckets: Vec<Vec<usize>>,
    pub count: usize,
    pub bucket_count: usize,
    pub first: Option<usize>,
    pub last: Option<usize>,
    pub resource: BaseAllocator,
    pub hasher: H,
}
pub(super) struct OrderedItems<'a, K, V> {
    pub slots: &'a [Option<AllocatedBox<OrderedNode<K, V>>>],
    pub current: Option<usize>,
    // The map's head when the iterator was created: native `Reset()` restarts there.
    pub first: Option<usize>,
}
impl<K, V, H: HashPolicy<K>> OrderedTable<K, V, H> {
    pub fn new(hasher: H, buckets: usize, resource: BaseAllocator) -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
            buckets: Vec::new(),
            count: 0,
            // NCollection_BaseMap keeps a requested zero; the first insertion resizes to a prime.
            bucket_count: buckets,
            first: None,
            last: None,
            resource,
            hasher,
        }
    }
    pub fn resize(&mut self, requested: usize) {
        let count = next_map_prime(requested).max(self.bucket_count);
        if count == self.bucket_count && !self.buckets.is_empty() {
            return;
        }
        let mut buckets: Vec<Vec<usize>> = (0..=count).map(|_| Vec::new()).collect();
        for &slot in self.buckets.iter().flatten() {
            let hash = self.hasher.hash(&self.slots[slot].as_ref().unwrap().key) % count + 1;
            buckets[hash].insert(0, slot);
        }
        self.buckets = buckets;
        self.bucket_count = count;
    }
    pub fn lookup(&self, key: &K) -> Option<usize> {
        if self.count == 0 {
            return None;
        }
        let hash = self.hasher.hash(key) % self.bucket_count + 1;
        self.buckets[hash].iter().copied().find(|&slot| {
            self.hasher
                .equal(&self.slots[slot].as_ref().unwrap().key, key)
        })
    }
    pub fn place(
        &mut self,
        key: K,
        construct: impl FnOnce() -> V,
        replace_key: bool,
        replace_value: bool,
    ) -> (bool, usize) {
        if self.count == 0 || self.count > self.bucket_count {
            self.resize(self.count);
        }
        if let Some(slot) = self.lookup(&key) {
            let node = self.slots[slot].as_mut().unwrap();
            if replace_key {
                node.key = key;
            }
            if replace_value {
                node.value = construct();
            }
            return (false, slot);
        }
        (true, self.append_node(key, construct()))
    }
    fn append_node(&mut self, key: K, value: V) -> usize {
        let hash = self.hasher.hash(&key) % self.bucket_count + 1;
        let slot = self.free.pop().unwrap_or_else(|| {
            self.slots.push(None);
            self.slots.len() - 1
        });
        self.slots[slot] = Some(self.resource.allocate_value(OrderedNode {
            key,
            value,
            previous: self.last,
            next: None,
        }));
        if let Some(last) = self.last {
            self.slots[last].as_mut().unwrap().next = Some(slot);
        } else {
            self.first = Some(slot);
        }
        self.last = Some(slot);
        self.buckets[hash].insert(0, slot);
        self.count += 1;
        slot
    }
    pub fn remove(&mut self, key: &K) -> bool {
        let Some(slot) = self.lookup(key) else {
            return false;
        };
        let node = self.slots[slot].take().unwrap();
        let hash = self.hasher.hash(&node.key) % self.bucket_count + 1;
        self.buckets[hash].retain(|&entry| entry != slot);
        if let Some(previous) = node.previous {
            self.slots[previous].as_mut().unwrap().next = node.next;
        } else {
            self.first = node.next;
        }
        if let Some(next) = node.next {
            self.slots[next].as_mut().unwrap().previous = node.previous;
        } else {
            self.last = node.previous;
        }
        self.free.push(slot);
        self.count -= 1;
        true
    }
    pub fn clear(&mut self, release: bool) {
        self.slots.clear();
        self.free.clear();
        self.first = None;
        self.last = None;
        self.count = 0;
        if release {
            self.buckets.clear();
        } else {
            for bucket in &mut self.buckets {
                bucket.clear();
            }
        }
    }
    /// Native `Clear(theAllocator)`: release bucket storage only when the allocator changes,
    /// then adopt `resource`, or the common allocator for a null handle.
    pub fn clear_with_allocator(&mut self, resource: Option<BaseAllocator>) {
        self.clear(resource.as_ref() != Some(&self.resource));
        self.resource = resource.unwrap_or_else(BaseAllocator::common);
    }
    pub fn items(&self) -> OrderedItems<'_, K, V> {
        OrderedItems {
            slots: &self.slots,
            current: self.first,
            first: self.first,
        }
    }
}
impl<K: Clone, V: Clone, H: HashPolicy<K> + Clone> OrderedTable<K, V, H> {
    pub fn assign(&mut self, source: &Self, replace_values: bool) {
        self.clear(false);
        if source.count == 0 {
            return;
        }
        self.resize(source.count - 1);
        for (key, value) in source.items() {
            self.place(key.clone(), || value.clone(), false, replace_values);
        }
    }
}
impl<K: Clone, V: Clone, H: HashPolicy<K> + Clone> Clone for OrderedTable<K, V, H> {
    fn clone(&self) -> Self {
        let mut copy = Self::new(
            self.hasher.clone(),
            self.bucket_count,
            self.resource.clone(),
        );
        copy.assign(self, false);
        copy
    }
}
/// `NCollection_BaseMap::nbBucketsFromInt`: a negative signed bucket count raises
/// Standard_OutOfRange; `buckets_from_int(-1)` is an error naming -1.
pub(super) fn buckets_from_int(buckets: i32) -> Result<usize, crate::standard::Failure> {
    usize::try_from(buckets).map_err(|_| {
        crate::standard::Failure::with_type(
            "Standard_OutOfRange",
            &format!("bucket count {buckets}; expected a non-negative count"),
            "",
        )
    })
}
mod iteration;
mod nodes;
pub use nodes::{OrderedNodeMut, OrderedNodeRef};

mod mutable;
pub(super) use mutable::OrderedItemsMut;
