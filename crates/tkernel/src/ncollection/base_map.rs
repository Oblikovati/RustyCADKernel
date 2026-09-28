// Created by: Alexander GRIGORIEV
// Copyright (c) 2002-2014 OPEN CASCADE SAS
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

//! The public surface `NCollection_BaseMap` gives every hashed map.
use super::{BaseAllocator, DataMap, DoubleMap, HashPolicy, IndexedDataMap, IndexedMap, Map};
use std::fmt::Debug;

/// Bucket and size queries shared by the hashed maps, as `NCollection_BaseMap` declares them.
///
/// ```
/// use rustycad_tkernel::ncollection::{BaseMap, Map};
/// let mut map = Map::<i32>::default();
/// map.add(7);
/// assert_eq!((BaseMap::extent(&map), BaseMap::nb_buckets(&map)), (1, 101));
/// ```
pub trait BaseMap {
    /// `NbBuckets()`: the hash modulus, kept even after `Clear(true)` releases the buckets.
    fn nb_buckets(&self) -> usize;
    /// `Size()`: the number of stored entries.
    fn size(&self) -> usize;
    /// `Allocator()`: the node allocator.
    fn allocator(&self) -> &BaseAllocator;
    /// `Extent()`: the entry count narrowed like the native `static_cast<int>`.
    fn extent(&self) -> i32 {
        self.size() as i32
    }
    /// `Length()`: the same narrowed entry count as `Extent()`.
    fn length(&self) -> i32 {
        self.size() as i32
    }
    /// `IsEmpty()`: true when no entry is stored.
    fn is_empty(&self) -> bool {
        self.size() == 0
    }
}

impl<K: Debug, V, H: HashPolicy<K>> BaseMap for DataMap<K, V, H> {
    fn nb_buckets(&self) -> usize {
        self.bucket_count()
    }
    fn size(&self) -> usize {
        self.len()
    }
    fn allocator(&self) -> &BaseAllocator {
        DataMap::allocator(self)
    }
}

impl<K: Debug, H: HashPolicy<K>> BaseMap for Map<K, H> {
    fn nb_buckets(&self) -> usize {
        self.bucket_count()
    }
    fn size(&self) -> usize {
        self.len()
    }
    fn allocator(&self) -> &BaseAllocator {
        Map::allocator(self)
    }
}

impl<K: Debug, H: HashPolicy<K>> BaseMap for IndexedMap<K, H> {
    fn nb_buckets(&self) -> usize {
        self.bucket_count()
    }
    fn size(&self) -> usize {
        self.len()
    }
    fn allocator(&self) -> &BaseAllocator {
        IndexedMap::allocator(self)
    }
}

impl<K: Debug, V, H: HashPolicy<K>> BaseMap for IndexedDataMap<K, V, H> {
    fn nb_buckets(&self) -> usize {
        self.bucket_count()
    }
    fn size(&self) -> usize {
        self.len()
    }
    fn allocator(&self) -> &BaseAllocator {
        IndexedDataMap::allocator(self)
    }
}

impl<A: Debug, B: Debug, H1: HashPolicy<A>, H2: HashPolicy<B>> BaseMap for DoubleMap<A, B, H1, H2> {
    fn nb_buckets(&self) -> usize {
        self.bucket_count()
    }
    fn size(&self) -> usize {
        self.len()
    }
    fn allocator(&self) -> &BaseAllocator {
        DoubleMap::allocator(self)
    }
}

/// `Clear(const Handle(NCollection_BaseAllocator)&)`: bucket memory is released only when the
/// allocator changes, and a null handle (`None`) selects the common allocator.
pub(super) fn replace_allocator(
    current: &mut BaseAllocator,
    requested: Option<BaseAllocator>,
) -> bool {
    let release = requested.as_ref() != Some(current);
    *current = requested.unwrap_or_else(BaseAllocator::common);
    release
}

/// The Standard_NoSuchObject an OCCT map `Iterator` raises when read past its end.
pub(super) fn exhausted_iterator(method: &str) -> crate::standard::Failure {
    crate::standard::Failure::with_type(
        "Standard_NoSuchObject",
        &format!("{method}: iterator exhausted; expected More() to be true"),
        "",
    )
}
